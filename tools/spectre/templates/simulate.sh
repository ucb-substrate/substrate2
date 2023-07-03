#!/bin/bash

set -x

{% if bashrc -%}
source {{ bashrc }}
{%- endif %}

set -e

spectre \
  -format {{ format }} \
  -raw {{ raw_output_dir }} \
  =log {{ log_path }} \
  {{ flags }} \
  {{ netlist }}
