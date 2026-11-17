module polyglot_examples_tb;
    logic [7:0] mac_a, mac_b;
    logic [15:0] mac_acc, mac_y;
    logic [1:0] route;
    logic [7:0] flit, west, east, north, south;

    ChiselAiMacTile mac(
        .io_a(mac_a), .io_b(mac_b), .io_acc(mac_acc), .io_y(mac_y)
    );
    SpinalNoCRouter router(
        .io_route(route), .io_flit(flit),
        .io_west(west), .io_east(east), .io_north(north), .io_south(south)
    );

    initial begin
        mac_a = 8'd3;
        mac_b = 8'd4;
        mac_acc = 16'd5;
        route = 2'd2;
        flit = 8'hA5;
        #1;
        if (mac_y != 16'd17) begin
            $display("POLYGLOT-EXAMPLES-FAIL chisel expected 17 got %0d", mac_y);
            $finish;
        end
        if (north != 8'hA5 || west != 0 || east != 0 || south != 0) begin
            $display("POLYGLOT-EXAMPLES-FAIL spinal route mismatch");
            $finish;
        end
        $display("POLYGLOT-EXAMPLES-PASS 2/2 — Chisel MAC + SpinalHDL NoC router");
        $finish;
    end
endmodule
