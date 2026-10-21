// VIKAS demo 5 — FIFO backpressure / valid-ready handshake (perf demo).
module fifo_pe(
    input  logic       clk, rst_n,
    input  logic       valid_in,
    input  logic [7:0] data_in,
    output logic       ready_out,
    input  logic       valid_out_consumed,
    output logic [7:0] data_out
);
    logic [3:0] count;
    logic [7:0] mem [0:7];
    logic [2:0] wp, rp;
    assign ready_out = (count < 4'd8);
    assign data_out = mem[rp];
    always_ff @(posedge clk) begin
        if (!rst_n) begin
            count <= 0; wp <= 0; rp <= 0;
        end else begin
            if (valid_in && ready_out) begin
                mem[wp] <= data_in;
                wp <= wp + 1;
                count <= count + 1;
            end
            if (valid_out_consumed && count > 0) begin
                rp <= rp + 1;
                count <= count - 1;
            end
        end
    end
endmodule

module fifo_tb;
    logic clk, rst_n, valid_in, ready_out, valid_out_consumed;
    logic [7:0] data_in, data_out;
    fifo_pe dut(.clk(clk), .rst_n(rst_n), .valid_in(valid_in), .data_in(data_in),
                 .ready_out(ready_out), .valid_out_consumed(valid_out_consumed), .data_out(data_out));
    initial begin clk = 0; forever #5 clk = !clk; end
    initial begin
        rst_n = 0; valid_in = 0; valid_out_consumed = 0; data_in = 8'd0;
        #12 rst_n = 1;
        // burst write until full (backpressure demo)
        repeat (10) begin
            #3 valid_in = 1; data_in = data_in + 8'd1;
        end
        #3 valid_in = 0;
        // drain
        repeat (6) #4 valid_out_consumed = 1;
        #4 valid_out_consumed = 0;
        #20 $display("ready_out=%0b data_out=%0d", ready_out, data_out);
        $finish;
    end
endmodule
