lvs \
    "{{netlist1_path}} {{cell1}}" \
    "{{netlist2_path}} {{cell2}}" \
    {{setup_file_path}} \
    {{compare_results_path}} \
    -json

# record node correspondences
set nxffile [open "{{nxf_path}}" w]
foreach pair [print -list nodes legal] {
    puts "[lindex $pair 0] [lindex $pair 1]"
}
flush $nxffile
close $nxffile

set ixffile [open "{{ixf_path}}" w]
foreach pair [print -list elements legal] {
    puts "[lindex $pair 0] [lindex $pair 1]"
}
flush $ixffile
close $ixffile

