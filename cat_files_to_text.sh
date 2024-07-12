#!/bin/bash

# Name of the output file
OUTPUT_FILE="all_files_content.txt"

# Clear the output file if it already exists
> $OUTPUT_FILE

# Function to process files and append their contents to the output file
process_files() {
  for file in "$1"/*.rs; do
    if [ -f "$file" ]; then
      echo "===== START OF $file =====" >> $OUTPUT_FILE
      cat "$file" >> $OUTPUT_FILE
      echo "===== END OF $file =====" >> $OUTPUT_FILE
      echo >> $OUTPUT_FILE
    fi
  done

  for dir in "$1"/*; do
    if [ -d "$dir" ]; then
      process_files "$dir"
    fi
  done
}

# Start processing from the src directory
process_files "src"

echo "All files have been processed and concatenated into $OUTPUT_FILE."