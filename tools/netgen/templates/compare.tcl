lvs \
    "{{netlist1_path}} {{cell1}}" \
    "{{netlist2_path}} {{cell2}}" \
    {{setup_file_path}} \
    {{compare_results_path}} \
    -json

# record node correspondences
set nxffile [open "{{nxf_path}}" w]
{% for node in node1_mappings %}
if {[catch {set node [matching node "{{node}}"]} errmsg]} {
puts stderr "error trying to find matching node for {{node}}: $errmsg"
puts $nxffile "{{no_matching_node}}"
} else {
puts $nxffile $node
}
{% endfor %}

flush $nxffile
close $nxffile

