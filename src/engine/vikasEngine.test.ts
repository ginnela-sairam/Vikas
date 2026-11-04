import test from 'node:test';
import assert from 'node:assert/strict';
import {
  analyzeLatches,
  applyLatchAutoPatch,
  buildOpenKdb,
  computeSignoffScore,
  computeSwitchingPower,
  egraphOptimize,
  formatSignalValue,
  propagateTaintDye,
  runMutationTesting,
  runSarAdcRk4Cosim,
  simulateVerilogSource,
  solveCounterfactual,
  sweepSubthresholdCorners,
} from './vikasEngine';

test('Breakthrough 13: 3-Layer Inferred Latch Detection & Auto-Patcher', () => {
  const trapSv = `
module demo_latch_trap(input logic sel, input logic [7:0] d, output logic [7:0] q);
  always_comb begin
    if (sel) q = d;
  end
endmodule
`;
  const latches = analyzeLatches([{ path: 'demo_latch_trap.sv', content: trapSv }]);
  assert.equal(latches.length, 1);
  assert.equal(latches[0].signal, 'q');
  assert.equal(latches[0].enable_condition, 'sel');

  const { patched, count } = applyLatchAutoPatch(trapSv);
  assert.equal(count, 1);
  const afterLatches = analyzeLatches([{ path: 'demo_latch_trap.sv', content: patched }]);
  assert.equal(afterLatches.length, 0, 'Patched RTL must be 100% latch-clean');
});

test('Open-KDB CSR Graph, Per-Signal Driver Extraction & Counterfactual Solver', () => {
  const aluSv = `
module nova_alu(
  input logic clk,
  input logic rst_n,
  input logic [15:0] a,
  input logic [15:0] b,
  input logic [1:0] op,
  output logic [15:0] y,
  output logic zflag
);
  assign zflag = (y == 16'd0);
  always_ff @(posedge clk) begin
    if (!rst_n) y <= 16'd0;
    else y <= a + b;
  end
endmodule
`;
  const kdb = buildOpenKdb([{ path: 'examples/nova_soc/rtl/core/nova_alu.sv', content: aluSv }]);
  assert.equal(kdb.top, 'nova_alu');
  const zDrv = kdb.drivers.find((d) => d.signal === 'zflag');
  assert.ok(zDrv, 'zflag driver must exist');
  assert.equal(zDrv.file, 'nova_alu.sv');
  assert.ok(zDrv.rhs_text.includes('y =='));

  const yDrv = kdb.drivers.find((d) => d.signal === 'y');
  assert.ok(yDrv, 'y driver must exist');
  assert.equal(yDrv.rhs_text, 'a + b', 'always_ff driver must capture non-reset RHS expression');

  const sim = simulateVerilogSource([{ path: 'nova_alu.sv', content: aluSv }], 200);
  const cf = solveCounterfactual(kdb, sim.signals, sim.steps, 'zflag', 50, '1');
  assert.ok(cf.blocking_inputs.length >= 1);
  assert.ok(cf.smt_clause.includes('assert'));

  const taint = propagateTaintDye(kdb, 'y', 'zflag');
  assert.equal(taint.reachable, true);
  assert.ok(taint.dye_tag_hex.startsWith('0x'));
});

test('Breakthroughs 2, 3, 6, 7, 15: Switching Power (fJ), Sub-Threshold 0.12V/77K, RK4 SAR ADC & E-Graph', () => {
  const sim = simulateVerilogSource(
    [
      {
        path: 'top.sv',
        content: `
module top(input logic clk, input logic rst_n, output logic [7:0] count);
  always_ff @(posedge clk) count <= count + 1;
endmodule`,
      },
    ],
    400
  );
  const pwr = computeSwitchingPower(sim.signals, sim.steps, 0.8, 1.4, 100);
  assert.ok(pwr.total_energy_fj > 10, 'Total switching energy in fJ must be non-zero');
  assert.ok(pwr.hotspots.length >= 2);

  const corners = sweepSubthresholdCorners([0.12, 0.4, 0.8], 77);
  assert.equal(corners.length, 3);
  assert.ok(corners[0].delay_mult > corners[2].delay_mult);
  assert.ok(corners[0].verdict.includes('CRYO'));

  const sar = runSarAdcRk4Cosim(12, 0.8, 300);
  assert.equal(sar.bits, 12);
  assert.ok(sar.enob >= 9.5, `Expected ENOB >= 9.5, got ${sar.enob}`);
  assert.equal(sar.monotonic, true);

  const eg = egraphOptimize('(a && a) || (b ^ b)');
  assert.ok(eg.fo4_after <= eg.fo4_before);
  assert.ok(eg.applied_rules.length >= 1);
});

test('Breakthroughs 14 & 16: RTL Mutation Testing & One-Command Signoff Score', () => {
  const files = [
    {
      path: 'alu_core.sv',
      content: `
module alu_core(input logic [7:0] a, input logic [7:0] b, output logic [7:0] sum, output logic eq);
  assign sum = a + b;
  assign eq = (a == b);
endmodule`,
    },
  ];
  const mut = runMutationTesting(files, 'alu_core', 8, 70);
  assert.ok(mut.total_mutants >= 2);
  assert.equal(mut.passed, true);

  const sim = simulateVerilogSource(files, 200);
  const signoff = computeSignoffScore(sim.kdb, sim.signals, sim.steps, mut);
  assert.ok(signoff.total_score >= 85);
  assert.equal(signoff.verdict, 'SIGNOFF READY');
  assert.equal(signoff.exit_code, 0);
});

test('Radix Formatting for 4-State Logic (hex, bin, dec, signed, oct, ascii)', () => {
  assert.equal(formatSignalValue(255, 8, 'hex'), '0xff');
  assert.equal(formatSignalValue(255, 8, 'dec'), '255');
  assert.equal(formatSignalValue(255, 8, 'signed'), '-1');
  assert.equal(formatSignalValue(65, 8, 'ascii'), 'A');
  assert.equal(formatSignalValue('10xz1100', 8, 'hex'), '0xxc');
});
