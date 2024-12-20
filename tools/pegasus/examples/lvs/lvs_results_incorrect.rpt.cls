##############################################################################################################
#####  
#####                                       Pegasus LVS COMPARISON
#####  
#####  Version                       :   23.10-p015
#####  NVN Run Start                 :   Mon Dec 16 13:37:47 2024
#####  ERC Summary File              :   erc.summary
#####  Extraction Report File        :   lvs_results.rpt
#####  Comparison Report File        :   /bwrcq/C/rohankumar/substrate2/tools/pegasus/build/test_run_lvs_col_inv/lvs_results.rpt.cls
#####  Top Cell                      :   test_col_inv_array  <vs>  col_inv_array
#####  
#####                                :      @     @
#####                                :       @   @ 
#####                                :        @ @  
#####  Run Result                    :     MISMATCH
#####                                :        @ @  
#####                                :       @   @ 
#####                                :      @     @
#####  
#####  Run Summary                   :   [ERROR] Connectivity Mismatches
#####                                :   [ERROR] Instance Subtype Mismatches
#####                                :   [ERROR] Parameter Mismatches
#####                                :   [ERROR] Pin Mismatches
#####                                :   [INFO]  ERC Results: Empty
#####                                :   [INFO]  Extraction Clean
#####                                :   [INFO]  Some Sections Have Been Truncated
#####  
#####  Layout Design                 :   /bwrcq/C/rohankumar/substrate2/tools/pegasus/examples/gds/test_col_inv_array.gds (gdsii)
#####  Layout File                   :   test_col_inv_array.net (cdl)
#####  Schematic File                :   /bwrcq/C/rohankumar/substrate2/tools/pegasus/build/test_run_lvs_col_inv/source.spice (cdl)
#####  Rules File                    :   /tools/commercial/skywater/sky130_cds/PDK/sky130_release_0.0.4/Sky130_LVS/sky130.lvs.pvl
#####  Pin Swap File                 :   lvs_results.rpt.cps
#####  
#####  Extraction CPU Time           :   0h 0m 1s - (1s)
#####  Extraction Exec Time          :   0h 0m 9s - (9s)
#####  Extraction Peak Memory Usage  :   173.00MB
#####  NVN CPU Time                  :   0h 0m 0s - (0s)                 
#####  NVN Exec Time                 :   0h 0m 2s - (2s)                 
#####  NVN Peak Memory Usage         :   276.20MB
#####  LVS Total CPU Time            :   0h 0m 1s - (1s)                 
#####  LVS Total Exec Time           :   0h 0m 11s - (11s)               
#####  LVS Total Peak Memory Usage   :   276.20MB
#####  
##############################################################################################################


##############################################################################################################
#
#                                            CELL MATCH STATISTICS                                            
#
##############################################################################################################

Cell Statistic Descriptions                  | Count
---------------------------------------------+----------
Cells matched                                |         0
Cells expanded                               |         0
Cells not run                                |         0
Cells which mismatch                         |         1
---------------------------------------------+----------
Total                                        |         1


Cell Statistic Detail                        | Count
---------------------------------------------+----------
Cells with parameter mismatches              |         1
Cells with mismatched instance subtypes      |         1
Cells that have been blackboxed              |         0


##############################################################################################################
#
#                                             CELL MATCH SUMMARY                                              
#
##############################################################################################################

                                    |  Initial Pins  |  Compare Pins  |           | 
Cell                                |  lay  :  sch   |  lay  :  sch   | Status    | Detail
------------------------------------+-------+--------+-------+--------+-----------+---------------------------
(test_col_inv_array, col_inv_array) |    *0 :     66 |    *0 :     66 | mismatch  | subtype errors, truncated


##############################################################################################################
#
#                                                  LVS RULES                                                  
#
##############################################################################################################

Command Line LVS Rule Values
----------------------------

    lvs_cname                               yes -cell {-automatch}

