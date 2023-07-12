Get-ChildItem ".\puzzles" |
Foreach-Object {
    cargo run --release $_.FullName > ".\solutions\$($_.BaseName).txt"
}
