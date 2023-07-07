#!/bin/bash

set -x



set -e

spectre \
  -format psfbin \
  -raw /Users/rahul/personal/substrate2/examples/sky130_inverter/tests/design_inverter/pw1200/psf/ \
  =log /Users/rahul/personal/substrate2/examples/sky130_inverter/tests/design_inverter/pw1200/spectre.log \
   \
  /Users/rahul/personal/substrate2/examples/sky130_inverter/tests/design_inverter/pw1200/netlist.scs
