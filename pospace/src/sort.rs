use crate::storage::{deserialize, plotentry_size, serialize, store_table_part, PlotEntry};
use anyhow::{Context, Result};
use log::*;
use std::fmt::Debug;
use std::fs::{read_dir, remove_file, rename, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::error::{MergeChunkError, SortError};
use crate::{table_final_filename_format, table_sorted_filename_format};
use std::collections::VecDeque;

pub fn sort_table_part(
    path: &Path,
    table_index: usize,
    part_index: usize,
    k: usize,
) -> Result<PathBuf> {
    info!("[Table {}] Sorting part {} ...", table_index, part_index);
    let mut buffer = Vec::new();
    let mut file = File::open(&path).context(format!(
        "Could not open plot file part {} of table {}",
        part_index, table_index
    ))?;
    file.read_to_end(&mut buffer).context(format!(
        "Could not read plot file part {} of table {}",
        part_index, table_index
    ))?;
    let mut entries = deserialize(&buffer, plotentry_size(table_index, k)).context(format!(
        "Could not deserialize part {} of table {}",
        part_index, table_index
    ))?;

    entries.sort_unstable();

    let out_path = path.parent().unwrap().join(format!(
        table_sorted_filename_format!(),
        table_index, part_index
    ));

    store_table_part(&entries, &out_path).context(format!(
        "Could not store table {} part {} to disk",
        table_index, part_index
    ))?;
    info!("[Table {}] Part {} sorted", table_index, part_index);
    Ok(out_path)
}

pub fn sort_table_on_disk(
    table_index: usize,
    path: &Path,
    entries_per_chunk: usize,
    k: usize,
) -> Result<()> {
    // Sort each bucket
    let mut chunks_count = 0;
    let mut parts = Vec::new();

    // Sort individual table parts
    for (index, entry) in read_dir(path)
        .context(format!("Could not read directory: {:?}", path))?
        .filter_map(Result::ok)
        .map(|x| x.path())
        .filter(|e| {
            e.is_file()
                && e.file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .starts_with(format!("table{}_raw_", table_index).as_str())
        })
        .enumerate()
    {
        let part_path = sort_table_part(&entry, table_index, index + 1, k)?;
        parts.push(part_path);
        chunks_count += 1;
    }

    // K-Way Merge sort

    if chunks_count > 1 {
        info!("[Table {}] K-Way merging ...", table_index);

        let mut state = KWayMerge::new(
            &parts,
            plotentry_size(table_index, k),
            entries_per_chunk,
            &path.join(format!(table_final_filename_format!(), table_index)),
            table_index,
        )
        .context(format!(
            "Could not start k-way merge for table {}",
            table_index
        ))?;

        while state
            .run_iteration()
            .context("An error occurred during a k-way merge iteration")?
            != KWayMergeState::Done
        {}

        info!("[Table {}] K-Way merge done", table_index);
    } else {
        rename(
            path.join(format!(table_sorted_filename_format!(), table_index, 1)),
            path.join(format!(table_final_filename_format!(), table_index)),
        )
        .or_else(|e| Err(SortError::RenameError(e.kind())))
        .context(format!(
            "Could not rename plot file for table {}",
            table_index
        ))?
    }

    info!("[Table {}] Cleaning intermediate files ...", table_index);
    read_dir(path)
        .context(format!(
            "Could not read directory ({:?}) to clean intermediate files",
            path
        ))?
        .filter_map(Result::ok)
        .map(|x| x.path())
        .filter(|e| {
            let filename = e.file_name().unwrap().to_str().unwrap();
            return e.is_file()
                && (filename.starts_with(format!("table{}_raw_", table_index).as_str())
                    || filename.starts_with(format!("table{}_sorted_", table_index).as_str()));
        })
        .map(|f| {
            remove_file(&f).or_else(|e| Err(SortError::DeleteError(f.to_owned(), e.kind()).into()))
        })
        .collect::<Result<()>>()
        .context(format!(
            "Could not clean intermediate files for table {}",
            table_index
        ))?;

    Ok(())
}

#[derive(Debug, PartialEq)]
enum KWayMergeState {
    Success,
    Done,
}

#[derive(Debug)]
struct KWayMerge {
    entries_per_chunk: usize,
    output_file: File,
    chunks: Vec<MergeChunk>,
    output: Vec<PlotEntry>,
    iter_count: usize,
    item_count: usize,
    table_index: usize,
}

impl KWayMerge {
    pub fn new(
        paths: &[PathBuf],
        entry_size: usize,
        entries_per_chunk: usize,
        output_file_path: &Path,
        table_index: usize,
    ) -> Result<Self> {
        let chunk_size = entries_per_chunk / (paths.len() - 1) * entry_size;
        let mut state = Self {
            entries_per_chunk,
            chunks: Vec::new(),
            output: Vec::new(),
            iter_count: 0,
            item_count: 0,
            output_file: File::create(output_file_path)
                .context(format!("Failed to create file: {:?}", output_file_path))?,
            table_index,
        };

        let mut id_counter = 1;

        for path in paths {
            let file = File::open(path)?;
            let file_size = file.metadata()?.len() as usize;
            let merge_chunk = MergeChunk {
                id: id_counter,
                file,
                entry_size,
                total_size: file_size,
                remaining_size: file_size,
                content: VecDeque::new(),
                chunk_size,
            };
            state.chunks.push(merge_chunk);
            id_counter += 1;
        }

        Ok(state)
    }

    pub fn run_iteration(&mut self) -> Result<KWayMergeState> {
        // Load new data into chunks if they are empty
        for chunk in self.chunks.iter_mut() {
            // Refill chunk
            chunk.refill().context("Failed to refill chunk")?;
        }

        // Find the min
        let min = self
            .find_min_chunk()
            .context("Failed to find minimum among chunks")?;
        let min_chunk = &mut self.chunks[min];

        // Move the minimum value to the output vec
        // Delete the minimum from the chunk (increase the index)
        self.output.push(
            min_chunk
                .content
                .pop_front()
                .ok_or(MergeChunkError::MinChunkIsEmpty)?,
        );

        // Write output if it is full
        if self.output.len() >= self.entries_per_chunk {
            self.write_output().context("Failed to write output data")?;
            self.output.clear();
        }

        // Keeping only unfinished chunks
        self.chunks.retain(|x| !x.is_done());

        if self.chunks.len() == 0 {
            self.write_output().context("Failed to write output data")?;
            info!(
                "[Table {}] Final part {} written",
                self.table_index, self.iter_count
            );
            return Ok(KWayMergeState::Done);
        }

        Ok(KWayMergeState::Success)
    }

    pub fn find_min_chunk(&self) -> Result<usize> {
        if self.chunks.iter().all(|c| c.content.get(0).is_some()) {
            Ok(self
                .chunks
                .iter()
                .map(|c| &c.content[0])
                .collect::<Vec<&PlotEntry>>()
                .iter()
                .enumerate()
                .min_by_key(|&(_, x)| x)
                .ok_or(MergeChunkError::EmptyChunksWhileFetchingMininum)?
                .0)
        } else {
            return Err(MergeChunkError::EmptyChunksWhileFetchingMininum.into());
        }
    }

    fn write_output(&mut self) -> Result<()> {
        self.iter_count += 1;
        self.item_count += self.output.len();
        if !self.output.is_empty() {
            let bin_data = serialize(&self.output)?;
            self.output_file.write_all(&bin_data)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct MergeChunk {
    id: u32,
    file: File, // TODO don't keep the file open to prevent "Too many open files" error
    content: VecDeque<PlotEntry>,
    entry_size: usize,
    total_size: usize,
    chunk_size: usize,
    remaining_size: usize,
}

impl MergeChunk {
    pub fn refill(&mut self) -> Result<()> {
        if self.content.len() == 0 && self.remaining_size > 0 {
            let amount;
            let mut buffer;
            if self.remaining_size > self.chunk_size {
                // Read only 1 chunk
                buffer = vec![0u8; self.chunk_size];
                self.file.read_exact(&mut buffer)?;
                amount = self.chunk_size;
            } else {
                // Read to the end
                buffer = Vec::new();
                amount = self.file.read_to_end(&mut buffer)?;
            }

            self.remaining_size -= amount;

            // Deserilalize entries
            let entries = deserialize(&buffer, self.entry_size)?;
            self.content = VecDeque::from(entries);
        }

        Ok(())
    }

    pub fn is_done(&self) -> bool {
        return self.content.len() == 0 && self.remaining_size == 0;
    }
}
