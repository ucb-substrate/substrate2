#!/bin/bash

# Print commands that are executed
set -x

# Turn on error checking options
# Running the bashrc can result in errors,
# which we'll just ignore.
set -euf -o pipefail

# Run Pegasus DRC
PEGASUS_DRC="{{ rules_dir }}" pegasus -drc -dp 12 -license_dp_continue -control {{ runset_path }} -ui_data {{ rules_path }}
