// =====================================================================
// NOVA bus arbiter — 2-master priority grant (m1 = debug master has
// priority; grant0 default). One-hot grants, no lock, no starvation
// of m0 in practice since m1 issues are single-cycle reads.
// =====================================================================
module nova_arbiter (
    input  logic m0_req,
    input  logic m1_req,
    output logic gnt0,
    output logic gnt1
);
    assign gnt1 = m1_req;
    assign gnt0 = !m1_req;
endmodule
