`include "nova_pkg.svh"

// =====================================================================
// NOVA SPI master — mode 0 (CPOL=0, CPHA=0), MSB-first, loopback-capable
// start strobe = bus write to 0xF1; done is sticky until next start;
// receive byte readable at 0xF1 (shifts somi on the same edge)
// =====================================================================
module nova_spi (
    input  logic        clk,
    input  logic        rst_n,
    input  logic        start,
    input  logic [7:0]  data_in,
    output logic        simo,
    input  logic        somi,
    output logic        done,
    output logic [7:0]  rx
);
    logic [7:0]              txsh, rxsh;
    logic [3:0]              bit_cnt;
    logic [NOVA_SPI_DIV-1:0] div;
    logic                    active;

    always_ff @(posedge clk) begin
        if (!rst_n) begin
            txsh <= 8'd0; rxsh <= 8'd0; bit_cnt <= 4'd0; div <= 0;
            active <= 1'b0; done <= 1'b0; simo <= 1'b0; rx <= 8'd0;
        end else if (start) begin
            txsh <= data_in; rxsh <= 8'd0; bit_cnt <= 4'd0; div <= 0;
            active <= 1'b1; done <= 1'b0; simo <= data_in[7];
        end else if (active) begin
            if (div == NOVA_SPI_DIV-1) begin
                div <= 0;
                if (bit_cnt == 4'd7) begin
                    rxsh <= {rxsh[6:0], somi};
                    rx <= {rxsh[6:0], somi};
                    active <= 1'b0;
                    done <= 1'b1;
                end else begin
                    txsh <= {txsh[6:0], 1'b0};
                    rxsh <= {rxsh[6:0], somi};
                    simo <= txsh[6];
                    bit_cnt <= bit_cnt + 1'b1;
                end
            end else begin
                div <= div + 1'b1;
            end
        end
    end
endmodule
