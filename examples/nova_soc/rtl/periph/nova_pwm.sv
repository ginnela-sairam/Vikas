`include "nova_pkg.svh"

// =====================================================================
// NOVA PWM — 4-bit period counter, duty written via 0xF5 (0 = disabled)
// out = (count < duty) — duty 8/16 gives a 50% waveform
// =====================================================================
module nova_pwm (
    input  logic                  clk,
    input  logic                  rst_n,
    input  logic                  duty_we,
    input  logic [NOVA_DATA_W-1:0] duty_in,
    output logic                  pwm_out
);
    logic [NOVA_DATA_W-1:0] duty;
    logic [3:0]             count;

    always_ff @(posedge clk) begin
        if (!rst_n) begin
            duty <= 16'd0; count <= 4'd0; pwm_out <= 1'b0;
        end else begin
            if (duty_we) begin
                duty <= duty_in;
                count <= 4'd0;
                pwm_out <= 1'b0;
            end else if (duty != 16'd0) begin
                count <= count + 1'b1;
                if (count == 4'd15) count <= 4'd0;
                pwm_out <= (count < duty[3:0]);
            end
        end
    end
endmodule
