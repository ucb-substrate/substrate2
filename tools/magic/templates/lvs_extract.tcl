gds read {{ gds_path }}
load {{ cell_name }}

# settings
extract no capacitance
extract no resistance
extract no coupling
extract no length

ext2spice default
ext2spice format ngspice
ext2spice rthresh infinite
ext2spice cthresh infinite
ext2spice merge none
ext2spice extresist off
ext2spice resistor tee off
ext2spice subcircuit on
ext2spice subcircuit top on
ext2spice subcircuit descend on
ext2spice short none
ext2spice scale off
ext2spice lvs

# perform extraction
extract all
ext2spice -o {{ netlist_path }} {{ cell_name }}
quit
