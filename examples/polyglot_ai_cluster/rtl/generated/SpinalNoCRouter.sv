module SpinalNoCRouter (
    input  logic [1:0] io_route,
    input  logic [7:0] io_flit,
    output logic [7:0] io_west,
    output logic [7:0] io_east,
    output logic [7:0] io_north,
    output logic [7:0] io_south
);
    always_comb begin
        io_west = 8'h00;
        io_east = 8'h00;
        io_north = 8'h00;
        io_south = 8'h00;
        case (io_route)
            2'd0: io_west = io_flit;
            2'd1: io_east = io_flit;
            2'd2: io_north = io_flit;
            2'd3: io_south = io_flit;
            default: ;
        endcase
    end
endmodule
