`include "opcodes.vh"
module alu_core(
    input  logic [2:0]  op,
    input  logic [15:0] a, b,
    output logic [15:0] y
);
    always_comb begin
        case (op)
            OP_ADD:  y = a + b;
            OP_SUB:  y = a - b;
            OP_XOR:  y = a ^ b;
            default: y = a & b;
        endcase
    end
endmodule
