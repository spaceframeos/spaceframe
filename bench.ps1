hyperfine -w 2 -m 5 -i --export-markdown "bench.md" `
  ".\target\release\spaceframe-node.exe init -k 14" `
  ".\target\release\spaceframe-node.exe init -k 15" `
  ".\target\release\spaceframe-node.exe init -k 16" `
  ".\target\release\spaceframe-node.exe init -k 17" `
  ".\target\release\spaceframe-node.exe init -k 18" `
  ".\target\release\spaceframe-node.exe init -k 19" `
  ".\target\release\spaceframe-node.exe init -k 20"

hyperfine -m 4 -i --export-markdown "bench2.md" `
  ".\target\release\spaceframe-node.exe init -k 21" `
  ".\target\release\spaceframe-node.exe init -k 22"