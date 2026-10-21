// =====================================================================
// NOVA-SoC package parameters (.svh — file-scope parameters enter the
// VIKAS global constant table and const-fold into every module range)
// =====================================================================
parameter NOVA_DATA_W    = 16;   // data bus width
parameter NOVA_ADDR_W    = 8;    // address bus width (256-byte map)
parameter NOVA_RAM_WORDS = 128;  // 0x00-0x7F data RAM
parameter NOVA_IMEM_W    = 64;   // instruction memory words
parameter NOVA_PC_W      = 6;    // program counter width
parameter NOVA_RF_REGS   = 8;    // R0..R7
parameter NOVA_RF_AW     = 3;
parameter NOVA_UART_DIV  = 4;    // clks per UART bit
parameter NOVA_SPI_DIV   = 2;    // clks per SPI bit
parameter NOVA_WDT_BITS  = 12;   // watchdog period = 2^12 clks

// memory map
parameter NOVA_A_UART    = 8'hF0;
parameter NOVA_A_SPI     = 8'hF1;
parameter NOVA_A_TIMER   = 8'hF2;
parameter NOVA_A_WDT     = 8'hF3;
parameter NOVA_A_GPIO    = 8'hF4;
parameter NOVA_A_PWM     = 8'hF5;
parameter NOVA_A_SPARE   = 8'hFD;
parameter NOVA_A_STATUS  = 8'hFE;
parameter NOVA_A_HALT    = 8'hFF;
