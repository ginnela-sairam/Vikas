`include "nova_defs.vh"
`include "nova_pkg.svh"
`include "nova_scoreboard.svh"

// =====================================================================
// NOVA-SoC testbench — procedural offline regression (UVM-equivalent
// checking per tb/nova_uvm_tag.uvm). Loads a 27-instruction program,
// runs it to HALT, then verifies 9 golden checks across 5 hierarchy
// levels, 7 peripherals, the debug bus master, and 2 serial monitors.
// =====================================================================
module nova_soc_tb;
    // DUT connections
    logic                  clk, rst_n, run;
    logic                  ld_imem;
    logic [5:0]            ld_addr;
    logic [15:0]           ld_data;
    logic                  dbg_req, dbg_re;
    logic [7:0]            dbg_addr;
    logic [15:0]           dbg_rdata;
    logic                  uart_tx, uart_busy, uart_rx_in, sync_rx;
    logic                  spi_simo, spi_somi;
    logic                  timer_irq, wdt_bark, pwm_out, halted;
    logic [15:0]           gpio_out;

    // test machinery
    logic [15:0]           prog [0:26];
    logic [5:0]            i;
    logic [7:0]            guard;
    logic [7:0]            rx_byte;
    logic [7:0]            k;
    logic [7:0]            pwm_ones;
    integer                fails;

    nova_soc_top dut (
        .clk(clk), .rst_n(rst_n), .run(run),
        .ld_imem(ld_imem), .ld_addr(ld_addr), .ld_data(ld_data),
        .dbg_req(dbg_req), .dbg_re(dbg_re), .dbg_addr(dbg_addr),
        .dbg_rdata(dbg_rdata),
        .uart_tx(uart_tx), .uart_busy(uart_busy), .uart_rx_in(uart_rx_in),
        .sync_rx(sync_rx), .spi_simo(spi_simo), .spi_somi(spi_somi),
        .timer_irq(timer_irq), .wdt_bark(wdt_bark), .pwm_out(pwm_out),
        .gpio_out(gpio_out), .halted(halted)
    );

    // SPI loopback on the "board"
    assign spi_somi = spi_simo;

    // clock: 10 ns period
    initial begin
        clk = 0;
        forever #5 clk = ~clk;
    end

    // UART RX monitor: sample 8 data bits mid-bit on busy rising edge
    always @(uart_busy) begin
        if (uart_busy) begin
            #60;
            for (k = 0; k < 8; k = k + 1) begin
                rx_byte[k] = uart_tx;
                #40;
            end
        end
    end

    // PWM activity monitor: count rising edges
    always @(pwm_out) begin
        if (pwm_out) pwm_ones = pwm_ones + 1;
    end

    initial begin
        // ---- program image (op[15:12] · rd[11:9] · imm[7:0]) ------------
        prog[0]  = 16'h8000;  // LDI R0,#0
        prog[1]  = 16'h820A;  // LDI R1,#10
        prog[2]  = 16'h8400;  // LDI R2,#0
        prog[3]  = 16'h8600;  // LDI R3,#0
        prog[4]  = 16'h1607;  // ADD R3,#7        ; sum += 7
        prog[5]  = 16'h1401;  // ADD R2,#1        ; i++
        prog[6]  = 16'hD40A;  // CMP R2,#10       ; Z = (i == 10)
        prog[7]  = 16'hB009;  // BZ  9            ; exit loop when done
        prog[8]  = 16'hC004;  // JMP 4
        prog[9]  = 16'hA640;  // ST  [0x40],R3    ; ram[0x40] = 70
        prog[10] = 16'hE600;  // OUT R3           ; gpio = 0x0046
        prog[11] = 16'h88A5;  // LDI R4,#0xA5
        prog[12] = 16'hA8F0;  // ST  [0xF0],R4    ; uart tx 0xA5
        prog[13] = 16'h8A3C;  // LDI R5,#0x3C
        prog[14] = 16'hAAF2;  // ST  [0xF2],R5    ; timer cmp = 60
        prog[15] = 16'hA0F3;  // ST  [0xF3],R0    ; wdt kick
        prog[16] = 16'h8C5A;  // LDI R6,#0x5A
        prog[17] = 16'hACF1;  // ST  [0xF1],R6    ; spi tx 0x5A (loopback)
        prog[18] = 16'h9EFE;  // LD  R7,[0xFE]    ; status
        prog[19] = 16'h4E01;  // AND R7,#1        ; isolate spi_done
        prog[20] = 16'hDE00;  // CMP R7,#0        ; Z = (!spi_done)
        prog[21] = 16'hB012;  // BZ  18           ; spin until spi done
        prog[22] = 16'h9EF1;  // LD  R7,[0xF1]    ; R7 = 0x5A
        prog[23] = 16'hEE00;  // OUT R7           ; gpio = 0x005A
        prog[24] = 16'h8A08;  // LDI R5,#8
        prog[25] = 16'hAAF5;  // ST  [0xF5],R5    ; pwm duty = 8/16
        prog[26] = 16'hF000;  // HALT

        fails    = 0;
        rx_byte  = 8'h00;
        pwm_ones = 8'h00;
        rst_n = 0; run = 0; ld_imem = 0; ld_addr = 0; ld_data = 0;
        dbg_req = 0; dbg_re = 0; dbg_addr = 0;
        uart_rx_in = 0;

        // ---- reset, load program over 27 bus-writes ------------------
        #40 rst_n = 1;
        for (i = 0; i < 27; i = i + 1) begin
            ld_imem = 1; ld_addr = i; ld_data = prog[i];
            #10 ld_imem = 0;
            #10;
        end

        // async sync-path exercise: drive uart_rx_in HIGH (level) through the
        // 2-FF synchronizer — stays 1 for the rest of the test
        uart_rx_in = 1;

        // ---- release the core ----------------------------------------
        #20 run = 1;

        // ---- wait for HALT (bounded poll) ----------------------------
        guard = 0;
        while ((!halted) && (guard < 200)) begin
            #20;
            guard = guard + 1;
        end

        // let UART frame + timer + PWM settle
        #700;

        // ---- 9 golden checks ------------------------------------------
        if (gpio_out == NOVA_EXP_GPIO)         $display("NOVA-OK   1 gpio_out  = %04h", gpio_out);
        else begin $display("NOVA-FAIL 1 gpio_out  = %04h (expect %04h)", gpio_out, NOVA_EXP_GPIO); fails = fails + 1; end

        dbg_req = 1; dbg_re = 1; dbg_addr = 8'h40; #20;
        if (dbg_rdata == NOVA_EXP_RAM_40)      $display("NOVA-OK   2 ram[0x40] = %0d", dbg_rdata);
        else begin $display("NOVA-FAIL 2 ram[0x40] = %0d (expect %0d)", dbg_rdata, NOVA_EXP_RAM_40); fails = fails + 1; end

        if (rx_byte == NOVA_EXP_UART)          $display("NOVA-OK   3 uart_rx   = %02h", rx_byte);
        else begin $display("NOVA-FAIL 3 uart_rx   = %02h (expect %02h)", rx_byte, NOVA_EXP_UART); fails = fails + 1; end

        dbg_addr = 8'hF1; #20;
        if (dbg_rdata == {8'h00, NOVA_EXP_SPI}) $display("NOVA-OK   4 spi_rx    = %02h", dbg_rdata[7:0]);
        else begin $display("NOVA-FAIL 4 spi_rx    = %02h (expect %02h)", dbg_rdata[7:0], NOVA_EXP_SPI); fails = fails + 1; end

        if (timer_irq == 1'b1)                 $display("NOVA-OK   5 timer_irq = %0d", timer_irq);
        else begin $display("NOVA-FAIL 5 timer_irq = %0d (expect 1)", timer_irq); fails = fails + 1; end

        if (wdt_bark == 1'b0)                  $display("NOVA-OK   6 wdt_bark  = %0d", wdt_bark);
        else begin $display("NOVA-FAIL 6 wdt_bark  = %0d (expect 0)", wdt_bark); fails = fails + 1; end

        if (sync_rx == 1'b1)                   $display("NOVA-OK   7 sync_rx   = %0d", sync_rx);
        else begin $display("NOVA-FAIL 7 sync_rx   = %0d (expect 1)", sync_rx); fails = fails + 1; end

        dbg_addr = 8'hFD; #20;
        if (dbg_rdata == NOVA_EXP_SPARE)       $display("NOVA-OK   8a spare    = %04h", dbg_rdata);
        else begin $display("NOVA-FAIL 8a spare    = %04h (expect %04h)", dbg_rdata, NOVA_EXP_SPARE); fails = fails + 1; end
        dbg_addr = 8'hFE; #20;
        if (dbg_rdata == NOVA_EXP_STATUS)      $display("NOVA-OK   8b status   = %04h", dbg_rdata);
        else begin $display("NOVA-FAIL 8b status   = %04h (expect %04h)", dbg_rdata, NOVA_EXP_STATUS); fails = fails + 1; end

        if (pwm_ones >= NOVA_EXP_PWM_MIN)      $display("NOVA-OK   9 pwm_edges = %0d", pwm_ones);
        else begin $display("NOVA-FAIL 9 pwm_edges = %0d (expect >= %0d)", pwm_ones, NOVA_EXP_PWM_MIN); fails = fails + 1; end

        dbg_req = 0; dbg_re = 0;

        // ---- verdict ---------------------------------------------------
        if (fails == 0) $display("NOVA-PASS 9/9 — SoC fabric, core, 7 peripherals, dual-master bus verified");
        else            $display("NOVA-FAIL %0d/9 checks failed", fails);
        $finish;
    end
endmodule
