#!/bin/bash

set -eufx -o pipefail

# clear old compare results
rm -f {{compare_results_path}} {{nxf_path}}

# run netgen
netgen -batch source {{tcl_path}}
