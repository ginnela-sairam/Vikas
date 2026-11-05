// [VIKAS-KRONOS] IEEE 1800-2023 SystemVerilog Synthesis & Simulation Target
module nova_dsp_mac #(parameter int WIDTH = 32) (
    input  logic             clk,
    input  logic             rst_n,
    input  logic             valid_i,
    input  logic [WIDTH-1:0] a_i,
    input  logic [WIDTH-1:0] b_i,
    input  logic [WIDTH-1:0] acc_i,
    output logic [WIDTH-1:0] mac_o,
    output logic             valid_o
);
    typedef enum logic [1:0] { S_IDLE, S_ACTIVE, S_DONE } state_t;
state_t state_q, state_d;
always_ff @(posedge clk or negedge rst_n) begin
    if (!rst_n) state_q <= S_IDLE;
    else        state_q <= state_d;
end
always_comb begin
    state_d = state_q;
    unique case (state_q)
        S_IDLE:   if (start) state_d = S_ACTIVE;
        S_ACTIVE: if (done)  state_d = S_DONE;
        S_DONE:              state_d = S_IDLE;
        default:             state_d = S_IDLE;
    endcase
end
    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            mac_o   <= '0;
            valid_o <= 1'b0;
        end else begin
            valid_o <= valid_i;
            if (valid_i) mac_o <= (a_i * b_i) + acc_i;
        end
    end
endmodule