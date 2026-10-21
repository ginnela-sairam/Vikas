`include "defs.vh"
module regfile(
    input  logic            clk, rst_n,
    input  logic            we,
    input  logic [3:0]      waddr,
    input  logic [DATA_W-1:0] wdata,
    output logic [DATA_W-1:0] rdata
);
    logic [DATA_W-1:0] mem [0:15];
    assign rdata = mem[waddr];
    always_ff @(posedge clk) begin
        if (!rst_n) begin
            for (int i = 0; i < 16; i = i + 1) mem[i] <= 16'd0;
        end else if (we) begin
            mem[waddr] <= wdata;
        end
    end
endmodule
