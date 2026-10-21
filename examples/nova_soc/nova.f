// =====================================================================
// NOVA-SoC — master filelist (industry-standard .f manifest)
// Build:  vikas run nova_soc/nova.f
// =====================================================================
+incdir+inc
+incdir+rtl
+incdir+tb
+define+NOVA_SIM
+define+NOVA_UART_DIV=4

// ---- include headers (pre-processed, parameters enter global const table)
inc/nova_defs.vh
inc/nova_pkg.svh

// ---- RTL: NOVA microcontroller core
rtl/core/nova_pc.sv
rtl/core/nova_regfile.sv
rtl/core/nova_alu.sv
rtl/core/nova_decoder.sv
rtl/core/nova_core.sv

// ---- RTL: bus fabric
rtl/bus/nova_arbiter.sv
rtl/bus/nova_bus.sv

// ---- RTL: peripherals
rtl/periph/nova_ram.sv
rtl/periph/nova_gpio.sv
rtl/periph/nova_uart_tx.sv
rtl/periph/nova_spi.sv
rtl/periph/nova_timer.sv
rtl/periph/nova_pwm.sv
rtl/periph/nova_sync.v
rtl/legacy/nova_wdt.v

// ---- physical library
rtl/netlist/nova_tiecell.netlist

// ---- golden-reference spec (tolerated: documentation + netlist stub)
rtl/spec/nova_golden.vho

// ---- top level
rtl/nova_soc_top.sv

// ---- testbench (nested filelist)
-f tb/tb.f
