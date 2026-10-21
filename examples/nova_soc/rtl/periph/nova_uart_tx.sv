`include "nova_pkg.svh"

// =====================================================================
// NOVA UART TX — 8N1, NOVA_UART_DIV clocks per bit (40 ns @ 10 ns clk)
// start strobe = bus write to 0xF0; frame: start(0) + 8 data LSB-first + stop(1)
// =====================================================================
module nova_uart_tx (
    input  logic        clk,
    input  logic        rst_n,
    input  logic        start,
    input  logic [7:0]  data,
    output logic        busy,
    output logic        tx
);
    logic [3:0]           bit_cnt;
    logic [NOVA_UART_DIV-1:0] div;
    logic [9:0]           sh;

    always_ff @(posedge clk) begin
        if (!rst_n) begin
            busy <= 1'b0; tx <= 1'b1; bit_cnt <= 4'd0; div <= 0; sh <= 10'h3FF;
        end else begin
            if (start) begin
                sh <= {1'b1, data, 1'b0};   // stop, data[7:0], start
                busy <= 1'b1; bit_cnt <= 4'd0; div <= 0; tx <= 1'b0;
            end else if (busy) begin
                if (div == NOVA_UART_DIV-1) begin
                    div <= 0;
                    if (bit_cnt == 4'd9) begin
                        busy <= 1'b0; tx <= 1'b1;
                    end else begin
                        bit_cnt <= bit_cnt + 1'b1;
                        sh <= {1'b1, sh[9:1]};
                        tx <= sh[1];
                    end
                end else begin
                    div <= div + 1'b1;
                end
            end
        end
    end
endmodule
