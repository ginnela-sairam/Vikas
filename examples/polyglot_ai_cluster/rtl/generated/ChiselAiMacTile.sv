module ChiselAiMacTile (
    input  logic [7:0]  io_a,
    input  logic [7:0]  io_b,
    input  logic [15:0] io_acc,
    output logic [15:0] io_y
);
    assign io_y = io_a * io_b + io_acc;
endmodule
