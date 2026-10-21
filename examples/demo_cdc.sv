// VIKAS demo 4 — CDC hazard: two async clocks crossing data unsynchronized.
module cdc_demo(
    input  logic       clk_a,    // 100 MHz
    input  logic       clk_b,    // 33 MHz (async)
    input  logic       rst_n,
    input  logic [7:0] data_in,
    output logic [7:0] data_b
);
    logic [7:0] r_a, r_b;
    always_ff @(posedge clk_a) if (!rst_n) r_a <= 8'd0; else r_a <= data_in;
    // CDC BUG: single-flop, no 2-stage synchronizer
    always_ff @(posedge clk_b) if (!rst_n) r_b <= 8'd0; else r_b <= r_a;
    assign data_b = r_b;
endmodule

module cdc_tb;
    logic clk_a, clk_b, rst_n;
    logic [7:0] data_in, data_b;
    cdc_demo dut(.clk_a(clk_a), .clk_b(clk_b), .rst_n(rst_n), .data_in(data_in), .data_b(data_b));
    initial begin clk_a = 0; forever #5 clk_a = !clk_a; end
    initial begin clk_b = 0; forever #15 clk_b = !clk_b; end
    initial begin
        rst_n = 0; data_in = 8'h00;
        #12 rst_n = 1;
        #7 data_in = 8'h3C;
        #13 data_in = 8'hC3;
        #9 data_in = 8'h5A;
        #20 $display("data_b=%0h", data_b);
        $finish;
    end
endmodule
