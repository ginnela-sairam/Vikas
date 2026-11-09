// =====================================================================
// VIKAS v3.0 — Polyglot AI Tensor Cluster Master Manifest (.f)
// Demonstrates multi-file, multi-language elaboration across:
//   SystemVerilog (.sv/.vh) + VHDL-2019 (.vhd) + Chisel 6 (.chisel) +
//   FIRRTL (.fir) + SystemRDL 2.0 (.rdl) + IEEE 1801 UPF (.upf) + SDC (.sdc)
// =====================================================================
+incdir+inc
+define+CLUSTER_SIM=1
+define+CLUSTER_LANES=4

inc/cluster_defs.vh
rtl/noc_router.vhd
rtl/tensor_mac.chisel
rtl/dma_engine.fir
rtl/csr_map.rdl
rtl/cluster_power.upf
rtl/cluster_timing.sdc
rtl/ai_cluster_top.sv
tb/ai_cluster_tb.sv
