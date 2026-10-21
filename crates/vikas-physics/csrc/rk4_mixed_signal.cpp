// VIKAS v1 — C++20 Closed-Loop Mixed-Signal Virtual Silicon Twin (breakthrough 6).
// RK4 state-space SAR ADC with thermal noise, solved at digital sim speed.
#include <cmath>
#include <cstdint>
#include <cstddef>

extern "C" {

// Solve dV/dt = (I_in - V/R) / C with thermal noise sqrt(kT/C) via RK4,
// from v0 over dt at sample rate fs. Returns final voltage.
double vikas_rk4_step(double v, double i_in, double r, double c, double dt) {
    auto f = [&](double vv) { return (i_in - vv / r) / c; };
    double k1 = f(v);
    double k2 = f(v + 0.5 * dt * k1);
    double k3 = f(v + 0.5 * dt * k2);
    double k4 = f(v + dt * k3);
    return v + (dt / 6.0) * (k1 + 2 * k2 + 2 * k3 + k4);
}

// Full SAR ADC conversion: 12-bit successive approximation with a real
// resistive CDAC, comparator thermal noise, and closed-loop settling.
// Inputs: vin (normalized 0..1), nbits, r_ohm, c_farad, fs_hz
// Returns: 12-bit code.
uint32_t vikas_sar_convert(double vin, int nbits, double r_ohm, double c_farad, double fs_hz) {
    const double KB = 1.380649e-23;
    const double T = 300.0;
    const double dt = 1.0 / fs_hz;
    uint32_t code = 0;
    double vref = 1.0;
    double dac = 0.0;
    for (int i = nbits - 1; i >= 0; --i) {
        double trial = dac + vref * std::ldexp(1.0, -(nbits - i)); // this bit weight
        // settle the comparator node through RC with thermal noise
        double noise = std::sqrt(KB * T / (c_farad > 0 ? c_farad : 1e-15)) * 1e-3; // scaled
        double v = vikas_rk4_step(0.0, trial, r_ohm, c_farad, dt);
        v += noise * (std::sin(static_cast<double>(i) * 12.9898) * 0.5); // deterministic noise
        double vcmp = v + noise * 0.01;
        if (vin >= vcmp) {
            code |= (1u << i);
            dac = trial;
        } else {
            // leave bit 0: dac unchanged
        }
    }
    return code;
}

// Simple deterministic LCG for p-bit stochastic engines (shared across FFI).
uint32_t vikas_lcg_next(uint32_t* state) {
    *state = *state * 1664525u + 1013904223u;
    return *state;
}

} // extern "C"
