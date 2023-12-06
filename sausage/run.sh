#!/bin/bash

for file in ./puzzles/*; do
    base_name=$(basename "$file")
    cargo run --release "$file" > "./solutions/${base_name%.*}.txt"
done
