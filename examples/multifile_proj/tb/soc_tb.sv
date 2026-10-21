`include "opcodes.vh"
module soc_tb;
    logic clk, rst_n, we;
    logic [3:0]  waddr;
    logic [15:0] wdata, result;
    logic [2:0]  op;
    soc_top dut(.clk(clk), .rst_n(rst_n), .we(we), .waddr(waddr), .wdata(wdata), .op(op), .result(result));
    initial begin clk = 0; forever #5 clk = !clk; end
    initial begin
        rst_n = 0; we = 0; waddr = 4'd0; wdata = 16'd0; op = OP_ADD;
        #12 rst_n = 1;
        #3 we = 1; waddr = 4'd5; wdata = 16'd300;   // single write to mem[5]
        #10 we = 0; wdata = 16'd155;                   // deassert write-enable
        #10 op = OP_SUB; wdata = 16'd100;             // expect: 300 - 100 = 200
        #10 $display("result=%0d (expect 200: 300-100)", result);
        $finish;
    end
endmodule
