#!/bin/bash

set -x

{% if bashrc -%}
source {{ bashrc }}
{%- endif %}

set -e

ngspice \
  -b -r {{ raw_output_file }} \
  {{ flags }} \
  {{ netlist }} \
  > {{ log_path }} \
  2> {{ err_path }}
