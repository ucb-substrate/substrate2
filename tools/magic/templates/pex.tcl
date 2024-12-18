gds read {{ gds_path }}
load {{ cell_name }}
extract do all
extract all
select top cell
port makeall
ext2spice format ngspice
ext2spice rthresh 0
ext2spice cthresh 0
ext2spice merge none
ext2spice extresist on
ext2spice subcircuit on
ext2spice subcircuit top on
ext2spice short none
ext2spice -o {{ pex_netlist_path }} {{ cell_name }}
quit
