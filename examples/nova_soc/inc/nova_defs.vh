// =====================================================================
// NOVA-SoC global defines (.vh — VIKAS tolerates `define/`timescale and
// records them; standard-cell macros here document integration intent)
// =====================================================================
`timescale 1ns / 1ps

// simulation build of the NOVA fabric (silicon build ties this off)
`define NOVA_SIM 1

// UART oversampling divider: 4 clks per bit => 40 ns bit @ 100 MHz-model
`define NOVA_UART_DIV 4

// watchdog must be kicked via bus write 0xF3 within its period
`define NOVA_WDT_PERIOD_BITS 12
