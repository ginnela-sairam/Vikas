// =====================================================================
// FIRRTL Tensor DMA Engine (lowered by VIKAS-POLYGLOT to IEEE 1800 SV)
// =====================================================================
circuit dma_engine :
  module dma_engine :
    input clk : Clock
    input rst_n : UInt<1>
    input dma_src : UInt<16>
    output dma_dst : UInt<16>
