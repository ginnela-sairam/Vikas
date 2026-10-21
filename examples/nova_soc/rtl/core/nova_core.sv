`include "nova_pkg.svh"

// =====================================================================
// NOVA CPU core — 2-phase (FETCH/EXEC) multicycle machine
//   ISA: 0 NOP · 1 ADD · 2 SUB · 3 XOR · 4 AND · 5 OR · 6 SHL · 7 SHR
//        8 LDI · 9 LD · A(10) ST · B(11) BZ · C(12) JMP · D(13) CMP
//        E(14) OUT · F(15) HALT
//   state: 0 FETCH · 1 EXEC · 2 LDWAIT · 3 HALTED
// =====================================================================
module nova_core (
    input  logic                clk,
    input  logic                rst_n,
    input  logic                run,          // release PC after program load
    // program load port (TB writes instruction memory directly)
    input  logic                ld_imem,
    input  logic [NOVA_PC_W-1:0]  ld_addr,
    input  logic [NOVA_DATA_W-1:0] ld_data,
    // bus master interface
    output logic                bus_we,
    output logic                bus_re,
    output logic [NOVA_ADDR_W-1:0]  bus_addr,
    output logic [NOVA_DATA_W-1:0]  bus_wdata,
    input  logic [NOVA_DATA_W-1:0]  bus_rdata,
    output logic                halted
);
    // internal state
    logic [1:0]              state;
    logic [NOVA_DATA_W-1:0]  ir;
    logic [NOVA_PC_W-1:0]    pc_q;
    logic [2:0]              ld_rd;        // destination reg of in-flight LD
    logic                    zflag, nflag;

    // instruction memory (Harvard, loaded via ld port)
    logic [NOVA_DATA_W-1:0]  imem [0:NOVA_IMEM_W-1];

    // sub-block instances
    logic [3:0]              d_op;
    logic [2:0]              d_rd;
    logic [7:0]              d_imm;
    logic [NOVA_DATA_W-1:0]  rf_qa;
    logic [NOVA_DATA_W-1:0]  alu_y;
    logic                    rf_we;
    logic [NOVA_RF_AW-1:0]   rf_wa;
    logic [NOVA_DATA_W-1:0]  rf_wd;
    logic                    pc_inc, pc_load;

    nova_decoder  u_dec  (.instr(ir),  .op(d_op), .rd(d_rd), .imm8(d_imm));
    nova_regfile  u_rf   (.clk(clk), .we(rf_we), .wa(rf_wa), .wd(rf_wd),
                          .ra(d_rd),  .qa(rf_qa));
    nova_alu      u_alu  (.op(d_op), .a(rf_qa), .b({8'h00, d_imm}), .y(alu_y));
    nova_pc       u_pc   (.clk(clk), .rst_n(rst_n), .inc(pc_inc), .load(pc_load),
                          .target(d_imm[NOVA_PC_W-1:0]), .pc(pc_q));

    // PC control — COMBINATIONAL (design fix): pc_inc/pc_load must be valid
    // DURING the EXEC cycle so the branch target lands at the END of EXEC and
    // the NEXT fetch reads imem[target]. The previous registered version acted
    // one fetch late — every taken branch executed its fall-through "delay
    // slot" first (BZ→JMP→ST→loop re-entry), corrupting the program flow.
    always_comb begin
        pc_inc  = 1'b0;
        pc_load = 1'b0;
        if (state == 2'd2) begin
            pc_inc = 1'b1;                        // LD completes -> advance
        end else if (state == 2'd1) begin
            case (d_op)
                4'd11: begin                      // BZ
                    if (zflag) pc_load = 1'b1;
                    else        pc_inc  = 1'b1;
                end
                4'd12: pc_load = 1'b1;            // JMP
                4'd9:  pc_inc = 1'b0;             // LD: advance in LDWAIT
                4'd15: pc_inc = 1'b0;             // HALT: freeze
                default: pc_inc = 1'b1;           // all 1-cycle ops
            endcase
        end
    end

    // instruction fetch register
    always_ff @(posedge clk) begin
        if (!rst_n) begin
            ir <= 16'd0;
        end else if (state == 2'd0) begin
            ir <= imem[pc_q];
        end
    end

    // program load port
    always_ff @(posedge clk) begin
        if (ld_imem) imem[ld_addr] <= ld_data;
    end

    // main control FSM
    always_ff @(posedge clk) begin
        if (!rst_n) begin
            state <= 2'd0; rf_we <= 1'b0; bus_we <= 1'b0; bus_re <= 1'b0;
            bus_addr <= 8'd0; bus_wdata <= 16'd0; rf_wa <= 3'd0; rf_wd <= 16'd0;
            ld_rd <= 3'd0; zflag <= 1'b0; nflag <= 1'b0; halted <= 1'b0;
        end else begin
            // defaults: single-cycle pulses self-clear
            rf_we <= 1'b0; bus_we <= 1'b0; bus_re <= 1'b0;

            if (!run) begin
                state <= 2'd0;               // held at reset-fetch until released
            end else begin
                case (state)
                    2'd0: begin              // FETCH: ir loads this cycle
                        state <= 2'd1;
                    end
                    2'd1: begin              // EXEC
                        case (d_op)
                            4'd1, 4'd2, 4'd3, 4'd4, 4'd5, 4'd6, 4'd7, 4'd8: begin
                                rf_we <= 1'b1; rf_wa <= d_rd; rf_wd <= alu_y;
                                state <= 2'd0;
                            end
                            4'd9: begin       // LD rd,[imm8]
                                bus_re <= 1'b1;
                                bus_addr <= d_imm;
                                ld_rd <= d_rd;
                                state <= 2'd2;
                            end
                            4'd10: begin      // ST [imm8],rd
                                bus_we <= 1'b1;
                                bus_addr <= d_imm;
                                bus_wdata <= rf_qa;
                                state <= 2'd0;
                            end
                            4'd11: begin      // BZ imm8 (pc handled combinationally)
                                state <= 2'd0;
                            end
                            4'd12: begin      // JMP imm8 (pc handled combinationally)
                                state <= 2'd0;
                            end
                            4'd13: begin      // CMP rd,imm8
                                zflag <= (rf_qa == {8'h00, d_imm});
                                nflag <= (rf_qa  < {8'h00, d_imm});
                                state <= 2'd0;
                            end
                            4'd14: begin      // OUT rd -> GPIO (ST to 0xF4)
                                bus_we <= 1'b1;
                                bus_addr <= 8'hF4;
                                bus_wdata <= rf_qa;
                                state <= 2'd0;
                            end
                            4'd15: begin      // HALT
                                state <= 2'd3;
                                halted <= 1'b1;
                            end
                            default: begin   // 0 NOP
                                state <= 2'd0;
                            end
                        endcase
                    end
                    2'd2: begin              // LDWAIT: capture comb bus read
                        rf_we <= 1'b1; rf_wa <= ld_rd; rf_wd <= bus_rdata;
                        state <= 2'd0;
                    end
                    2'd3: begin              // HALTED
                        halted <= 1'b1;
                    end
                endcase
            end
        end
    end
endmodule
