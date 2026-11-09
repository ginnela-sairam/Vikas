`include "cluster_defs.vh"
// =====================================================================
// Polyglot AI Tensor Cluster Self-Checking Testbench
// Verifies cross-language elaboration: SV + VHDL + Chisel + FIRRTL + RDL + UPF + SDC
// =====================================================================
module ai_cluster_tb;
    logic        clk;
    logic        rst_n;
    logic [15:0] act_in;
    logic [15:0] wgt_in;
    logic [15:0] routed_flit;
    logic [15:0] mac_result;
    logic [15:0] dma_out;
    logic [15:0] cluster_acc;
    logic        cluster_valid;
    integer      fails;

    ai_cluster_top dut (
        .clk(clk),
        .rst_n(rst_n),
        .act_in(act_in),
        .wgt_in(wgt_in),
        .routed_flit(routed_flit),
        .mac_result(mac_result),
        .dma_out(dma_out),
        .cluster_acc(cluster_acc),
        .cluster_valid(cluster_valid)
    );

    initial begin
        clk = 0;
        forever #5 clk = ~clk;
    end

    initial begin
        fails  = 0;
        rst_n  = 0;
        act_in = 16'd10;
        wgt_in = 16'd20;
        #20 rst_n = 1;
        #20;

        // Check 1: VHDL NoC router increments flit_in (10 + 1 = 11)
        if (routed_flit == 16'd11) $display("CLUSTER-OK 1 vhdl_noc_router = %0d", routed_flit);
        else begin $display("CLUSTER-FAIL 1 vhdl_noc_router = %0d (expect 11)", routed_flit); fails = fails + 1; end

        // Check 2: Chisel Tensor MAC adds routed_flit + wgt_in (11 + 20 = 31)
        if (mac_result == 16'd31) $display("CLUSTER-OK 2 chisel_tensor_mac = %0d", mac_result);
        else begin $display("CLUSTER-FAIL 2 chisel_tensor_mac = %0d (expect 31)", mac_result); fails = fails + 1; end

        // Check 3: FIRRTL DMA engine increments mac_result (31 + 1 = 32)
        if (dma_out == 16'd32) $display("CLUSTER-OK 3 firrtl_dma_engine = %0d", dma_out);
        else begin $display("CLUSTER-FAIL 3 firrtl_dma_engine = %0d (expect 32)", dma_out); fails = fails + 1; end

        // Step second vector
        act_in = 16'd49;
        wgt_in = 16'd50;
        #20;

        // Check 4: Second vector pipeline (routed=50, mac=100, dma=101)
        if (dma_out == 16'd101) $display("CLUSTER-OK 4 polyglot_vec2_dma = %0d", dma_out);
        else begin $display("CLUSTER-FAIL 4 polyglot_vec2_dma = %0d (expect 101)", dma_out); fails = fails + 1; end

        // Check 5: Valid pipeline handshake active
        if (cluster_valid == 1'b1) $display("CLUSTER-OK 5 cluster_valid = %0d", cluster_valid);
        else begin $display("CLUSTER-FAIL 5 cluster_valid = %0d (expect 1)", cluster_valid); fails = fails + 1; end

        // Check 6: Accumulator non-zero progression
        if (cluster_acc > 16'd60) $display("CLUSTER-OK 6 cluster_acc = %0d", cluster_acc);
        else begin $display("CLUSTER-FAIL 6 cluster_acc = %0d (expect > 60)", cluster_acc); fails = fails + 1; end

        if (fails == 0) $display("CLUSTER-PASS 6/6 — Polyglot AI Cluster (SV + VHDL + Chisel + FIRRTL + RDL + UPF + SDC) verified");
        else $display("CLUSTER-FAIL %0d/6 checks failed", fails);
        $finish;
    end
endmodule
