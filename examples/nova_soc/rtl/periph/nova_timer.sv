`include "nova_pkg.svh"

// =====================================================================
// NOVA system timer — 16-bit up counter, IRQ when count >= compare
// (compare write 0xF2 enables; IRQ is a level held once reached)
// =====================================================================
module nova_timer (
    input  logic                  clk,
    input  logic                  rst_n,
    input  logic                  cmp_we,
    input  logic [NOVA_DATA_W-1:0] cmp,
    output logic                  irq
);
    logic [NOVA_DATA_W-1:0] cmp_q;
    logic [NOVA_DATA_W-1:0] count;

    always_ff @(posedge clk) begin
        if (!rst_n) begin
            cmp_q <= 16'd0; count <= 16'd0; irq <= 1'b0;
        end else begin
            if (cmp_we) begin
                cmp_q <= cmp;
                count <= 16'd0;
                irq   <= 1'b0;
            end else if (cmp_q != 16'd0) begin
                count <= count + 1'b1;
                if (count >= cmp_q) irq <= 1'b1;
            end
        end
    end
endmodule
