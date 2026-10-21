// =====================================================================
// NOVA watchdog timer — legacy Verilog-2001 (.v), 2^12-clk period,
// kicked via bus write 0xF3; bark is a sticky flag once expired
// =====================================================================
module nova_wdt (
    input  wire        clk,
    input  wire        rst_n,
    input  wire        kick,
    output reg         bark
);
    reg [11:0] count;
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            count <= 12'd0;
            bark  <= 1'b0;
        end else begin
            if (kick) begin
                count <= 12'd0;
            end else begin
                if (count == 12'hFFF) begin
                    bark <= 1'b1;         // sticky — only reset clears
                end else begin
                    count <= count + 1'b1;
                end
            end
        end
    end
endmodule
