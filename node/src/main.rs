use std::fs::create_dir_all;
use std::path::Path;

use anyhow::Context;
use anyhow::Result;
use console::style;
use console::truncate_str;
use console::Emoji;
use dialoguer::console::Term;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Input;
use dialoguer::Select;
use log::*;
use rand::Rng;
use rand::{rngs::OsRng, RngCore};
use simplelog::{ColorChoice, Config, LevelFilter, TermLogger, TerminalMode};
use spaceframe_crypto::ed25519::Ed25519KeyPair;
use spaceframe_crypto::traits::Keypair;
use spaceframe_ledger::account::Address;
use spaceframe_ledger::error::BlockError;
use spaceframe_ledger::ledger::Ledger;
use spaceframe_ledger::transaction::Tx;
use spaceframe_pospace::constants::PARAM_BC;
use spaceframe_pospace::constants::PARAM_EXT;
use spaceframe_pospace::core::PoSpace;
use spaceframe_pospace::fx_calculator::matching_naive;
use spaceframe_pospace::fx_calculator::FxCalculator;
use spaceframe_pospace::fx_calculator::Match;
use spaceframe_pospace::proofs::Proof;
use spaceframe_pospace::proofs::Prover;
use spaceframe_pospace::storage::PlotEntry;
use spaceframe_pospace::verifier::Verifier;
use spaceframe_storage::keypair::read_all_keypair;
use spaceframe_storage::keypair::store_keypair;
use spaceframe_storage::ledger::read_from_disk;
use spaceframe_storage::ledger::write_to_disk;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "spaceframe-node", author = "Gil Balsiger")]
struct Opts {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt)]
enum Command {
    /// Initialize the proofs of space
    Init {
        #[structopt(short = "k")]
        space: usize,
    },

    /// Used for benchmarking the proof of space proving
    Prove {
        #[structopt(short = "k")]
        space: usize,
    },

    /// Used for benchmarking the proof of space verifing
    Verify {
        #[structopt(short = "k")]
        space: usize,
    },

    /// Used for benchmarking the proof of space matching functions
    Match {
        #[structopt(short = "k")]
        k: usize,

        /// Use naive matching function
        #[structopt(short = "n")]
        naive: bool,
    },

    /// Interact with a demo blockchain in the console
    Demo {
        #[structopt(short = "k")]
        k: usize,
    },
}

const fn get_challenge(k: usize) -> [u8; 32] {
    match k {
        17 => [
            95, 23, 106, 107, 81, 99, 43, 112, 157, 49, 246, 48, 199, 114, 163, 190, 160, 165, 251,
            13, 92, 80, 240, 210, 241, 247, 44, 74, 94, 126, 245, 226,
        ],
        19 => [
            140, 31, 177, 106, 121, 35, 250, 68, 109, 103, 251, 149, 126, 201, 224, 230, 37, 74,
            247, 24, 146, 131, 28, 74, 17, 105, 126, 93, 105, 34, 222, 152,
        ],
        20 => [
            8, 69, 62, 233, 63, 175, 0, 92, 104, 211, 47, 131, 61, 52, 7, 0, 19, 150, 63, 103, 88,
            212, 133, 181, 140, 197, 12, 27, 33, 249, 33, 196,
        ],
        21 => [
            154, 156, 38, 140, 105, 6, 177, 113, 168, 152, 154, 83, 173, 244, 200, 201, 218, 49,
            102, 110, 98, 200, 99, 103, 187, 151, 182, 107, 149, 19, 244, 32,
        ],
        22 => [
            95, 173, 51, 119, 88, 121, 172, 0, 127, 130, 7, 43, 153, 16, 149, 83, 31, 188, 77, 86,
            226, 139, 33, 67, 232, 168, 112, 191, 24, 57, 130, 138,
        ],
        23 => [
            140, 68, 99, 54, 232, 127, 7, 76, 16, 142, 107, 249, 168, 126, 107, 139, 8, 163, 255,
            57, 54, 221, 139, 159, 175, 120, 246, 228, 68, 141, 187, 12,
        ],
        24 => [
            144, 123, 11, 221, 104, 85, 239, 89, 29, 145, 216, 197, 80, 113, 141, 53, 10, 17, 88,
            158, 221, 19, 230, 239, 110, 219, 195, 31, 174, 78, 214, 191,
        ],
        25 => [
            144, 152, 10, 181, 73, 130, 35, 247, 217, 187, 27, 183, 205, 119, 253, 176, 144, 123,
            122, 44, 23, 12, 87, 203, 241, 128, 6, 210, 32, 84, 254, 181,
        ],
        26 => [
            29, 248, 68, 61, 42, 102, 28, 207, 224, 69, 99, 250, 122, 119, 0, 212, 242, 153, 185,
            228, 86, 21, 149, 176, 59, 228, 126, 207, 63, 231, 66, 205,
        ],
        _ => [0u8; 32],
    }
}

