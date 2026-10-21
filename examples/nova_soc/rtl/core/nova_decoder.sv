`include "nova_pkg.svh"

// =====================================================================
// NOVA instruction decoder — pure combinational field extraction
//   instr[15:12] = opcode · [11:8] = rd · [7:0] = immediate
// =====================================================================
module nova_decoder (
    input  logic [NOVA_DATA_W-1:0] instr,
    output logic [3:0]             op,
    output logic [2:0]             rd,
    output logic [7:0]             imm8
);
    assign op   = instr[15:12];
    assign rd   = instr[11:9];
    assign imm8 = instr[7:0];
endmodule
