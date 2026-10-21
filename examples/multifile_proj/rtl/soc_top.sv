`include "defs.vh"
`include "opcodes.vh"
module soc_top(
    input  logic            clk, rst_n,
    input  logic            we,
    input  logic [3:0]      waddr,
    input  logic [15:0]     wdata,
    input  logic [2:0]      op,
    output logic [15:0]     result
);
    logic [DATA_W-1:0] rf_out;
    regfile u_rf(
        .clk(clk), .rst_n(rst_n),
        .we(we), .waddr(waddr), .wdata(wdata),
        .rdata(rf_out)
    );
    alu_core u_alu(
        .op(op), .a(rf_out), .b(wdata), .y(result)
    );
endmodule
