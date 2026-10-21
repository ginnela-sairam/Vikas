// =====================================================================
// NOVA 2-flop synchronizer — legacy Verilog-2001 style (.v extension
// proves VIKAS handles mixed-era sources in one build)
// =====================================================================
module nova_sync (
    input  wire clk,
    input  wire rst_n,
    input  wire async_in,
    output reg  sync_out
);
    reg ff0;
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            ff0 <= 1'b0;
            sync_out <= 1'b0;
        end else begin
            ff0 <= async_in;
            sync_out <= ff0;
        end
    end
endmodule
