hyperfine -w 3 -i --export-markdown "bench.md" `
  ".\target\release\spaceframe-node.exe init -k 14" `
  ".\target\release\spaceframe-node.exe init -k 15" `
  ".\target\release\spaceframe-node.exe init -k 16" `
  ".\target\release\spaceframe-node.exe init -k 17" `
  ".\target\release\spaceframe-node.exe init -k 18" `
  ".\target\release\spaceframe-node.exe init -k 19" `
  ".\target\release\spaceframe-node.exe init -k 20"

hyperfine -w 2 -i --export-markdown "bench2.md" `
  ".\target\release\spaceframe-node.exe init -k 21" `
  ".\target\release\spaceframe-node.exe init -k 22"

hyperfine --export-csv "bench3.csv" `
  ".\target\release\spaceframe-node.exe match -k 14" `
  ".\target\release\spaceframe-node.exe match -k 15" `
  ".\target\release\spaceframe-node.exe match -k 16" `
  ".\target\release\spaceframe-node.exe match -k 17" `
  ".\target\release\spaceframe-node.exe match -k 18" `
  ".\target\release\spaceframe-node.exe match -k 19" `
  ".\target\release\spaceframe-node.exe match -k 20" `
  ".\target\release\spaceframe-node.exe match -k 21" `
  ".\target\release\spaceframe-node.exe match -k 22" `
  ".\target\release\spaceframe-node.exe match -k 23" `
  ".\target\release\spaceframe-node.exe match -k 24" `
  ".\target\release\spaceframe-node.exe match -k 25" `
  ".\target\release\spaceframe-node.exe match -k 26"

hyperfine --export-csv "bench4.csv" `
  ".\target\release\spaceframe-node.exe match -n -k 14" `
  ".\target\release\spaceframe-node.exe match -n -k 15" `
  ".\target\release\spaceframe-node.exe match -n -k 16" `
  ".\target\release\spaceframe-node.exe match -n -k 17" `
  ".\target\release\spaceframe-node.exe match -n -k 18"