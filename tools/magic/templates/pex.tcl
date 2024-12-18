gds read {{ gds_path }}
load {{ cell_name }}

# settings
extract do all
extresist extout on
ext2spice format ngspice
ext2spice rthresh 0
ext2spice cthresh 0
ext2spice merge none
ext2spice extresist on
ext2spice subcircuit on
ext2spice subcircuit top on
ext2spice short none

# perform extraction
select top cell
port makeall
extract all
ext2spice -o {{ pex_netlist_path }} {{ cell_name }}
extresist all
ext2spice -o {{ pex_netlist_path }} {{ cell_name }}
quit
