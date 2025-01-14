#!/bin/bash

set -x



set -e

ngspice \
  -b -r /Users/rahul/work/substrate2/examples/sky130_inverter/tests/design_inverter_ngspice/pw3400/data.raw \
   \
  /Users/rahul/work/substrate2/examples/sky130_inverter/tests/design_inverter_ngspice/pw3400/netlist.spice \
  > /Users/rahul/work/substrate2/examples/sky130_inverter/tests/design_inverter_ngspice/pw3400/ngspice.log \
  2> /Users/rahul/work/substrate2/examples/sky130_inverter/tests/design_inverter_ngspice/pw3400/ngspice.err
