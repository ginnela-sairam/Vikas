// =====================================================================
// NOVA-SoC scoreboard (.svh — expected values, `include'd by the TB)
// Golden expectations match rtl/spec/nova_golden.vho
// =====================================================================
`ifndef NOVA_SCOREBOARD_SVH
`define NOVA_SCOREBOARD_SVH

parameter NOVA_EXP_SUM      = 16'd70;     // 7 * 10 iterations
parameter NOVA_EXP_RAM_40   = 16'd70;     // ST [0x40],R3
parameter NOVA_EXP_GPIO     = 16'h005A;   // final OUT R7 (SPI loopback byte)
parameter NOVA_EXP_UART     = 8'hA5;      // UART TX frame byte
parameter NOVA_EXP_SPI      = 8'h5A;      // SPI loopback receive byte
parameter NOVA_EXP_STATUS   = 16'h0005;   // {bark=0,irq=1,busy=0,done=1} = 0101
parameter NOVA_EXP_SPARE    = 16'h0000;   // tie cells
parameter NOVA_EXP_PWM_MIN  = 3;          // min rising edges in wait window

`endif