LVS Rules Given in the Rules File
---------------------------------

    layout_path                             /bwrcq/C/rohankumar/substrate2/tools/pegasus/examples/gds/test_col_inv_array.gds
    layout_format                           gdsii
    layout_primary                          test_col_inv_array
    layout_primary                          sky130_fd_sc_ls__bufinv_16
    schematic_path                          /bwrcq/C/rohankumar/substrate2/tools/pegasus/build/test_run_lvs_col_inv/source.spice
    schematic_primary                       col_inv_array
    lvs_black_box                           cap_vpp_02p4x04p6_m1m2_noshield -grey
    lvs_black_box                           cap_vpp_04p4x04p6_m1m2_noshield -grey
    lvs_black_box                           cap_vpp_04p4x04p6_m1m2m3_shieldl1 -grey
    lvs_black_box                           cap_vpp_08p6x07p8_m1m2_noshield -grey
    lvs_black_box                           cap_vpp_08p6x07p8_m1m2m3_shieldl1 -grey
    lvs_black_box                           cap_vpp_08p6x07p8_m1m2m3_shieldl1m5_floatm4 -grey
    lvs_black_box                           cap_vpp_11p5x11p7_l1m1m2_shieldpom3 -grey
    lvs_black_box                           cap_vpp_11p5x11p7_l1m1m2m3_shieldm4 -grey
    lvs_black_box                           cap_vpp_11p5x11p7_l1m1m2m3_shieldpom4 -grey
    lvs_black_box                           cap_vpp_11p5x11p7_l1m1m2m3m4_shieldm5 -grey
    lvs_black_box                           cap_vpp_11p5x11p7_l1m1m2m3m4_shieldpom5 -grey
    lvs_black_box                           cap_vpp_11p5x11p7_m1m2_noshield -grey
    lvs_black_box                           cap_vpp_11p5x11p7_m1m2m3_shieldl1 -grey
    lvs_black_box                           cap_vpp_11p5x11p7_m1m2m3m4_shieldl1m5 -grey
    lvs_black_box                           cap_vpp_11p5x11p7_m1m2m3m4_shieldm5 -grey
                                             
    lvs_cdn_flow_options                    -cdl_use_scale
    lvs_find_shorts                         yes 
    lvs_reduce_split_gates                  no  
    lvs_recognize_gates                     -none
    lvs_reduce                              yes -parallel_mos
    lvs_reduce                              yes -series_mos
    lvs_check_property                      MP L l
    lvs_check_property                      MP W w
    lvs_check_property                      MN L l
    lvs_check_property                      MN W w
    lvs_check_property                      R W W
    lvs_check_property                      R L L
    lvs_check_property                      C L L
    lvs_check_property                      C W W
    lvs_report_file                         /bwrcq/C/rohankumar/substrate2/tools/pegasus/build/test_run_lvs_col_inv/lvs_results.rpt
    lvs_report_opt                          S
    lvs_short_equivalent_nodes              yes
    lvs_generic_device                      C ( cap_vpp_02p4x04p6_m1m2_noshield ) [ PLUS = C0 MINUS = C1 BULK
                                            = B ] -source
    lvs_generic_device                      C ( cap_vpp_04p4x04p6_m1m2_noshield ) [ PLUS = C0 MINUS = C1 BULK
                                            = B ] -source
    lvs_generic_device                      C ( cap_vpp_04p4x04p6_m1m2m3_shieldl1 ) [ PLUS = C0 MINUS = C1
                                            BULK = B ] -source
    lvs_generic_device                      C ( cap_vpp_08p6x07p8_m1m2_noshield ) [ PLUS = C0 MINUS = C1 BULK
                                            = B ] -source
    lvs_generic_device                      C ( cap_vpp_08p6x07p8_m1m2m3_shieldl1 ) [ PLUS = C0 MINUS = C1
                                            BULK = B ] -source
    lvs_generic_device                      C ( cap_vpp_08p6x07p8_m1m2m3_shieldl1m5_floatm4 ) [ PLUS = C0
                                            MINUS = C1 BULK = B ] -source
    lvs_generic_device                      C ( cap_vpp_11p5x11p7_l1m1m2_shieldpom3 ) [ PLUS = C0 MINUS = C1
                                            BULK = B ] -source
    lvs_generic_device                      C ( cap_vpp_11p5x11p7_l1m1m2m3_shieldm4 ) [ PLUS = C0 MINUS = C1
                                            BULK = B ] -source
    lvs_generic_device                      C ( cap_vpp_11p5x11p7_l1m1m2m3_shieldpom4 ) [ PLUS = C0 MINUS = C1
                                            BULK = B ] -source
    lvs_generic_device                      C ( cap_vpp_11p5x11p7_l1m1m2m3m4_shieldm5 ) [ PLUS = C0 MINUS = C1
                                            BULK = B ] -source
    lvs_generic_device                      C ( cap_vpp_11p5x11p7_l1m1m2m3m4_shieldpom5 ) [ PLUS = C0 MINUS =
                                            C1 BULK = B ] -source
    lvs_generic_device                      C ( cap_vpp_11p5x11p7_m1m2_noshield ) [ PLUS = C0 MINUS = C1 BULK
                                            = B ] -source
    lvs_generic_device                      C ( cap_vpp_11p5x11p7_m1m2m3_shieldl1 ) [ PLUS = C0 MINUS = C1
                                            BULK = B ] -source
    lvs_generic_device                      C ( cap_vpp_11p5x11p7_m1m2m3m4_shieldl1m5 ) [ PLUS = C0 MINUS = C1
                                            BULK = B ] -source
    lvs_generic_device                      C ( cap_vpp_11p5x11p7_m1m2m3m4_shieldm5 ) [ PLUS = C0 MINUS = C1
                                            BULK = B ] -source
    text_depth                              -primary

