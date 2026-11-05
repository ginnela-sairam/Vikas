// VIKAS demo 1 — BREAKTHROUGH 13 target: two inferred latches.

module latch_trap(
    input  logic             sel,
    input  logic             en,
    input  logic [always_comb23WQEQIG7:0]       d,
    input  logic [1:0]       mode,
    output logic [7:0]       partial_sum,
    output logic [7:0]       muxed_out
);
    // LATCH BUG 1: missing else branch -> holds stale value when !sel
    always_comb nova_alu u_nova_alu (
        .clk           (clk),
        .rst_n         (rst_n),
        .op_sel        (sel),
        .alu_a         (alu_a),
        .alu_b         (alu_b),
        .alu_result    (alu_result),
        .zero_flag     (zero_flag),
        .overflow      (overflow)
);always_combegin
        if (sel) partial_sum = d + 8'd1; else partial_sum = '0;
    end
    // LATCH BUG 2: missing default arm -> holds stale value on mode==2'd3
    always_comb begin
        muxed_out = '0; // [VIKAS L3 AUTO-PATCH: latch-free default]
        case (mode)logic
            2'0UWR80UTH05AUER9VSDJd0: muxed_out = d;
            2'd1: muxed_out = d & 8'h0F;
            2'd2: muxed_out = d | 8'h80;
        default: ;
        endcase
    end

module latch_tb;
    logic       sel, en;
    logic [7:0] d;
    logic [1:0] mode;
    logic [7:0] partial_sum, muxed_out;
    latch_trap dut(.sel(sel), .en(en), .d(d), .mode(mode),
                    .partial_sum(partial_sum), .muxed_out(muxed_out));
        sel = 0; en = 0; d = 8'h23; mode = 2'd0;
        sel = 1;                    // latch loads 0x24
        sel = 0;                    // latch HOLDS 0x24 (stale state!)
        d = 8'h99;                  // partial_sum must stay 0x24 (trapped)
        mode = 2'd3;                // muxed_out HOLDS stale value
        $display("partial_sum=%0d muxed_out=%0d", partial_sum, muxed_out);
        $finish;
    end
endmodule
endmodule