fn get_proof(k: usize) -> Proof {
    let plot_seed = *b"aaaabbbbccccddddaaaabbbbccccdddd";
    match k {
        17 => Proof {
            k,
            plot_seed,
            challenge: vec![
                95, 23, 106, 107, 81, 99, 43, 112, 157, 49, 246, 48, 199, 114, 163, 190, 160, 165,
                251, 13, 92, 80, 240, 210, 241, 247, 44, 74, 94, 126, 245, 226,
            ],
            x_values: vec![
                46672, 123945, 25137, 96153, 44075, 57751, 43472, 53529, 118276, 26778, 54551,
                55527, 78723, 98898, 37295, 73748, 37594, 122322, 29862, 39783, 78833, 90645,
                51199, 16622, 107778, 16953, 124903, 85816, 69558, 119363, 68128, 85232, 116016,
                73508, 125878, 45672, 115363, 5140, 73432, 94498, 85600, 21806, 36775, 113527,
                104277, 109103, 25439, 35189, 66920, 101463, 16110, 49325, 83135, 73827, 25196,
                93703, 77659, 128937, 7216, 54000, 111494, 73524, 104245, 58276,
            ],
        },
        19 => Proof {
            k,
            plot_seed,
            challenge: vec![
                140, 31, 177, 106, 121, 35, 250, 68, 109, 103, 251, 149, 126, 201, 224, 230, 37,
                74, 247, 24, 146, 131, 28, 74, 17, 105, 126, 93, 105, 34, 222, 152,
            ],
            x_values: vec![
                326828, 226837, 467171, 19217, 189859, 503228, 137074, 152964, 179964, 476292,
                54147, 35682, 485343, 456105, 252920, 2385, 494048, 407697, 49682, 426297, 147264,
                349193, 163479, 251801, 317898, 159218, 144749, 60391, 397922, 393707, 152431,
                338751, 251743, 380638, 470257, 492738, 4688, 119647, 418527, 424192, 415192,
                219460, 32965, 149370, 371741, 139462, 217438, 403000, 273374, 427204, 329506,
                174100, 436861, 49923, 208038, 285216, 313296, 126968, 417137, 73340, 422042,
                470958, 502497, 239070,
            ],
        },
        20 => Proof {
            k,
            plot_seed,
            challenge: vec![
                8, 69, 62, 233, 63, 175, 0, 92, 104, 211, 47, 131, 61, 52, 7, 0, 19, 150, 63, 103,
                88, 212, 133, 181, 140, 197, 12, 27, 33, 249, 33, 196,
            ],
            x_values: vec![
                477931, 578035, 153607, 334295, 663377, 837112, 46451, 594748, 519771, 918984,
                588923, 590486, 69605, 154991, 152568, 322641, 301115, 493745, 789188, 871258,
                748448, 69350, 504405, 434764, 646851, 816049, 565706, 350365, 96069, 677778,
                491897, 51878, 83945, 289424, 263292, 872078, 908113, 157692, 320950, 655577,
                314201, 641305, 128850, 317680, 49843, 789407, 776607, 934930, 659921, 145260,
                796901, 78276, 1010292, 92290, 379240, 663933, 305439, 585537, 529340, 68460,
                313166, 633025, 6583, 382909,
            ],
        },
        22 => Proof {
            k,
            plot_seed,
            challenge: vec![
                95, 173, 51, 119, 88, 121, 172, 0, 127, 130, 7, 43, 153, 16, 149, 83, 31, 188, 77,
                86, 226, 139, 33, 67, 232, 168, 112, 191, 24, 57, 130, 138,
            ],
            x_values: vec![
                2206732, 1361892, 1111147, 2927325, 3187419, 1505932, 1665666, 2230025, 795391,
                1040601, 1540975, 960787, 2309970, 389690, 3478858, 1351343, 1636294, 4063128,
                955379, 3398243, 1785847, 1141308, 497087, 2984069, 3110479, 429643, 576518,
                407472, 626260, 3625927, 335514, 1453218, 2257503, 80774, 2782590, 687786, 391252,
                2687194, 2176126, 362765, 1776509, 2411109, 4048999, 2707198, 914737, 1264286,
                1508332, 888040, 3620073, 3094649, 889244, 2738166, 1111887, 3381587, 2135222,
                2591541, 1507552, 2266875, 1807565, 3783448, 3227488, 800092, 1110895, 547108,
            ],
        },
        23 => Proof {
            k,
            plot_seed,
            challenge: vec![
                140, 68, 99, 54, 232, 127, 7, 76, 16, 142, 107, 249, 168, 126, 107, 139, 8, 163,
                255, 57, 54, 221, 139, 159, 175, 120, 246, 228, 68, 141, 187, 12,
            ],
            x_values: vec![
                6455406, 8097893, 2452812, 1898921, 3100449, 7945133, 5348532, 1919633, 1960876,
                6702738, 5197967, 2702565, 3252791, 4205992, 2890007, 6417898, 7485372, 578228,
                4779214, 2143205, 1820487, 6487608, 3183976, 4013145, 6326730, 6841373, 1794984,
                3828663, 3184250, 2179181, 2630247, 7044644, 8119090, 538928, 7659977, 7580882,
                7300833, 7631942, 4209684, 4885359, 4018345, 759714, 7461405, 1484874, 7889297,
                2510903, 2535787, 7405421, 6117598, 7492140, 6296865, 3681233, 3706547, 7141159,
                1492674, 4356366, 5841654, 7401142, 3995816, 2138375, 7253529, 4704463, 4308537,
                672352,
            ],
        },
        24 => Proof {
            k,
            plot_seed,
            challenge: vec![
                144, 123, 11, 221, 104, 85, 239, 89, 29, 145, 216, 197, 80, 113, 141, 53, 10, 17,
                88, 158, 221, 19, 230, 239, 110, 219, 195, 31, 174, 78, 214, 191,
            ],
            x_values: vec![
                16214919, 3604170, 3286943, 12072286, 6295911, 9298125, 11610882, 14152234,
                7717550, 12865557, 8040681, 16603749, 15839384, 14263502, 11870180, 1250164,
                12193961, 14526238, 14476708, 13420636, 956816, 4891839, 9991983, 9675947,
                13805569, 6514213, 3204948, 3107130, 11872173, 3536497, 10650436, 7813634, 9557272,
                12634849, 1357113, 1908689, 1895476, 7953062, 4473611, 8893278, 4656652, 8164283,
                7280613, 155232, 3635812, 14465446, 3654498, 7124666, 13213784, 16001490, 4747268,
                14534318, 5439723, 10101096, 1117101, 10941456, 745298, 14275134, 13996681,
                14980356, 13244641, 5194908, 4844487, 5399705,
            ],
        },
        25 => Proof {
            k,
            plot_seed,
            challenge: vec![
                144, 152, 10, 181, 73, 130, 35, 247, 217, 187, 27, 183, 205, 119, 253, 176, 144,
                123, 122, 44, 23, 12, 87, 203, 241, 128, 6, 210, 32, 84, 254, 181,
            ],
            x_values: vec![
                15572556, 20559682, 29906792, 1120869, 9552541, 14929477, 30516153, 4575415,
                9228232, 21275747, 18797224, 27675734, 28043894, 28369314, 19324126, 20870736,
                12060547, 2234124, 20330081, 23806291, 22991617, 3544089, 2102017, 1485451,
                7611391, 30902337, 18469245, 23202534, 30085282, 13496416, 14706732, 22560214,
                8540692, 3728930, 10003405, 16637866, 12332892, 7594630, 7526939, 13376368,
                8629751, 18290949, 10876109, 30008281, 27787132, 15317838, 17958357, 23782375,
                22085780, 28167331, 4286036, 4734404, 3789708, 6328815, 23542672, 19168715,
                2205712, 33344495, 27169606, 17169536, 31617452, 3321761, 24597399, 10610386,
            ],
        },
        _ => Proof {
            k,
            x_values: Vec::new(),
            challenge: Vec::new(),
            plot_seed: [0; 32],
        },
    }
}