Default LVS Rule Values
-----------------------

    lvs_abort                               -softchk no
    lvs_abort                               -supply_error no
    lvs_abort                               -check_device_for_property_rule no
    lvs_abort                               -missing_hcell no
    lvs_abort                               -erc_error yes
    lvs_abort                               -inconsistent_reduction yes
    layout_case_sensitive                   no
    // layout_cpf_file                       
    // schematic_cpf_file                    
    schematic_case_sensitive                no
    schematic_format                        cdl
    // hcell_file                            
    // hcell                                 
    // lvs_exclude_hcell                     
    // lvs_delete_cell                       
    lvs_push_devices                        yes
    lvs_push_devices_properties             no
    lvs_netlist                             yes -substrate_comment
    lvs_netlist                             yes -box_contents
    lvs_netlist                             yes -unnamed_box_pins
    lvs_netlist                             no -allow_inconsistent_model
    lvs_netlist                             yes -properties_comment
    // lvs_write_netlist                     
    // lvs_write_netlist_reduced             
    // lvs_delete_cell_pin                   
    // lvs_interposer                        
    lvs_spice                               yes -allow_floating_pins
    lvs_spice                               yes -allow_duplicate_subcircuit_names
    lvs_spice                               no  -conditional_ldd
    lvs_spice                               no  -prefer_pins
    lvs_spice                               no  -override_globals
    lvs_spice                               no  -redefine_param
    lvs_spice                               no  -replicate_devices
    lvs_spice                               no  -replicate_devices_enhanced
    lvs_spice                               yes -slash_is_space
    lvs_spice                               yes -keep_back_slash
    lvs_spice                               no  -allow_unquoted_strings
    lvs_spice                               no  -allow_inline_parameters
    lvs_spice                               no  -calculate_mos_area
    lvs_spice                               no  -strict_wl
    lvs_spice                               no  -cull_primitive_subcircuits
    lvs_spice                               no  -ignore_option_scale
    lvs_spice                               no  -show_neg_param_warning
    lvs_spice                               no  -scale_x_parameters
    lvs_spice                               yes -strict_subckts_order
    // lvs_spice_option                      
    lvs_reverse_wl                          no
    // lvs_spice_multiplier_name             
                                             
    lvs_compare_case                        no
    lvs_cname                               yes -pin
    lvs_cname                               yes -net
    lvs_cname                               no  -instance
    lvs_non_user_name                       -port "^[0-9]+$"
    lvs_non_user_name                       -net  "^[0-9]+$"
    // lvs_non_user_name                    -instance 
    lvs_out_of_range_exclude_zero           no
    // lvs_group_models                      
    lvs_strict_subtypes                     no
    lvs_exact_subtypes                      no
    lvs_preserve_box_ports                  no
    // lvs_cpoint                            
    // lvs_power_name                        
    // lvs_ground_name                       
    // lvs_cell_list                         
    // lvs_device_type                       
    lvs_auto_pin_swap                       yes -swap_threshold 4000000
    lvs_swap_non_cpoint_pins                yes
    // lvs_swap_pins                         
    lvs_all_cap_pins_swappable              no
    lvs_builtin_device_pin_swap             yes
    // lvs_join_nets                         
    lvs_ignore_ports                        no
    lvs_compare_port_names                  no
    lvs_global_sigs_are_ports               yes
    lvs_discard_pins                        no
    lvs_break_ambig_max                     32
    lvs_filter                              no  -bipolar
    lvs_filter                              no  -capacitors
    lvs_filter                              no  -diodes
    lvs_filter                              no  -mos
    lvs_filter                              no  -resistors
    // lvs_filter_option                     
    // lvs_filter_device                     
    lvs_reduction_priority                  -parallel
    lvs_derive_logic                        yes -sram
    lvs_derive_logic                        no  -dram
    lvs_derive_logic                        no  -rom
    // lvs_recognize_gates_tolerance         
    lvs_reduce                              yes -parallel_bipolar
    lvs_reduce                              yes -parallel_caps
    lvs_reduce                              yes -series_caps
    lvs_reduce                              yes -parallel_diodes
    lvs_reduce                              no  -series_diodes
    lvs_reduce                              yes -parallel_resistors
    lvs_reduce                              yes -series_resistors
    // lvs_reduce_device                     
    // lvs_reduce_do_not_merge               
    lvs_expand_on_device_climbing           no
    lvs_expand_unbalanced_cells             yes
    lvs_expand_cell_on_error                no
    // lvs_check_all_properties_by_name      
    // lvs_property_map                      
    // lvs_transfer_property                 
    lvs_report_max                          50 -mismatched_net_limit 100
    lvs_warn_max                            50
    lvs_inconsistent_reduction_threshold    1000000
    lvs_report_units                        yes
    lvs_keep_data                           no
    lvs_keep_source_cells                   no
    lvs_create_match_db                     no
    // lvs_map_device                        
    unit                                    -length u
    unit                                    -resistance ohm
    unit                                    -capacitance pf
    lvs_verilog_bus_map_by_position         no
    lvs_verilog_keep_backslash              no
    lvs_verilog_detect_buses                no
    lvs_do_not_normalize                    no
    input_scale                             1000
    virtual_connect                         no  -colon
    virtual_connect                         yes -semicolon_as_colon
    virtual_connect                         -depth primary
    // virtual_connect                      -name 
    virtual_connect                         no -report
    virtual_connect                         no -incremental
    lvs_preserve_parameterized_cells        no
    lvs_check_color                         no
    // lvs_schematic_color_path              
    // lvs_assign_conflict_layer             
    // lvs_conflict_layer_check             
    lvs_check_color_label                   no
    // lvs_preserve_cells                    
    port                                    -depth -primary

Effective LVS Rule Values
-------------------------

    lvs_swap_non_cpoint_pins                yes -unbound_pin_threshold 10000
    layout_primary                          test_col_inv_array
    lvs_recognize_gates                     -none


##############################################################################################################
#
#                                               CELL COMPARISON                                               
#
##############################################################################################################

                              ##################################################
                              #                                                #
                              #       ------->>> $ MISMATCH $ <<<-------       #
                              #                                                #
                              #    test_col_inv_array  <vs>  col_inv_array     #
                              #                                                #
                              #            CONNECTIVITY MISMATCHES             #
                              #                 PIN MISMATCHES                 #
                              #              PARAMETER MISMATCHES              #
                              #               SUBTYPE MISMATCHES               #
                              #                                                #
                              ##################################################
                              # * Some sections truncated                      #
                              ##################################################

+===[col_inv_array]===========================================================================================
|                                                 STATISTICS                                                  
+=============================================================================================================


                   |     Original      |      Reduced      |     Unmatched     
