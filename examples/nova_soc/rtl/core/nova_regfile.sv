`include "nova_pkg.svh"

// =====================================================================
// NOVA register file — 8 x 16 bits, single write port, comb read port
// =====================================================================
module nova_regfile (
    input  logic                  clk,
    input  logic                  we,
    input  logic [NOVA_RF_AW-1:0] wa,
    input  logic [NOVA_DATA_W-1:0] wd,
    input  logic [NOVA_RF_AW-1:0] ra,
    output logic [NOVA_DATA_W-1:0] qa
);
    logic [NOVA_DATA_W-1:0] regs [0:NOVA_RF_REGS-1];

    always_ff @(posedge clk) begin
        if (we) regs[wa] <= wd;
    end

    assign qa = regs[ra];
endmodule