fn main() -> Result<()> {
    let opt = Opts::from_args();

    TermLogger::init(
        LevelFilter::Trace,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();

    match opt.cmd {
        Command::Init { space } => {
            let mut plot_seed = [0u8; 32];
            OsRng.fill_bytes(&mut plot_seed);
            info!("Plot seed generated");
            let pos = PoSpace::new(space, *b"aaaabbbbccccddddaaaabbbbccccdddd", "data".as_ref())
                .context("Failed to create proof of space instance")?;
            pos.run_phase_1()
                .context("Failed to run phase 1 of plotting")
        }
        Command::Prove { space } => {
            let challenge = get_challenge(space);
            // OsRng.fill_bytes(&mut challenge);
            let pos = PoSpace::new(space, *b"aaaabbbbccccddddaaaabbbbccccdddd", "data".as_ref())
                .context("Failed to create proof of space instance")?;
            let prover = Prover::new(pos);
            // prover.get_quality_string(challenge.as_ref());
            prover
                .retrieve_all_proofs(challenge.as_ref())
                .context("Cannot retrieve all proofs for challange")?;

            Ok(())
        }
        Command::Match { k, naive } => {
            let mut data = Vec::new();
            for _ in 0..(1 << k) {
                let mut rng = rand::thread_rng();
                data.push(PlotEntry {
                    fx: rng.gen_range(0..(1 << (k + PARAM_EXT))),
                    metadata: None,
                    position: None,
                    offset: None,
                })
            }
            data.sort_unstable();

            let mut matches = Vec::new();

            if naive {
                let mut left_bucket = Vec::new();
                let mut right_bucket = Vec::new();
                let mut bucket = 0;

                for entry in data {
                    let y_bucket = entry.fx / PARAM_BC;
                    if y_bucket == bucket {
                        left_bucket.push(entry);
                    } else if y_bucket == bucket + 1 {
                        right_bucket.push(entry);
                    } else {
                        if !left_bucket.is_empty() && !right_bucket.is_empty() {
                            'mainloop: for i in 0..left_bucket.len() {
                                for j in 0..right_bucket.len() {
                                    if i != j {
                                        if matching_naive(left_bucket[i].fx, right_bucket[j].fx) {
                                            matches.push(Match {
                                                left_index: i,
                                                right_index: j,
                                            });
                                        }

                                        if matches.len() >= (1 << k) {
                                            break 'mainloop;
                                        }
                                    }
                                }
                            }
                        }

                        if matches.len() >= (1 << k) {
                            break;
                        }

                        if y_bucket == bucket + 2 {
                            bucket += 1;
                            left_bucket = right_bucket.clone();
                            right_bucket.clear();
                            right_bucket.push(entry);
                        } else {
                            bucket = y_bucket;
                            left_bucket.clear();
                            left_bucket.push(entry);
                            right_bucket.clear();
                        }
                    }
                }
            } else {
                let mut left_bucket = Vec::new();
                let mut right_bucket = Vec::new();
                let mut bucket = 0;
                let mut fx = FxCalculator::new(k, 2);

                for entry in data {
                    let y_bucket = entry.fx / PARAM_BC;
                    if y_bucket == bucket {
                        left_bucket.push(entry);
                    } else if y_bucket == bucket + 1 {
                        right_bucket.push(entry);
                    } else {
                        if !left_bucket.is_empty() && !right_bucket.is_empty() {
                            matches.extend(fx.find_matches(&left_bucket, &right_bucket));
                        }

                        if matches.len() >= (1 << k) {
                            break;
                        }

                        if y_bucket == bucket + 2 {
                            bucket += 1;
                            left_bucket = right_bucket.clone();
                            right_bucket.clear();
                            right_bucket.push(entry);
                        } else {
                            bucket = y_bucket;
                            left_bucket.clear();
                            left_bucket.push(entry);
                            right_bucket.clear();
                        }
                    }
                }
            }

            info!("{} matches found", matches.len());

            Ok(())
        }
        Command::Verify { space } => {
            let verifier = Verifier::new();
            let proof = get_proof(space);
            verifier
                .verify_proof(&proof)
                .context("Could not verify the proof")
        }
        Command::Demo { k } => {
            const INITIAL_KEYPAIRS: usize = 3;
            const INITIAL_AMOUNT: u64 = 100;

            let chain_path = Path::new("blockchain_data");
            let keypairs_path = Path::new("keypair_data");

            let pospace = PoSpace::new(k, *b"aaaabbbbccccddddaaaabbbbccccdddd", "data".as_ref())?;
            let prover = Prover::new(pospace);

            let mut keypairs = match read_all_keypair(keypairs_path) {
                Ok(keypairs) => keypairs,
                Err(_) => {
                    create_dir_all(keypairs_path).ok();
                    Vec::new()
                }
            };

            let mut ledger = match read_from_disk(chain_path) {
                Ok(ledger) => {
                    if ledger.blockchain.len() == 0 {
                        info!("No existing ledger found. Creating a new one.");
                        create_dir_all(chain_path)?;
                        if keypairs.len() == 0 {
                            info!(
                                "No account found. Creating {} new accounts with {} SF.",
                                INITIAL_KEYPAIRS, INITIAL_AMOUNT
                            );
                            for _ in 0..INITIAL_KEYPAIRS {
                                let keypair = Ed25519KeyPair::generate();
                                store_keypair(&keypair, keypairs_path)?;
                                keypairs.push(keypair);
                            }
                        }
                        let ledger = Ledger::new(
                            &keypairs
                                .iter()
                                .map(|k| Tx::genesis(&Address::from(k.public), INITIAL_AMOUNT))
                                .collect::<Vec<Tx>>(),
                        )?;
                        write_to_disk(&ledger, chain_path)?;
                        ledger
                    } else {
                        ledger
                    }
                }
                Err(e) => {
                    return Err(e.context("Failed to read the blockchain from disk"));
                }
            };

            loop {
                let selection = Select::with_theme(&ColorfulTheme::default())
                    .items(&[
                        "Add keypair",
                        "Add new block",
                        "Show account balances",
                        "Show blocks",
                        "Exit",
                    ])
                    .default(0)
                    .interact_on(&Term::stderr())?;

                match selection {
                    0 => {
                        let keypair = Ed25519KeyPair::generate();
                        store_keypair(&keypair, keypairs_path)?;
                        keypairs.push(keypair);
                        info!("New keypair generated and stored");
                    }
                    1 => {
                        let mut transactions_buffer = Vec::new();
                        'new_block_loop: loop {
                            if transactions_buffer.len() > 0 {
                                println!("Transactions to add :");
                                for tx in &transactions_buffer {
                                    println!("{}", tx);
                                }
                            }

                            let selection = Select::with_theme(&ColorfulTheme::default())
                                .items(&[
                                    "Add new transaction",
                                    "Prove block and add to the blockchain",
                                    "Cancel",
                                ])
                                .default(0)
                                .interact_on(&Term::stderr())?;
                            match selection {
                                0 => {
                                    let sender_index =
                                        Select::with_theme(&ColorfulTheme::default())
                                            .items(
                                                &keypairs
                                                    .iter()
                                                    .map(|k| Address::from(k.public).to_string())
                                                    .collect::<Vec<String>>(),
                                            )
                                            .default(0)
                                            .with_prompt("Choose the sender")
                                            .interact_on(&Term::stderr())?;
                                    let receiver_index =
                                        Select::with_theme(&ColorfulTheme::default())
                                            .items(
                                                &keypairs
                                                    .iter()
                                                    .map(|k| Address::from(k.public).to_string())
                                                    .collect::<Vec<String>>(),
                                            )
                                            .default(0)
                                            .with_prompt("Choose the receiver")
                                            .interact_on(&Term::stderr())?;
                                    let amount = Input::new()
                                        .with_prompt("Choose the amount")
                                        .validate_with(
                                            |input: &String| -> core::result::Result<(), &str> {
                                                return match input.parse::<u64>() {
                                                    Ok(_) => Ok(()),
                                                    Err(_) => Err("Please enter a number"),
                                                };
                                            },
                                        )
                                        .interact_text()?;
                                    let amount = amount.parse()?;
                                    match Tx::new(
                                        &keypairs[sender_index],
                                        &Address::from(keypairs[receiver_index].public),
                                        amount,
                                        0,
                                    ) {
                                        Ok(tx) => transactions_buffer.push(tx),
                                        Err(e) => error!("Could not create transaction: {}", e),
                                    };
                                }
                                1 => loop {
                                    match ledger.add_block_from_transactions_and_prove(
                                        &transactions_buffer,
                                        &prover,
                                    ) {
                                        Ok(()) => {
                                            info!("Block successfully added to the ledger");
                                            write_to_disk(&ledger, chain_path)?;
                                            break 'new_block_loop;
                                        }
                                        Err(e) => match e.downcast_ref::<BlockError>() {
                                            Some(BlockError::NoProofFound) => {
                                                warn!("No proof found. Trying again ...");
                                                continue;
                                            }
                                            _ => {
                                                error!("{}", e);
                                                break;
                                            }
                                        },
                                    }
                                },
                                2 => break,
                                _ => return Err(anyhow::anyhow!("Invalid option")),
                            }
                        }
                    }
                    2 => {
                        println!("\nAccounts:");
                        for kp in &keypairs {
                            let address = Address::from(kp.public);
                            let balance = ledger.get_balance(&address)?;
                            println!("{}: {} SF", address, balance);
                        }
                    }
                    3 => {
                        println!("");
                        let verifier = Verifier::new();
                        for block in &ledger.blockchain {
                            let is_proof_valid = block
                                .proof
                                .as_ref()
                                .map_or(false, |p| verifier.verify_proof(&p).is_ok());
                            println!("Height: {}", block.height);
                            println!(
                                "Hash: {}",
                                truncate_str(&hex::encode(&block.hash), 20, "...")
                            );
                            block.previous_block_hash.as_ref().map(|h| {
                                println!(
                                    "Prev hash: {}",
                                    truncate_str(&hex::encode(&h), 20, "...")
                                );
                            });
                            println!(
                                "Proof: {}",
                                if is_proof_valid {
                                    style(Emoji("✔ Valid", "Valid")).green()
                                } else {
                                    if block.is_genesis() {
                                        style(Emoji("✔ None (genesis)", "None (genesis)")).green()
                                    } else {
                                        style(Emoji("✘ Not valid", "Not valid")).red()
                                    }
                                }
                            );
                            println!(
                                "Transactions: {}",
                                if block.transactions.len() == 0 {
                                    "None"
                                } else {
                                    ""
                                }
                            );
                            for tx in &block.transactions {
                                println!("  {}", tx);
                            }
                            println!("---------------------------");
                        }
                        println!("{} blocks in the chain", ledger.blockchain.len());
                        println!(
                            "Is the blockchain valid: {}",
                            if ledger.verify().is_ok() {
                                style(Emoji("✔ Valid", "Valid")).green()
                            } else {
                                style(Emoji("✘ Not valid", "Not valid")).red()
                            }
                        );
                    }
                    4 => {
                        return Ok(());
                    }
                    _ => return Err(anyhow::anyhow!("Invalid option")),
                }
            }
        }
    }
}