Cell               |   lay   :   sch   |   lay   :   sch   |   lay   :   sch
-------------------+---------+---------+---------+---------+---------+---------
MP [4 pins]        |         :         |         :         |         :         
 MP(PSHORT)        |      *0 :      32 |      *0 :      32 |       0 :        0
 MP(PFET_01V8)     |     *32 :       0 |     *32 :       0 |       0 :        0
MN [4 pins]        |         :         |         :         |         :         
 MN(NSHORT)        |      *0 :      32 |      *0 :      32 |       0 :        0
 MN(NFET_01V8)     |     *32 :       0 |     *32 :       0 |       0 :        0
-------------------+---------+---------+---------+---------+---------+---------
Total              |      64 :      64 |      64 :      64 |       0 :        0
===================+=========+=========+=========+=========+=========+=========
Pins               |         :         |      *0 :      66 |       0 :        0
Nets               |         :         |     *66 :       0 |       0 :        0
===================+=========+=========+=========+=========+=========+=========

+===[col_inv_array]===========================================================================================
|                                        MISMATCHED INSTANCE SUBTYPES                                         
+=============================================================================================================

#       | Layout Instance                            | Schematic Instance
--------+--------------------------------------------+--------------------------------
1       | X31/M0 @(154.585,0.000) MN(NFET_01V8)      | xinv_0/xMN0/M0 MN(NSHORT) 
2       | X31/M1 @(154.585,1.920) MP(PFET_01V8)      | xinv_0/xMP0/M0 MP(PSHORT) 
3       | X30/M0 @(149.735,0.000) MN(NFET_01V8)      | xinv_1/xMN0/M0 MN(NSHORT) 
4   .  .| X30/M1 @(149.735,1.920) MP(PFET_01V8)  .  .| xinv_1/xMP0/M0 MP(PSHORT) 
5       | X29/M0 @(144.585,0.000) MN(NFET_01V8)      | xinv_2/xMN0/M0 MN(NSHORT) 
6   .  .| X29/M1 @(144.585,1.920) MP(PFET_01V8)  .  .| xinv_2/xMP0/M0 MP(PSHORT) 
7       | X28/M0 @(139.735,0.000) MN(NFET_01V8)      | xinv_3/xMN0/M0 MN(NSHORT) 
8   .  .| X28/M1 @(139.735,1.920) MP(PFET_01V8)  .  .| xinv_3/xMP0/M0 MP(PSHORT) 
9       | X27/M0 @(134.585,0.000) MN(NFET_01V8)      | xinv_4/xMN0/M0 MN(NSHORT) 
10  .  .| X27/M1 @(134.585,1.920) MP(PFET_01V8)  .  .| xinv_4/xMP0/M0 MP(PSHORT) 
11      | X26/M0 @(129.735,0.000) MN(NFET_01V8)      | xinv_5/xMN0/M0 MN(NSHORT) 
12  .  .| X26/M1 @(129.735,1.920) MP(PFET_01V8)  .  .| xinv_5/xMP0/M0 MP(PSHORT) 
13      | X25/M0 @(124.585,0.000) MN(NFET_01V8)      | xinv_6/xMN0/M0 MN(NSHORT) 
14  .  .| X25/M1 @(124.585,1.920) MP(PFET_01V8)  .  .| xinv_6/xMP0/M0 MP(PSHORT) 
15      | X24/M0 @(119.735,0.000) MN(NFET_01V8)      | xinv_7/xMN0/M0 MN(NSHORT) 
16  .  .| X24/M1 @(119.735,1.920) MP(PFET_01V8)  .  .| xinv_7/xMP0/M0 MP(PSHORT) 
17      | X23/M0 @(114.585,0.000) MN(NFET_01V8)      | xinv_8/xMN0/M0 MN(NSHORT) 
18  .  .| X23/M1 @(114.585,1.920) MP(PFET_01V8)  .  .| xinv_8/xMP0/M0 MP(PSHORT) 
19      | X22/M0 @(109.735,0.000) MN(NFET_01V8)      | xinv_9/xMN0/M0 MN(NSHORT) 
20  .  .| X22/M1 @(109.735,1.920) MP(PFET_01V8)  .  .| xinv_9/xMP0/M0 MP(PSHORT) 
21      | X21/M0 @(104.585,0.000) MN(NFET_01V8)      | xinv_10/xMN0/M0 MN(NSHORT) 
22  .  .| X21/M1 @(104.585,1.920) MP(PFET_01V8)  .  .| xinv_10/xMP0/M0 MP(PSHORT) 
23      | X20/M0 @(99.735,0.000) MN(NFET_01V8)       | xinv_11/xMN0/M0 MN(NSHORT) 
24  .  .| X20/M1 @(99.735,1.920) MP(PFET_01V8)   .  .| xinv_11/xMP0/M0 MP(PSHORT) 
25      | X19/M0 @(94.585,0.000) MN(NFET_01V8)       | xinv_12/xMN0/M0 MN(NSHORT) 
26  .  .| X19/M1 @(94.585,1.920) MP(PFET_01V8)   .  .| xinv_12/xMP0/M0 MP(PSHORT) 
27      | X18/M0 @(89.735,0.000) MN(NFET_01V8)       | xinv_13/xMN0/M0 MN(NSHORT) 
28  .  .| X18/M1 @(89.735,1.920) MP(PFET_01V8)   .  .| xinv_13/xMP0/M0 MP(PSHORT) 
29      | X17/M0 @(84.585,0.000) MN(NFET_01V8)       | xinv_14/xMN0/M0 MN(NSHORT) 
30  .  .| X17/M1 @(84.585,1.920) MP(PFET_01V8)   .  .| xinv_14/xMP0/M0 MP(PSHORT) 
31      | X16/M0 @(79.735,0.000) MN(NFET_01V8)       | xinv_15/xMN0/M0 MN(NSHORT) 
32  .  .| X16/M1 @(79.735,1.920) MP(PFET_01V8)   .  .| xinv_15/xMP0/M0 MP(PSHORT) 
33      | X15/M0 @(74.585,0.000) MN(NFET_01V8)       | xinv_16/xMN0/M0 MN(NSHORT) 
34  .  .| X15/M1 @(74.585,1.920) MP(PFET_01V8)   .  .| xinv_16/xMP0/M0 MP(PSHORT) 
35      | X14/M0 @(69.735,0.000) MN(NFET_01V8)       | xinv_17/xMN0/M0 MN(NSHORT) 
36  .  .| X14/M1 @(69.735,1.920) MP(PFET_01V8)   .  .| xinv_17/xMP0/M0 MP(PSHORT) 
37      | X13/M0 @(64.585,0.000) MN(NFET_01V8)       | xinv_18/xMN0/M0 MN(NSHORT) 
38  .  .| X13/M1 @(64.585,1.920) MP(PFET_01V8)   .  .| xinv_18/xMP0/M0 MP(PSHORT) 
39      | X12/M0 @(59.735,0.000) MN(NFET_01V8)       | xinv_19/xMN0/M0 MN(NSHORT) 
40  .  .| X12/M1 @(59.735,1.920) MP(PFET_01V8)   .  .| xinv_19/xMP0/M0 MP(PSHORT) 
41      | X11/M0 @(54.585,0.000) MN(NFET_01V8)       | xinv_20/xMN0/M0 MN(NSHORT) 
42  .  .| X11/M1 @(54.585,1.920) MP(PFET_01V8)   .  .| xinv_20/xMP0/M0 MP(PSHORT) 
43      | X10/M0 @(49.735,0.000) MN(NFET_01V8)       | xinv_21/xMN0/M0 MN(NSHORT) 
44  .  .| X10/M1 @(49.735,1.920) MP(PFET_01V8)   .  .| xinv_21/xMP0/M0 MP(PSHORT) 
45      | X9/M0 @(44.585,0.000) MN(NFET_01V8)        | xinv_22/xMN0/M0 MN(NSHORT) 
46  .  .| X9/M1 @(44.585,1.920) MP(PFET_01V8) .  .  .| xinv_22/xMP0/M0 MP(PSHORT) 
47      | X8/M0 @(39.735,0.000) MN(NFET_01V8)        | xinv_23/xMN0/M0 MN(NSHORT) 
48  .  .| X8/M1 @(39.735,1.920) MP(PFET_01V8) .  .  .| xinv_23/xMP0/M0 MP(PSHORT) 
49      | X7/M0 @(34.585,0.000) MN(NFET_01V8)        | xinv_24/xMN0/M0 MN(NSHORT) 
50  .  .| X7/M1 @(34.585,1.920) MP(PFET_01V8) .  .  .| xinv_24/xMP0/M0 MP(PSHORT) 
51      | X6/M0 @(29.735,0.000) MN(NFET_01V8)        | xinv_25/xMN0/M0 MN(NSHORT) 
52  .  .| X6/M1 @(29.735,1.920) MP(PFET_01V8) .  .  .| xinv_25/xMP0/M0 MP(PSHORT) 
53      | X5/M0 @(24.585,0.000) MN(NFET_01V8)        | xinv_26/xMN0/M0 MN(NSHORT) 
54  .  .| X5/M1 @(24.585,1.920) MP(PFET_01V8) .  .  .| xinv_26/xMP0/M0 MP(PSHORT) 
55      | X4/M0 @(19.735,0.000) MN(NFET_01V8)        | xinv_27/xMN0/M0 MN(NSHORT) 
56  .  .| X4/M1 @(19.735,1.920) MP(PFET_01V8) .  .  .| xinv_27/xMP0/M0 MP(PSHORT) 
57      | X3/M0 @(14.585,0.000) MN(NFET_01V8)        | xinv_28/xMN0/M0 MN(NSHORT) 
58  .  .| X3/M1 @(14.585,1.920) MP(PFET_01V8) .  .  .| xinv_28/xMP0/M0 MP(PSHORT) 
59      | X2/M0 @(9.735,0.000) MN(NFET_01V8)         | xinv_29/xMN0/M0 MN(NSHORT) 
60  .  .| X2/M1 @(9.735,1.920) MP(PFET_01V8)  .  .  .| xinv_29/xMP0/M0 MP(PSHORT) 
61      | X1/M0 @(4.585,0.000) MN(NFET_01V8)         | xinv_30/xMN0/M0 MN(NSHORT) 
62  .  .| X1/M1 @(4.585,1.920) MP(PFET_01V8)  .  .  .| xinv_30/xMP0/M0 MP(PSHORT) 
63      | X0/M0 @(-0.265,0.000) MN(NFET_01V8)        | xinv_31/xMN0/M0 MN(NSHORT) 
64      | X0/M1 @(-0.265,1.920) MP(PFET_01V8)        | xinv_31/xMP0/M0 MP(PSHORT) 


