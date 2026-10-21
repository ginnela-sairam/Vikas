// VIKAS demo 6 — SAR ADC approximation + stochastic bitstream (physics demo).
module sar_adc(
    input  logic        clk, rst_n,
    input  logic [11:0] vin,        // "analog" input (sampled)
    output logic [11:0] code,
    output logic        conv_done
);
    logic [3:0]  bit_idx;   // 12-bit SAR from MSB
    logic [11:0] approx;
    logic        active;
    always_ff @(posedge clk) begin
        if (!rst_n) begin
            bit_idx <= 4'd11; approx <= 12'd0; code <= 12'd0;
            active <= 1'b0; conv_done <= 1'b0;
        end else if (!active) begin
            bit_idx <= 4'd11; approx <= 12'd0;
            active <= 1'b1; conv_done <= 1'b0;
        end else begin
            // trial bit
            if (vin >= (approx | (12'd1 << bit_idx))) approx <= approx | (12'd1 << bit_idx);
            if (bit_idx == 0) begin
                // capture the final LSB trial decision too (SAR convergence)
                code <= (vin >= (approx | 12'd1)) ? (approx | 12'd1) : approx;
                conv_done <= 1'b1; active <= 1'b0;
            end
            bit_idx <= bit_idx - 1;
        end
    end
endmodule

module pbit_stream(
    input  logic clk, rst_n,
    output logic bit_out      // stochastic bitstream for P-bit analysis
);
    logic [15:0] lfsr;
    always_ff @(posedge clk) begin
        if (!rst_n) lfsr <= 16'hACE1;
        else lfsr <= {lfsr[14:0], lfsr[15] ^ lfsr[13] ^ lfsr[12] ^ lfsr[10]};
    end
    assign bit_out = lfsr[15] ^ lfsr[7] ^ lfsr[2];  // ~50% duty stochastic stream
endmodule

module sar_tb;
    logic clk, rst_n, conv_done, bit_out;
    logic [11:0] vin, code;
    sar_adc dut(.clk(clk), .rst_n(rst_n), .vin(vin), .code(code), .conv_done(conv_done));
    pbit_stream pb(.clk(clk), .rst_n(rst_n), .bit_out(bit_out));
    initial begin clk = 0; forever #5 clk = !clk; end
    initial begin
        rst_n = 0; vin = 12'd2999;   // target ~73% of full-scale
        #12 rst_n = 1;
        #180 $display("sar code=%0d (target 2999) conv_done=%0b", code, conv_done);
        $finish;
    end
endmodule
