`include "nova_pkg.svh"

// =====================================================================
// NOVA data RAM — 128 x 16, synchronous write / combinational read
// =====================================================================
module nova_ram (
    input  logic                  clk,
    input  logic                  we,
    input  logic [6:0]            waddr,
    input  logic [NOVA_DATA_W-1:0] wdata,
    input  logic [6:0]            raddr,
    output logic [NOVA_DATA_W-1:0] rdata
);
    logic [NOVA_DATA_W-1:0] mem [0:NOVA_RAM_WORDS-1];

    always_ff @(posedge clk) begin
        if (we) mem[waddr] <= wdata;
    end

    assign rdata = mem[raddr];
endmodule
