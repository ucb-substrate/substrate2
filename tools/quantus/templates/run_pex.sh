#!/bin/bash

# Print commands that are executed
set -x

# Turn on error checking options
# Running the bashrc can result in errors,
# which we'll just ignore.
set -euf -o pipefail

# Run Quantus extraction
quantus -cmd {{ run_file_path }}
