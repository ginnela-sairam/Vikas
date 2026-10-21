// =====================================================================
// NOVA-SoC helper functions (.svh — parsed by VIKAS; procedural TBs do
// their checking inline, these document the reusable verification API)
// =====================================================================

// population count of a 16-bit vector (power/toggle analytics helper)
function [15:0] nova_popcnt16;
    input [15:0] x;
    integer k;
    begin
        nova_popcnt16 = 0;
        for (k = 0; k < 16; k = k + 1)
            nova_popcnt16 = nova_popcnt16 + x[k];
    end
endfunction

// reverse a byte (UART/SPI bit-order conversion helper)
function [7:0] nova_rev8;
    input [7:0] x;
    integer k;
    begin
        for (k = 0; k < 8; k = k + 1)
            nova_rev8[7 - k] = x[k];
    end
endfunction
