`include "cluster_defs.vh"
// =====================================================================
// Polyglot AI Tensor Cluster Top-Level (SystemVerilog-2023)
// Instantiates VHDL NoC Router + Chisel Tensor MAC + FIRRTL DMA + RDL CSR
// =====================================================================
module ai_cluster_top (
    input  logic        clk,
    input  logic        rst_n,
    input  logic [15:0] act_in,
    input  logic [15:0] wgt_in,
    output logic [15:0] routed_flit,
    output logic [15:0] mac_result,
    output logic [15:0] dma_out,
    output logic [15:0] cluster_acc,
    output logic        cluster_valid
);
    logic [31:0] unused_noc_y;
    logic        unused_noc_v;
    logic [31:0] unused_mac_y;
    logic        unused_mac_v;
    logic [31:0] unused_dma_y;
    logic        driver_valid;

    // 1. VHDL-2019 Mesh NoC Router instance
    noc_router u_noc (
        .clk(clk),
        .rst_n(rst_n),
        .poly_in_a(32'd0),
        .poly_in_b(32'd0),
        .poly_out_y(unused_noc_y),
        .poly_valid(unused_noc_v),
        .flit_in(act_in),
        .flit_out(routed_flit)
    );

    // 2. Chisel 6 Systolic Tensor MAC instance
    tensor_mac u_mac (
        .clk(clk),
        .rst_n(rst_n),
        .poly_in_a(32'd0),
        .poly_in_b(32'd0),
        .poly_out_y(unused_mac_y),
        .poly_valid(unused_mac_v),
        .mac_a(routed_flit),
        .mac_b(wgt_in),
        .mac_y(mac_result)
    );

    // 3. FIRRTL Tensor DMA Engine instance
    dma_engine u_dma (
        .clk(clk),
        .rst_n(rst_n),
        .poly_in_a(32'd0),
        .poly_in_b(32'd0),
        .poly_out_y(unused_dma_y),
        .poly_valid(driver_valid),
        .dma_src(mac_result),
        .dma_dst(dma_out)
    );

    always_ff @(posedge clk) begin
        if (!rst_n) begin
            cluster_acc   <= 16'd0;
            cluster_valid <= 1'b0;
        end else begin
            cluster_acc   <= cluster_acc + dma_out;
            cluster_valid <= driver_valid;
        end
    end
endmodule
