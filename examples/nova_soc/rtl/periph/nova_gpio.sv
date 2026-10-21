`include "nova_pkg.svh"

// =====================================================================
// NOVA GPIO — memory-mapped output register (0xF4)
// =====================================================================
module nova_gpio (
    input  logic                  clk,
    input  logic                  we,
    input  logic [NOVA_DATA_W-1:0] wdata,
    output logic [NOVA_DATA_W-1:0] gpio_out
);
    always_ff @(posedge clk) begin
        if (we) gpio_out <= wdata;
    end
endmodule
