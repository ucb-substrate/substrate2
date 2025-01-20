drc off
gds read {{ gds_path }}
load {{ cell_name }}

set ofile [open "{{drc_report_path}}" w]

# run DRC
select top cell
drc on
drc catchup

# write report
foreach {msg locs} [drc listall why] {
    puts $ofile $msg
    puts $ofile [llength $locs]
}
flush $ofile
close $ofile

puts "__substrate_magic_drc_complete_0"
quit
