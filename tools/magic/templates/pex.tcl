gds read {{ gds_path }}
load {{ cell_name }}

# settings
extract do all
extresist extout on
ext2sim default
ext2sim labels on
ext2sim rthresh 0
ext2sim cthresh 0
ext2sim merge none
ext2spice format ngspice
ext2spice rthresh 0
ext2spice cthresh 0
ext2spice merge none
ext2spice extresist on
ext2spice subcircuit on
ext2spice subcircuit top on
ext2spice short none
ext2spice scale off

# perform extraction
extract all
ext2spice -o {{ pex_netlist_path }} {{ cell_name }}
quit
