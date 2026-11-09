-- =====================================================================
-- VHDL-2019 Mesh NoC Flit Router (lowered by VIKAS-POLYGLOT to IEEE 1800 SV)
-- =====================================================================
library ieee;
use ieee.std_logic_1164.all;

entity noc_router is
  port (
    clk      : in std_logic;
    rst_n    : in std_logic;
    flit_in  : in std_logic_vector(15 downto 0);
    flit_out : out std_logic_vector(15 downto 0)
  );
end entity noc_router;
