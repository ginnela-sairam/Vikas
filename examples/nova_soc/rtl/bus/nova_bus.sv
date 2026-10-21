`include "nova_pkg.svh"

// =====================================================================
// NOVA bus fabric — address decode + RAM + peripheral fan-out + rdata mux
//   m0 = CPU (rw)   m1 = TB debug master (read-only)
//   map: 0x00-0x7F RAM · 0xF0 UART · 0xF1 SPI · 0xF2 TIMER cmp
//        0xF3 WDT kick · 0xF4 GPIO · 0xF5 PWM · 0xFD spare · 0xFE status
// =====================================================================
module nova_bus (
    input  logic                  clk,
    // master 0: CPU
    input  logic                  m0_we,
    input  logic                  m0_re,
    input  logic [NOVA_ADDR_W-1:0] m0_addr,
    input  logic [NOVA_DATA_W-1:0] m0_wdata,
    output logic [NOVA_DATA_W-1:0] m0_rdata,
    // master 1: debug (read-only)
    input  logic                  m1_req,
    input  logic                  m1_re,
    input  logic [NOVA_ADDR_W-1:0] m1_addr,
    output logic [NOVA_DATA_W-1:0] m1_rdata,
    // peripheral write strobes + data
    output logic                  uart_we,
    output logic [7:0]            uart_wdata,
    output logic                  spi_we,
    output logic [7:0]            spi_wdata,
    output logic                  timer_we,
    output logic [NOVA_DATA_W-1:0] timer_cmp,
    output logic                  wdt_kick,
    output logic                  gpio_we,
    output logic [NOVA_DATA_W-1:0] gpio_wdata,
    output logic                  pwm_we,
    output logic [NOVA_DATA_W-1:0] pwm_wdata,
    // peripheral readback / status
    input  logic [7:0]            spi_rdata,
    input  logic                  uart_busy,
    input  logic                  spi_done,
    input  logic                  timer_irq,
    input  logic                  wdt_bark,
    input  logic [1:0]            spare
);
    // arbitration
    logic gnt0, gnt1;
    nova_arbiter u_arb (.m0_req(m0_we | m0_re), .m1_req(m1_req), .gnt0(gnt0), .gnt1(gnt1));

    // granted master view
    logic                  we, re;
    logic [NOVA_ADDR_W-1:0] addr;
    logic [NOVA_DATA_W-1:0] wdata;
    assign we    = gnt0 ? m0_we    : 1'b0;
    assign re    = gnt0 ? m0_re    : (gnt1 ? m1_re : 1'b0);
    assign addr  = gnt1 ? m1_addr  : m0_addr;
    assign wdata = m0_wdata;

    // decode
    logic sel_ram, sel_periph;
    logic [3:0] sub;
    assign sel_ram    = !addr[7];
    assign sel_periph = addr[7] && (addr[7:4] == 4'hF);
    assign sub        = addr[3:0];

    // RAM instance (sync write, comb read)
    logic [NOVA_DATA_W-1:0] ram_q;
    nova_ram u_ram (.clk(clk), .we(we && sel_ram), .waddr(addr[6:0]),
                    .wdata(wdata), .raddr(addr[6:0]), .rdata(ram_q));

    // peripheral write strobes (single-cycle pulses)
    assign uart_we   = we && sel_periph && (sub == 4'h0);
    assign spi_we    = we && sel_periph && (sub == 4'h1);
    assign timer_we  = we && sel_periph && (sub == 4'h2);
    assign wdt_kick  = we && sel_periph && (sub == 4'h3);
    assign gpio_we   = we && sel_periph && (sub == 4'h4);
    assign pwm_we    = we && sel_periph && (sub == 4'h5);

    assign uart_wdata = wdata[7:0];
    assign spi_wdata  = wdata[7:0];
    assign timer_cmp  = wdata;
    assign gpio_wdata = wdata;
    assign pwm_wdata  = wdata;

    // read mux
    logic [NOVA_DATA_W-1:0] periph_q;
    always_comb begin
        case (sub)
            4'h1:    periph_q = {8'h00, spi_rdata};                       // SPI
            4'hD:    periph_q = {14'h0000, spare};                        // spare/tie
            4'hE:    periph_q = {12'h000, wdt_bark, timer_irq, uart_busy, spi_done}; // status
            default: periph_q = 16'h0000;
        endcase
    end

    assign m0_rdata = sel_ram ? ram_q : periph_q;
    assign m1_rdata = m0_rdata;
endmodule
