`include "nova_pkg.svh"

// =====================================================================
// NOVA-SoC top level — CPU + bus + 7 peripherals, all wired
//   hierarchy: tb -> nova_soc_top -> {nova_core -> {decoder, regfile,
//   alu, pc}, nova_bus -> {arbiter, ram}, uart, spi, timer, pwm, gpio,
//   sync(.v), wdt(.v), tiecells(.netlist)}   (5 levels deep)
// =====================================================================
module nova_soc_top (
    input  logic                  clk,
    input  logic                  rst_n,
    input  logic                  run,
    // program load (TB -> core)
    input  logic                  ld_imem,
    input  logic [NOVA_PC_W-1:0]  ld_addr,
    input  logic [NOVA_DATA_W-1:0] ld_data,
    // debug master (TB -> bus m1)
    input  logic                  dbg_req,
    input  logic                  dbg_re,
    input  logic [NOVA_ADDR_W-1:0] dbg_addr,
    output logic [NOVA_DATA_W-1:0] dbg_rdata,
    // serial / board IO
    output logic                  uart_tx,
    output logic                  uart_busy,
    input  logic                  uart_rx_in,
    output logic                  sync_rx,
    output logic                  spi_simo,
    input  logic                  spi_somi,
    // system status
    output logic                  timer_irq,
    output logic                  wdt_bark,
    output logic                  pwm_out,
    output logic [NOVA_DATA_W-1:0] gpio_out,
    output logic                  halted
);
    // core <-> bus
    logic                  bus_we, bus_re;
    logic [NOVA_ADDR_W-1:0]  bus_addr;
    logic [NOVA_DATA_W-1:0]  bus_wdata, bus_rdata;

    // bus <-> peripherals
    logic                  uart_we, spi_we, timer_we, wdt_kick, gpio_we, pwm_we;
    logic [7:0]            uart_wdata, spi_wdata;
    logic [NOVA_DATA_W-1:0] timer_cmp, gpio_wdata, pwm_wdata;
    logic [7:0]            spi_rdata;
    logic                  uart_busy, spi_done;
    // internal status wire (single driver: uart instance output port)
    logic [1:0]            spare;
    logic                  tc0, tc1;
    logic                  uart_busy_i;

    nova_core u_core (
        .clk(clk), .rst_n(rst_n), .run(run),
        .ld_imem(ld_imem), .ld_addr(ld_addr), .ld_data(ld_data),
        .bus_we(bus_we), .bus_re(bus_re), .bus_addr(bus_addr),
        .bus_wdata(bus_wdata), .bus_rdata(bus_rdata),
        .halted(halted)
    );

    nova_bus u_bus (
        .clk(clk),
        .m0_we(bus_we), .m0_re(bus_re), .m0_addr(bus_addr),
        .m0_wdata(bus_wdata), .m0_rdata(bus_rdata),
        .m1_req(dbg_req), .m1_re(dbg_re), .m1_addr(dbg_addr), .m1_rdata(dbg_rdata),
        .uart_we(uart_we), .uart_wdata(uart_wdata),
        .spi_we(spi_we), .spi_wdata(spi_wdata),
        .timer_we(timer_we), .timer_cmp(timer_cmp),
        .wdt_kick(wdt_kick),
        .gpio_we(gpio_we), .gpio_wdata(gpio_wdata),
        .pwm_we(pwm_we), .pwm_wdata(pwm_wdata),
        .spi_rdata(spi_rdata), .uart_busy(uart_busy), .spi_done(spi_done),
        .timer_irq(timer_irq), .wdt_bark(wdt_bark),
        .spare(spare)
    );

    nova_uart_tx u_uart (
        .clk(clk), .rst_n(rst_n), .start(uart_we), .data(uart_wdata),
        .busy(uart_busy_i), .tx(uart_tx)
    );

    nova_spi u_spi (
        .clk(clk), .rst_n(rst_n), .start(spi_we), .data_in(spi_wdata),
        .simo(spi_simo), .somi(spi_somi),
        .done(spi_done), .rx(spi_rdata)
    );

    nova_timer u_timer (
        .clk(clk), .rst_n(rst_n), .cmp_we(timer_we), .cmp(timer_cmp),
        .irq(timer_irq)
    );

    nova_pwm u_pwm (
        .clk(clk), .rst_n(rst_n), .duty_we(pwm_we), .duty_in(pwm_wdata),
        .pwm_out(pwm_out)
    );

    nova_gpio u_gpio (
        .clk(clk), .we(gpio_we), .wdata(gpio_wdata),
        .gpio_out(gpio_out)
    );

    nova_sync u_sync (
        .clk(clk), .rst_n(rst_n), .async_in(uart_rx_in),
        .sync_out(sync_rx)
    );

    nova_wdt u_wdt (
        .clk(clk), .rst_n(rst_n), .kick(wdt_kick),
        .bark(wdt_bark)
    );

    // physical tie cells feed the spare status bits (readable at 0xFD)
    nova_tiecell u_tc0 (.t(tc0));
    nova_tiecell u_tc1 (.t(tc1));
    assign spare = {tc1, tc0};
    assign uart_busy = uart_busy_i;
endmodule
