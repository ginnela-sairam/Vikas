`include "nova_pkg.svh"

// =====================================================================
// NOVA program counter — branchable up-counter (params: NOVA_PC_W)
// =====================================================================
module nova_pc (
    input  logic              clk,
    input  logic              rst_n,
    input  logic              inc,      // advance to next instruction
    input  logic              load,     // branch/jump to target
    input  logic [NOVA_PC_W-1:0] target,
    output logic [NOVA_PC_W-1:0] pc
);
    always_ff @(posedge clk) begin
        if (!rst_n)       pc <= 6'd0;
        else if (load)    pc <= target;
        else if (inc)     pc <= pc + 1'b1;
    end
endmodule