+===[col_inv_array]===========================================================================================
|                                       UNMATCHED SCHEMATIC PIN LABELS                                        
+=============================================================================================================

Labeled Schematic Pin     | Matched Layout Pin     | Matched Layout Net
--------------------------+------------------------+-----------------------
din_31                    | ** missing pin **      | 4
din_30                    | ** missing pin **      | 7
din_29                    | ** missing pin **      | 10
din_28 .  .  .  .  .  .  .| ** missing pin **  .  .| 13
din_27                    | ** missing pin **      | 16
din_26 .  .  .  .  .  .  .| ** missing pin **  .  .| 19
din_25                    | ** missing pin **      | 22
din_24 .  .  .  .  .  .  .| ** missing pin **  .  .| 25
din_23                    | ** missing pin **      | 28
din_22 .  .  .  .  .  .  .| ** missing pin **  .  .| 31
din_21                    | ** missing pin **      | 34
din_20 .  .  .  .  .  .  .| ** missing pin **  .  .| 37
din_19                    | ** missing pin **      | 40
din_18 .  .  .  .  .  .  .| ** missing pin **  .  .| 43
din_17                    | ** missing pin **      | 46
din_16 .  .  .  .  .  .  .| ** missing pin **  .  .| 49
din_15                    | ** missing pin **      | 52
din_14 .  .  .  .  .  .  .| ** missing pin **  .  .| 55
din_13                    | ** missing pin **      | 58
din_12 .  .  .  .  .  .  .| ** missing pin **  .  .| 61
din_11                    | ** missing pin **      | 64
din_10 .  .  .  .  .  .  .| ** missing pin **  .  .| 67
din_9                     | ** missing pin **      | 70
din_8  .  .  .  .  .  .  .| ** missing pin **  .  .| 73
din_7                     | ** missing pin **      | 76
din_6  .  .  .  .  .  .  .| ** missing pin **  .  .| 79
din_5                     | ** missing pin **      | 82
din_4  .  .  .  .  .  .  .| ** missing pin **  .  .| 85
din_3                     | ** missing pin **      | 88
din_2  .  .  .  .  .  .  .| ** missing pin **  .  .| 91
din_1                     | ** missing pin **      | 94
din_0  .  .  .  .  .  .  .| ** missing pin **  .  .| 97
din_b_31                  | ** missing pin **      | 1
din_b_30  .  .  .  .  .  .| ** missing pin **  .  .| 6
din_b_29                  | ** missing pin **      | 9
din_b_28  .  .  .  .  .  .| ** missing pin **  .  .| 12
din_b_27                  | ** missing pin **      | 15
din_b_26  .  .  .  .  .  .| ** missing pin **  .  .| 18
din_b_25                  | ** missing pin **      | 21
din_b_24  .  .  .  .  .  .| ** missing pin **  .  .| 24
din_b_23                  | ** missing pin **      | 27
din_b_22  .  .  .  .  .  .| ** missing pin **  .  .| 30
din_b_21                  | ** missing pin **      | 33
din_b_20  .  .  .  .  .  .| ** missing pin **  .  .| 36
din_b_19                  | ** missing pin **      | 39
din_b_18  .  .  .  .  .  .| ** missing pin **  .  .| 42
din_b_17                  | ** missing pin **      | 45
din_b_16  .  .  .  .  .  .| ** missing pin **  .  .| 48
din_b_15                  | ** missing pin **      | 51
din_b_14  .  .  .  .  .  .| ** missing pin **  .  .| 54
din_b_13                  | ** missing pin **      | 57
din_b_12  .  .  .  .  .  .| ** missing pin **  .  .| 60
din_b_11                  | ** missing pin **      | 63
din_b_10  .  .  .  .  .  .| ** missing pin **  .  .| 66
din_b_9                   | ** missing pin **      | 69
din_b_8   .  .  .  .  .  .| ** missing pin **  .  .| 72
din_b_7                   | ** missing pin **      | 75
din_b_6   .  .  .  .  .  .| ** missing pin **  .  .| 78
din_b_5                   | ** missing pin **      | 81
din_b_4   .  .  .  .  .  .| ** missing pin **  .  .| 84
din_b_3                   | ** missing pin **      | 87
din_b_2   .  .  .  .  .  .| ** missing pin **  .  .| 90
din_b_1                   | ** missing pin **      | 93
din_b_0   .  .  .  .  .  .| ** missing pin **  .  .| 96
vdd                       | ** missing pin **      | 2
vss                       | ** missing pin **      | 3


