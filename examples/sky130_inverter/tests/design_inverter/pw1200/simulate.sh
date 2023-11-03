#!/bin/bash

set -x



set -e

ngspice \
  -b -r /Users/rohan/layout/substrate2/examples/sky130_inverter/tests/design_inverter/pw1200/data.raw \
   \
  /Users/rohan/layout/substrate2/examples/sky130_inverter/tests/design_inverter/pw1200/netlist.spice \
  > /Users/rohan/layout/substrate2/examples/sky130_inverter/tests/design_inverter/pw1200/ngspice.log \
  2> /Users/rohan/layout/substrate2/examples/sky130_inverter/tests/design_inverter/pw1200/ngspice.err
