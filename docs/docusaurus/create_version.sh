#!/bin/bash
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
(cd $SCRIPT_DIR \
    && yarn docusaurus docs:version $1 \
    && echo "{\"examples_path\": \"examples/$1\"}" > "versioned_docs/version-$1/docs-config.json")
