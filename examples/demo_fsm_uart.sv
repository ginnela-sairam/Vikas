// VIKAS demo 3 — UART TX FSM with enum states (FSM extractor demo).
module uart_tx(
    input  logic       clk, rst_n,
    input  logic       tx_start,
    input  logic [7:0] tx_data,
    output logic       tx, tx_busy, tx_done
);
    typedef enum logic [2:0] { IDLE=3'd0, START=3'd1, DATA=3'd2, STOP=3'd3, DONE=3'd4 } state_t;
    state_t state, nstate;
    logic [2:0] bit_idx;
    logic [7:0] shifter;
    always_ff @(posedge clk) begin
        if (!rst_n) begin
            state <= IDLE; bit_idx <= 3'd0; shifter <= 8'd0;
        end else state <= nstate;
    end
    always_comb begin
        nstate = state; tx = 1'b1; tx_done = 1'b0;
        case (state)
            IDLE: begin tx = 1'b1; if (tx_start) nstate = START; end
            START: begin tx = 1'b0; nstate = DATA; end
            DATA: begin tx = shifter[0]; if (bit_idx == 3'd7) nstate = STOP; end
            STOP: begin tx = 1'b1; nstate = DONE; end
            DONE: begin tx_done = 1'b1; nstate = IDLE; end
            default: nstate = IDLE;
        endcase
    end
    // shift logic
    always_ff @(posedge clk) begin
        if (!rst_n) begin
            bit_idx <= 0; shifter <= tx_data; tx_busy <= 0;
        end else begin
            case (state)
                IDLE: if (tx_start) begin shifter <= tx_data; bit_idx <= 0; tx_busy <= 1; end
                START: begin shifter <= tx_data; end
                DATA: begin shifter <= {1'b0, shifter[7:1]}; bit_idx <= bit_idx + 1; end
                DONE: tx_busy <= 0;
                default: ;
            endcase
        end
    end
endmodule

module uart_tb;
    logic clk, rst_n, tx_start, tx, tx_busy, tx_done;
    logic [7:0] tx_data;
    uart_tx dut(.clk(clk), .rst_n(rst_n), .tx_start(tx_start), .tx_data(tx_data),
                 .tx(tx), .tx_busy(tx_busy), .tx_done(tx_done));
    initial begin clk = 0; forever #5 clk = !clk; end
    initial begin
        rst_n = 0; tx_start = 0; tx_data = 8'hA5;
        #12 rst_n = 1;
        #3 tx_start = 1;
        #10 tx_start = 0;
        #80 $display("uart tx_done=%0b busy=%0b", tx_done, tx_busy);
        $finish;
    end
endmodule
