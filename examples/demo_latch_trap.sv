// VIKAS demo 1 — BREAKTHROUGH 13 target: two inferred latches.
module latch_trap(
    input  logic       sel,
    input  logic       en,
    input  logic [7:0] d,
    input  logic [1:0] mode,
    output logic [7:0] partial_sum,   // latch #1: incomplete if (no else)
    output logic [7:0] muxed_out      // latch #2: incomplete case (no default)
);
    // LATCH BUG 1: missing else branch -> holds stale value when !sel
    always_comb begin
        if (sel) partial_sum = d + 8'd1;
    end
    // LATCH BUG 2: missing default arm -> holds stale value on mode==2'd3
    always_comb begin
        case (mode)
            2'd0: muxed_out = d;
            2'd1: muxed_out = d & 8'h0F;
            2'd2: muxed_out = d | 8'h80;
        endcase
    end
endmodule

module latch_tb;
    logic       sel, en;
    logic [7:0] d;
    logic [1:0] mode;
    logic [7:0] partial_sum, muxed_out;
    latch_trap dut(.sel(sel), .en(en), .d(d), .mode(mode),
                    .partial_sum(partial_sum), .muxed_out(muxed_out));
    initial begin
        sel = 0; en = 0; d = 8'h23; mode = 2'd0;
        #10 sel = 1;                    // latch loads 0x24
        #10 sel = 0;                    // latch HOLDS 0x24 (stale state!)
        #10 d = 8'h99;                  // partial_sum must stay 0x24 (trapped)
        #10 mode = 2'd3;                // muxed_out HOLDS stale value
        #10 $display("partial_sum=%0d muxed_out=%0d", partial_sum, muxed_out);
        $finish;
    end
endmodule
