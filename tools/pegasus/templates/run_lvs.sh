#!/bin/bash

# Print commands that are executed
set -x

# Turn on error checking options
# Running the bashrc can result in errors,
# which we'll just ignore.
set -euf -o pipefail

# Run Calibre LVS
pegasus -lvs -dp 12 -license_dp_continue -automatch -check_schematic \
    -control {{ run_file_path }} -rc_data -ui_data {{ rules_path }}
