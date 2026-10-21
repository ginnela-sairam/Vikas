// VIKAS demo 2 — pipelined ALU with forwarding hazard + glitch cone.
module alu_slice(
    input  logic [2:0]  op,
    input  logic [15:0] a, b,
    output logic [15:0] result
);
    always_comb begin
        case (op)
            3'd0: result = a + b;
            3'd1: result = a - b;
            3'd2: result = a ^ b;
            3'd3: result = a & b;
            3'd4: result = a | b;
            3'd5: result = a << 1;
            3'd6: result = {a[14:0], 1'b0};
            default: result = ~a;
        endcase
    end
endmodule

module alu_pipeline(
    input  logic        clk, rst_n,
    input  logic        valid_in,
    input  logic [2:0]  op,
    input  logic [15:0] a, b,
    output logic        valid_ex,
    output logic [15:0] alu_result,
    output logic [15:0] partial_sum
);
    // stage registers
    logic [2:0]  op_r;
    logic [15:0] a_r, b_r;
    logic        valid_r;
    // deep combinational cone (FO4 depth demo + glitch hazard)
    logic [15:0] s0, s1, s2, s3, s4;
    assign s0 = a_r + b_r;
    assign s1 = s0 ^ a_r;
    assign s2 = s1 & b_r;
    assign s3 = s2 | a_r;
    assign s4 = s3 ^ s0;
    assign partial_sum = s4;
    always_ff @(posedge clk) begin
        if (!rst_n) begin
            op_r <= 3'd0; a_r <= 16'd0; b_r <= 16'd0; valid_r <= 1'b0;
        end else begin
            op_r <= op; a_r <= a; b_r <= b; valid_r <= valid_in;
        end
    end
    logic [15:0] ex_result;
    alu_slice u_alu(.op(op_r), .a(a_r), .b(b_r), .result(ex_result));
    always_ff @(posedge clk) begin
        if (!rst_n) begin
            valid_ex   <= 1'b0;
            alu_result <= 16'd0;
        end else begin
            valid_ex   <= valid_r;
            alu_result <= valid_r ? ex_result : 16'hDEAD;
        end
    end
endmodule

module alu_tb;
    logic clk, rst_n, valid_in, valid_ex;
    logic [2:0]  op;
    logic [15:0] a, b, alu_result, partial_sum;
    alu_pipeline dut(.clk(clk), .rst_n(rst_n), .valid_in(valid_in), .op(op),
                      .a(a), .b(b), .valid_ex(valid_ex),
                      .alu_result(alu_result), .partial_sum(partial_sum));
    // clock
    initial begin clk = 0; forever #5 clk = !clk; end
    initial begin
        rst_n = 0; valid_in = 0; op = 3'd0; a = 16'd0; b = 16'd0;
        #12 rst_n = 1;
        // ADD 100 + 55
        #3 valid_in = 1; op = 3'd0; a = 16'd100; b = 16'd55;
        #10 op = 3'd1; a = 16'd200; b = 16'd155;   // SUB
        #10 op = 3'd2; a = 16'hAAAA; b = 16'h5555; // XOR
        #10 valid_in = 0;
        #20 $display("alu_result=%0d valid_ex=%0b partial_sum=%0d", alu_result, valid_ex, partial_sum);
        $finish;
    end
endmodule
