`include "nova_pkg.svh"

// =====================================================================
// NOVA ALU — 16-bit datapath, 10 operations selected by ISA opcode
//   1 ADD · 2 SUB · 3 XOR · 4 AND · 5 OR · 6 SHL · 7 SHR · 8 PASSB(LDI)
// =====================================================================
module nova_alu (
    input  logic [3:0]              op,
    input  logic [NOVA_DATA_W-1:0]  a,
    input  logic [NOVA_DATA_W-1:0]  b,
    output logic [NOVA_DATA_W-1:0]  y
);
    always_comb begin
        case (op)
            4'd1:    y = a + b;
            4'd2:    y = a - b;
            4'd3:    y = a ^ b;
            4'd4:    y = a & b;
            4'd5:    y = a | b;
            4'd6:    y = a << b[3:0];
            4'd7:    y = a >> b[3:0];
            4'd8:    y = b;          // LDI: pass immediate
            default: y = a;
        endcase
    end
endmodule
