#!/bin/bash

set -x

{% if bashrc -%}
source {{ bashrc }}
{%- endif %}

set -e

spectre \
  -format {{ format }} \
  -raw {{ raw_output_path }} \
  =log {{ log_path }} \
  {{ flags }} \
  {{ netlist }}
