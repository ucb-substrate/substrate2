gds read {{ gds_path }}
load {{ cell_name }}
extract all
select top cell
port makeall
ext2spice lvs
ext2spice cthresh 0.01
ext2spice rthresh 0.01
ext2spice subcircuit on
ext2spice ngspice
ext2spice {{ pex_netlist_path }}
quit
