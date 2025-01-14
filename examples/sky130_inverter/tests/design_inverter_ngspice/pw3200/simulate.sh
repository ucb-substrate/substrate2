#!/bin/bash

set -x



set -e

ngspice \
  -b -r /Users/rahul/work/substrate2/examples/sky130_inverter/tests/design_inverter_ngspice/pw3200/data.raw \
   \
  /Users/rahul/work/substrate2/examples/sky130_inverter/tests/design_inverter_ngspice/pw3200/netlist.spice \
  > /Users/rahul/work/substrate2/examples/sky130_inverter/tests/design_inverter_ngspice/pw3200/ngspice.log \
  2> /Users/rahul/work/substrate2/examples/sky130_inverter/tests/design_inverter_ngspice/pw3200/ngspice.err