+===[col_inv_array]===========================================================================================
|                                       MISMATCHED INSTANCE PARAMETERS                                        
+====================================================================================================truncated


Layout Instance                           | Schematic Instance             |     Parameter Error     
------------------------------------------+--------------------------------+------------------(mip 1)
X31/M0 @(154.585,0.000) MN(NFET_01V8)     | xinv_0/xMN0/M0 MN(NSHORT)      |                         
W: 1.4 u                                  | w: 1.4e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+------------------(mip 2)
X31/M1 @(154.585,1.920) MP(PFET_01V8)     | xinv_0/xMP0/M0 MP(PSHORT)      |                         
W: 2.6 u                                  | w: 2.6e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+------------------(mip 3)
X30/M0 @(149.735,0.000) MN(NFET_01V8)     | xinv_1/xMN0/M0 MN(NSHORT)      |                         
W: 1.4 u                                  | w: 1.4e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+------------------(mip 4)
X30/M1 @(149.735,1.920) MP(PFET_01V8)     | xinv_1/xMP0/M0 MP(PSHORT)      |                         
W: 2.6 u                                  | w: 2.6e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+------------------(mip 5)
X29/M0 @(144.585,0.000) MN(NFET_01V8)     | xinv_2/xMN0/M0 MN(NSHORT)      |                         
W: 1.4 u                                  | w: 1.4e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+------------------(mip 6)
X29/M1 @(144.585,1.920) MP(PFET_01V8)     | xinv_2/xMP0/M0 MP(PSHORT)      |                         
W: 2.6 u                                  | w: 2.6e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+------------------(mip 7)
X28/M0 @(139.735,0.000) MN(NFET_01V8)     | xinv_3/xMN0/M0 MN(NSHORT)      |                         
W: 1.4 u                                  | w: 1.4e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+------------------(mip 8)
X28/M1 @(139.735,1.920) MP(PFET_01V8)     | xinv_3/xMP0/M0 MP(PSHORT)      |                         
W: 2.6 u                                  | w: 2.6e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+------------------(mip 9)
X27/M0 @(134.585,0.000) MN(NFET_01V8)     | xinv_4/xMN0/M0 MN(NSHORT)      |                         
W: 1.4 u                                  | w: 1.4e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+-----------------(mip 10)
X27/M1 @(134.585,1.920) MP(PFET_01V8)     | xinv_4/xMP0/M0 MP(PSHORT)      |                         
W: 2.6 u                                  | w: 2.6e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+-----------------(mip 11)
X26/M0 @(129.735,0.000) MN(NFET_01V8)     | xinv_5/xMN0/M0 MN(NSHORT)      |                         
W: 1.4 u                                  | w: 1.4e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+-----------------(mip 12)
X26/M1 @(129.735,1.920) MP(PFET_01V8)     | xinv_5/xMP0/M0 MP(PSHORT)      |                         
W: 2.6 u                                  | w: 2.6e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+-----------------(mip 13)
X25/M0 @(124.585,0.000) MN(NFET_01V8)     | xinv_6/xMN0/M0 MN(NSHORT)      |                         
W: 1.4 u                                  | w: 1.4e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+-----------------(mip 14)
X25/M1 @(124.585,1.920) MP(PFET_01V8)     | xinv_6/xMP0/M0 MP(PSHORT)      |                         
W: 2.6 u                                  | w: 2.6e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+-----------------(mip 15)
X24/M0 @(119.735,0.000) MN(NFET_01V8)     | xinv_7/xMN0/M0 MN(NSHORT)      |                         
W: 1.4 u                                  | w: 1.4e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+-----------------(mip 16)
X24/M1 @(119.735,1.920) MP(PFET_01V8)     | xinv_7/xMP0/M0 MP(PSHORT)      |                         
W: 2.6 u                                  | w: 2.6e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+-----------------(mip 17)
X23/M0 @(114.585,0.000) MN(NFET_01V8)     | xinv_8/xMN0/M0 MN(NSHORT)      |                         
W: 1.4 u                                  | w: 1.4e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+-----------------(mip 18)
X23/M1 @(114.585,1.920) MP(PFET_01V8)     | xinv_8/xMP0/M0 MP(PSHORT)      |                         
W: 2.6 u                                  | w: 2.6e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+-----------------(mip 19)
X22/M0 @(109.735,0.000) MN(NFET_01V8)     | xinv_9/xMN0/M0 MN(NSHORT)      |                         
W: 1.4 u                                  | w: 1.4e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+-----------------(mip 20)
X22/M1 @(109.735,1.920) MP(PFET_01V8)     | xinv_9/xMP0/M0 MP(PSHORT)      |                         
W: 2.6 u                                  | w: 2.6e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+-----------------(mip 21)
X21/M0 @(104.585,0.000) MN(NFET_01V8)     | xinv_10/xMN0/M0 MN(NSHORT)     |                         
W: 1.4 u                                  | w: 1.4e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+-----------------(mip 22)
X21/M1 @(104.585,1.920) MP(PFET_01V8)     | xinv_10/xMP0/M0 MP(PSHORT)     |                         
W: 2.6 u                                  | w: 2.6e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+-----------------(mip 23)
X20/M0 @(99.735,0.000) MN(NFET_01V8)      | xinv_11/xMN0/M0 MN(NSHORT)     |                         
W: 1.4 u                                  | w: 1.4e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+-----------------(mip 24)
X20/M1 @(99.735,1.920) MP(PFET_01V8)      | xinv_11/xMP0/M0 MP(PSHORT)     |                         
W: 2.6 u                                  | w: 2.6e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+-----------------(mip 25)
X19/M0 @(94.585,0.000) MN(NFET_01V8)      | xinv_12/xMN0/M0 MN(NSHORT)     |                         
W: 1.4 u                                  | w: 1.4e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+-----------------(mip 26)
X19/M1 @(94.585,1.920) MP(PFET_01V8)      | xinv_12/xMP0/M0 MP(PSHORT)     |                         
W: 2.6 u                                  | w: 2.6e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+-----------------(mip 27)
X18/M0 @(89.735,0.000) MN(NFET_01V8)      | xinv_13/xMN0/M0 MN(NSHORT)     |                         
W: 1.4 u                                  | w: 1.4e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+-----------------(mip 28)
X18/M1 @(89.735,1.920) MP(PFET_01V8)      | xinv_13/xMP0/M0 MP(PSHORT)     |                         
W: 2.6 u                                  | w: 2.6e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+-----------------(mip 29)
X17/M0 @(84.585,0.000) MN(NFET_01V8)      | xinv_14/xMN0/M0 MN(NSHORT)     |                         
W: 1.4 u                                  | w: 1.4e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+-----------------(mip 30)
X17/M1 @(84.585,1.920) MP(PFET_01V8)      | xinv_14/xMP0/M0 MP(PSHORT)     |                         
W: 2.6 u                                  | w: 2.6e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+-----------------(mip 31)
X16/M0 @(79.735,0.000) MN(NFET_01V8)      | xinv_15/xMN0/M0 MN(NSHORT)     |                         
W: 1.4 u                                  | w: 1.4e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+-----------------(mip 32)
X16/M1 @(79.735,1.920) MP(PFET_01V8)      | xinv_15/xMP0/M0 MP(PSHORT)     |                         
W: 2.6 u                                  | w: 2.6e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+-----------------(mip 33)
X15/M0 @(74.585,0.000) MN(NFET_01V8)      | xinv_16/xMN0/M0 MN(NSHORT)     |                         
W: 1.4 u                                  | w: 1.4e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+-----------------(mip 34)
X15/M1 @(74.585,1.920) MP(PFET_01V8)      | xinv_16/xMP0/M0 MP(PSHORT)     |                         
W: 2.6 u                                  | w: 2.6e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+-----------------(mip 35)
X14/M0 @(69.735,0.000) MN(NFET_01V8)      | xinv_17/xMN0/M0 MN(NSHORT)     |                         
W: 1.4 u                                  | w: 1.4e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+-----------------(mip 36)
X14/M1 @(69.735,1.920) MP(PFET_01V8)      | xinv_17/xMP0/M0 MP(PSHORT)     |                         
W: 2.6 u                                  | w: 2.6e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+-----------------(mip 37)
X13/M0 @(64.585,0.000) MN(NFET_01V8)      | xinv_18/xMN0/M0 MN(NSHORT)     |                         
W: 1.4 u                                  | w: 1.4e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+-----------------(mip 38)
X13/M1 @(64.585,1.920) MP(PFET_01V8)      | xinv_18/xMP0/M0 MP(PSHORT)     |                         
W: 2.6 u                                  | w: 2.6e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+-----------------(mip 39)
X12/M0 @(59.735,0.000) MN(NFET_01V8)      | xinv_19/xMN0/M0 MN(NSHORT)     |                         
W: 1.4 u                                  | w: 1.4e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+-----------------(mip 40)
X12/M1 @(59.735,1.920) MP(PFET_01V8)      | xinv_19/xMP0/M0 MP(PSHORT)     |                         
W: 2.6 u                                  | w: 2.6e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+-----------------(mip 41)
X11/M0 @(54.585,0.000) MN(NFET_01V8)      | xinv_20/xMN0/M0 MN(NSHORT)     |                         
W: 1.4 u                                  | w: 1.4e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+-----------------(mip 42)
X11/M1 @(54.585,1.920) MP(PFET_01V8)      | xinv_20/xMP0/M0 MP(PSHORT)     |                         
W: 2.6 u                                  | w: 2.6e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+-----------------(mip 43)
X10/M0 @(49.735,0.000) MN(NFET_01V8)      | xinv_21/xMN0/M0 MN(NSHORT)     |                         
W: 1.4 u                                  | w: 1.4e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+-----------------(mip 44)
X10/M1 @(49.735,1.920) MP(PFET_01V8)      | xinv_21/xMP0/M0 MP(PSHORT)     |                         
W: 2.6 u                                  | w: 2.6e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+-----------------(mip 45)
X9/M0 @(44.585,0.000) MN(NFET_01V8)       | xinv_22/xMN0/M0 MN(NSHORT)     |                         
W: 1.4 u                                  | w: 1.4e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+-----------------(mip 46)
X9/M1 @(44.585,1.920) MP(PFET_01V8)       | xinv_22/xMP0/M0 MP(PSHORT)     |                         
W: 2.6 u                                  | w: 2.6e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+-----------------(mip 47)
X8/M0 @(39.735,0.000) MN(NFET_01V8)       | xinv_23/xMN0/M0 MN(NSHORT)     |                         
W: 1.4 u                                  | w: 1.4e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+-----------------(mip 48)
X8/M1 @(39.735,1.920) MP(PFET_01V8)       | xinv_23/xMP0/M0 MP(PSHORT)     |                         
W: 2.6 u                                  | w: 2.6e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+-----------------(mip 49)
X7/M0 @(34.585,0.000) MN(NFET_01V8)       | xinv_24/xMN0/M0 MN(NSHORT)     |                         
W: 1.4 u                                  | w: 1.4e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
------------------------------------------+--------------------------------+-----------------(mip 50)
X7/M1 @(34.585,1.920) MP(PFET_01V8)       | xinv_24/xMP0/M0 MP(PSHORT)     |                         
W: 2.6 u                                  | w: 2.6e+06 u                   | 99.9999%                
L: 0.15 u                                 | l: 150000 u                    | 99.9999%                
++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
++++++ Section truncated to 50 issues, adjust this using the lvs_report_max rule +++++++++++++++++++++++++++++
++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++




##############################################################################################################
#
#                                                END OF REPORT                                                
#
##############################################################################################################

