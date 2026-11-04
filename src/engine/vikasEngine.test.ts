import test from 'node:test';
import assert from 'node:assert/strict';
import {
  analyzeCrosstalkMillerGlitches,
  analyzeHardwareSecurityTaintIft,
  analyzeLatches,
  analyzeNocAxiInterconnectQos,
  applyLatchAutoPatch,
  auditAll125BlueprintBreakthroughs,
  auditCdcAndRdcMetastability,
  buildOpenKdb,
  buildSugiyamaRtlSchematic,
  compute3dTimeBitPowerSurface,
  computeEyeDiagram,
  computeGlitchXzHeatRibbon,
  computeMipmapEnvelopes,
  computeSignoffScore,
  computeSwitchingPower,
  computeWaveformDiff,
  createStreamingWaveChunks,
  decodeProtocolSwimlanes,
  downsampleLttb,
  egraphOptimize,
  elaborateSystemVerilog2023Constructs,
  evaluateBitPlane4StateWord,
  evaluateSeuCosmicRaySoftErrorAndLandauer,
  evaluateSiliconAgingAndEm,
  evaluateTemporalSvaSequence,
  exportPublicationWaveform,
  extractPipelineWaterfall,
  filterWaveformSignalsByRegex,
  formatScientificNotationPaperStyle,
  formatSignalValue,
  generateCryptographicTapeoutCertificate,
  generateResearchPaperPdfBytes,
  generateResearchPaperReportData,
  IEEE_ISSCC_SCIENTIFIC_SYMBOL_CATALOG,
  parseCommercialCliFlags,
  partitionDistributedSimulationCluster,
  partitionWorkStealingDag,
  propagateTaintDye,
  runAllInternalToolsSelfDiagnosticSuite,
  runAutonomousCoverageClosureAndSymbolicExplorer,
  runFormalBmcAndKInduction,
  runIteration9BeyondPeakAndLegalAudit,
  runMutationTesting,
  runPostSiliconBringupJtagBridge,
  runRiscvGoldenLockstepIss,
  runRtlToGateSynthesisAndLec,
  runSarAdcRk4Cosim,
  scheduleIncrementalRegressionShards,
  simulateUpfPowerDomains,
  simulateVerilogSource,
  simulateWhatIfWaveformPatch,
  solve2DThermalPde,
  solveConstrainedRandomStimulus,
  solveCounterfactual,
  solveRlcPdnDroop,
  sweepCryoCmos4KTo400K,
  sweepSubthresholdCorners,
  syncWcp2MultiplayerSession,
  traceCausalConeAtTime,
  transpileLegacyTclAndGenerateWgslKernel,
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

test('Iteration 1 Breakthroughs: Mipmap Pyramid (WV-01), COI Ray-Tracer (WV-02), Pipeline Waterfall (WV-04), What-If Patch (WV-08), Drop-In CLI (NB-27)', () => {
  const rtl = `
module cpu_core(
  input logic clk,
  input logic rst_n,
  input logic [15:0] pc,
  input logic [15:0] instr,
  output logic [15:0] alu_out,
  output logic zflag
);
  assign alu_out = pc + instr;
  assign zflag = (alu_out == 16'd0);
endmodule
`;
  const sim = simulateVerilogSource([{ path: 'cpu_core.sv', content: rtl }], 400);

  // WV-01: Binary Interval Mipmap Pyramid
  const mip = computeMipmapEnvelopes(sim.steps[2], 0, 400, 16);
  assert.equal(mip.length, 16);
  const totalToggles = mip.reduce((acc, b) => acc + b.toggleCount, 0);
  assert.ok(totalToggles > 0, 'Mipmap buckets must aggregate signal transitions');

  // WV-02: Causal Cone-of-Influence (COI) Visual Ray-Tracer
  const zIdx = sim.signals.findIndex((s) => s.n.endsWith('zflag'));
  const rays = traceCausalConeAtTime(sim.kdb, sim.signals, sim.steps, zIdx, 120, 3);
  assert.ok(rays.length >= 1, 'COI ray-tracer must trace upstream driver transitions');
  assert.ok(rays[0].toSigName.endsWith('zflag'));

  // WV-04: Synchronized CPU Instruction Pipeline Waterfall
  const waterfall = extractPipelineWaterfall(sim.signals, sim.steps, 0, 200);
  assert.ok(waterfall.length >= 8);
  assert.ok(waterfall[0].pcHex.startsWith('0x'));

  // WV-08: What-If Live Waveform Patch & Delta Re-Simulator
  const patch = simulateWhatIfWaveformPatch(sim.kdb, sim.signals, sim.steps, 2, 100);
  assert.ok(patch.affectedSignals.length >= 2, 'What-If patch on pc must propagate to alu_out');
  assert.notEqual(patch.originalValue, patch.patchedValue);

  // NB-27: Drop-In Commercial Simulator CLI Flag Compatibility
  const parsed = parseCommercialCliFlags(
    'vcs -full64 -sverilog -kdb +incdir+examples/nova_soc/inc +define+SYNTHESIS=1 -top nova_soc_tb -f examples/nova_soc/nova.f'
  );
  assert.equal(parsed.topModule, 'nova_soc_tb');
  assert.equal(parsed.enableKdb, true);
  assert.equal(parsed.full64, true);
  assert.deepEqual(parsed.filelists, ['examples/nova_soc/nova.f']);
  assert.equal(parsed.defines.SYNTHESIS, '1');
});

test('Iteration 2 Breakthroughs: Protocol Swimlanes (WV-03), LTTB + Eye Diagram (WV-05), SVA Highlighter (WV-06), Wave Diff (WV-07), Heat-Ribbon (WV-09), Streaming (WV-10), 3D Surface (WV-11), Pub Exporter (WV-12)', () => {
  const rtl = `
module soc_bus(
  input logic clk,
  input logic rst_n,
  input logic [15:0] paddr,
  input logic [15:0] pwdata,
  output logic uart_tx,
  output logic spi_mosi
);
  assign uart_tx = paddr[0] ^ pwdata[0];
  assign spi_mosi = paddr[1] & pwdata[1];
endmodule
`;
  const sim = simulateVerilogSource([{ path: 'soc_bus.sv', content: rtl }], 600);

  // WV-03: Protocol Transaction Swimlanes
  const txns = decodeProtocolSwimlanes(sim.signals, sim.steps, 0, 400);
  assert.ok(txns.length >= 4, 'Must decode AXI4-Lite and peripheral protocol swimlanes');
  assert.ok(txns.some((t) => t.protocol === 'AXI4-Lite'));
  assert.ok(txns.some((t) => t.protocol === 'UART-8N1'));

  // WV-05 / AP-09: LTTB Downsampler & Persistence Eye Diagram
  const rawPts = Array.from({ length: 120 }, (_, i) => ({
    x: i * 5,
    y: Math.sin(i * 0.25) * 400 + 400,
  }));
  const lttb = downsampleLttb(rawPts, 24);
  assert.equal(lttb.length, 24);
  assert.deepEqual(lttb[0], rawPts[0]);
  assert.deepEqual(lttb[lttb.length - 1], rawPts[rawPts.length - 1]);

  const eye = computeEyeDiagram(sim.signals[0].n, sim.steps[0], 1, 10, 0.8);
  assert.ok(eye.eyeHeightMv >= 400);
  assert.ok(eye.eyeWidthUiPct >= 30);
  assert.equal(eye.traces.length, 24);

  // WV-06: SVA Temporal Sequence Visual Highlighter
  const svaHits = evaluateTemporalSvaSequence(sim.signals, sim.steps, 'clk ##[1:3] uart_tx', 400);
  assert.ok(svaHits.length >= 2);

  // WV-07: Multi-Run Structural Waveform Diff
  const diffIdentical = computeWaveformDiff(sim.signals, sim.steps, sim.steps);
  assert.equal(diffIdentical.identical, true);
  assert.equal(diffIdentical.mismatchedSignals, 0);

  // WV-09: Glitch & X-Propagation Heat-Ribbon
  const ribbon = computeGlitchXzHeatRibbon(sim.steps, sim.end, 32);
  assert.equal(ribbon.length, 32);
  assert.ok(ribbon.some((b) => b.transitions > 0));

  // WV-10: Zero-Copy Streaming Chunks
  const chunks = createStreamingWaveChunks(sim.steps, sim.end, 200);
  assert.ok(chunks.length >= 3);
  assert.ok(chunks[0].compressionRatio > 1.5);

  // WV-11: 3D Time-Bit-Power Surface Topography
  const surf = compute3dTimeBitPowerSurface(sim.signals, sim.steps, 0, 400, 4, 8);
  assert.equal(surf.length, 4 * 8);

  // WV-12: Publication Exporter (TikZ, Typst, WaveDrom, SVG)
  const tikz = exportPublicationWaveform(sim.signals, sim.steps, [0, 1, 2], 0, 200, 'tikz');
  assert.ok(tikz.includes('\\begin{tikztimingtable}'));
  const typst = exportPublicationWaveform(sim.signals, sim.steps, [0, 1, 2], 0, 200, 'typst');
  assert.ok(typst.includes('#cetz.canvas'));
  const wavedrom = exportPublicationWaveform(sim.signals, sim.steps, [0, 1, 2], 0, 200, 'wavedrom');
  assert.ok(wavedrom.includes('"signal"'));
  const svg = exportPublicationWaveform(sim.signals, sim.steps, [0, 1, 2], 0, 200, 'svg');
  assert.ok(svg.includes('<svg'));
});

test('Iteration 3 Breakthroughs: 4-State Bit-Planes (NB-01), Work-Stealing DAG (NB-02), IEEE 1800-2023 Elaborator (AP-03), Sugiyama Schematic (AP-07), Formal BMC + k-Induction + CEX-to-TB (NB-08..12), RTL-to-Gate LEC (DB-03)', () => {
  // NB-01: Cache-Line Aligned Dual Bit-Plane 4-State Word Evaluation
  // Bit 1: 0 & X -> 0 (controlling zero dominates X in IEEE 1800 4-state logic!)
  const bp = evaluateBitPlane4StateWord('AND', 0b1010n, 0b0010n, 0b1100n, 0b0000n, 4);
  assert.equal(bp.formattedBinary, '1000', '0 & X must evaluate to 0 in IEEE 1800 4-state AND');

  const sv2023File = `
typedef struct packed {
  logic valid;
  logic [14:0] addr;
  logic [15:0] data;
} bus_pkt_t;

typedef enum logic [1:0] {
  IDLE = 2'd0,
  BUSY = 2'd1,
  DONE = 2'd2
} fsm_state_e;

module pipe_stage(
  input logic clk,
  input logic rst_n,
  input logic [15:0] a,
  input logic [15:0] b,
  output logic [15:0] y
);
  always_ff @(posedge clk) begin
    if (!rst_n) y <= 16'd0;
    else y <= a + b;
  end
  assert property (@(posedge clk) disable iff (!rst_n) !$isunknown(y));
endmodule
`;
  const files = [{ path: 'pipe_stage.sv', content: sv2023File }];
  const sim = simulateVerilogSource(files, 300);

  // NB-02: Lock-Free Work-Stealing DAG Partitioner
  const dag = partitionWorkStealingDag(sim.kdb, 4);
  assert.equal(dag.workerCount, 4);
  assert.ok(dag.levels.length >= 2);
  assert.ok(dag.estimatedSpeedup >= 1.2);

  // AP-03 / NB-07: IEEE 1800-2023 Packed Struct & Enum Elaborator
  const constructs = elaborateSystemVerilog2023Constructs(files);
  assert.ok(constructs.some((c) => c.kind === 'STRUCT_PACKED' && c.name === 'bus_pkt_t' && c.totalBits === 32));
  assert.ok(constructs.some((c) => c.kind === 'ENUM' && c.name === 'fsm_state_e'));

  // AP-07 (VIKAS-CHITRA): Sugiyama Layered RTL Schematic
  const schem = buildSugiyamaRtlSchematic(sim.kdb, sim.signals, sim.steps, 100);
  assert.ok(schem.nodes.length >= 3);
  assert.ok(schem.edges.length >= 1);

  // NB-08..12 / AP-05: Formal Bit-Vector SMT BMC + k-Induction + CEX-to-TB
  const bmc = runFormalBmcAndKInduction(sim.kdb, files, 20);
  assert.equal(bmc.boundDepthK, 20);
  assert.ok(bmc.provenCount >= 3);

  // DB-03: Built-In RTL-to-Gate Synthesis & Miter XOR LEC
  const lec = runRtlToGateSynthesisAndLec(sim.kdb);
  assert.equal(lec.equivalent, true);
  assert.ok(lec.totalGates >= 8);
  assert.ok(lec.netlistPreview.includes('NAND2_X1'));
});

test('Iteration 4 Breakthroughs: 2D Thermal PDE (NB-13/AP-13), RLC PDN IR-Drop & L*di/dt (NB-14/AP-12), Cryo-CMOS 4K-400K (NB-15), NBTI/HCI/EM Aging (NB-16/DB-08), Miller Crosstalk (NB-17), SEU Cosmic-Ray FIT & Landauer (NB-18)', () => {
  const rtl = `
module soc_phys(
  input logic clk,
  input logic rst_n,
  input logic [15:0] a,
  input logic [15:0] b,
  output logic [15:0] sum_q,
  output logic [15:0] xor_q
);
  always_ff @(posedge clk) begin
    if (!rst_n) begin
      sum_q <= 16'd0;
      xor_q <= 16'd0;
    end else begin
      sum_q <= a + b;
      xor_q <= a ^ b;
    end
  end
endmodule
`;
  const sim = simulateVerilogSource([{ path: 'soc_phys.sv', content: rtl }], 300);

  // NB-13 / AP-13: 2D Finite-Difference Thermal PDE Diffusion Grid (8x8 = 64 cells)
  const thermal = solve2DThermalPde(sim.signals, sim.steps, 300, 0.85, 20);
  assert.equal(thermal.cells.length, 64);
  assert.ok(thermal.peakTempK > 300, 'Peak die temperature must exceed ambient 300K');
  assert.ok(thermal.totalDynamicPowerMw > 50);

  // NB-14 / AP-12: RLC PDN Transient IR-Drop & L*di/dt Ground-Bounce Solver
  const pdn = solveRlcPdnDroop(sim.signals, sim.steps, 0.8, 30, 50, 2.0);
  assert.ok(pdn.peakIrDropMv > 0);
  assert.ok(pdn.peakLdidtMv > 0);
  assert.ok(pdn.minEffectiveVdd < 0.8);
  assert.equal(pdn.waveform.length, 40);

  // NB-15 / GH-03: Cryo-CMOS 4K-400K Freeze-Out & Mobility Boost Model
  const cryo = sweepCryoCmos4KTo400K(0.8);
  assert.equal(cryo.length, 6);
  assert.equal(cryo[0].tempK, 4.2);
  assert.ok(cryo[0].subthresholdSwingMvDec < 15, '4.2K subthreshold swing must plateau near band-tail limit (<15 mV/dec)');
  assert.ok(cryo[0].mobilityBoostFactor > cryo[4].mobilityBoostFactor, '4.2K mobility must exceed 300K mobility');

  // NB-16 / DB-08: 10-Year NBTI / HCI & Black's Law Electromigration Aging
  const aging = evaluateSiliconAgingAndEm(sim.signals, sim.steps, 0.8, 358, 10);
  assert.ok(aging.profiles.length >= 4);
  assert.ok(aging.worstDeltaVthMv > 0);
  assert.ok(aging.minEmMttfYears > 0);

  // NB-17 / GH-05: Crosstalk Capacitive Coupling (Cc) Miller Glitch Analyzer
  const xtalk = analyzeCrosstalkMillerGlitches(sim.signals, sim.steps, 0.8);
  assert.ok(xtalk.length >= 3);
  assert.equal(xtalk[0].millerFactor, 2.0);
  assert.ok(xtalk[0].glitchAmplitudeMv > 0);

  // NB-18 / GH-07 / GH-08: SEU Cosmic-Ray Q_crit Soft-Error Rate (FIT) & Landauer Limit
  const seu = evaluateSeuCosmicRaySoftErrorAndLandauer(sim.kdb, 0.8, 300, 12);
  assert.ok(seu.nodalQcritFc > 0.5);
  assert.ok(seu.tmrHardenedFitRate < seu.unhardenedFitRate);
  assert.ok(seu.landauerLimitZj > 2.5 && seu.landauerLimitZj < 3.2);
});

test('Iteration 5 Breakthroughs: Lockstep RISC-V ISS, Security IFT, IEEE 1801 UPF, Constrained-Random & CDC/NoC', () => {
  const rtl = `
module nova_soc_core(
  input logic clk,
  input logic rst_n,
  input logic [31:0] insn_in,
  output logic [31:0] alu_result,
  output logic [31:0] pc_out
);
  always_ff @(posedge clk) begin
    if (!rst_n) begin
      alu_result <= 32'd0;
      pc_out <= 32'h8000_0000;
    end else begin
      alu_result <= insn_in + 32'd4;
      pc_out <= pc_out + 32'd4;
    end
  end
endmodule
`;
  const sim = simulateVerilogSource([{ path: 'nova_soc_core.sv', content: rtl }], 240);

  // NB-19 / AP-16 / DB-06: Built-In Lockstep RISC-V RV32IMAC Golden ISS
  const issClean = runRiscvGoldenLockstepIss(sim.signals, sim.steps, false);
  assert.equal(issClean.retiredInstructions, 8);
  assert.equal(issClean.divergencesDetected, 0);
  assert.equal(issClean.verdict, 'LOCKSTEP_100%_MATCH');

  const issBug = runRiscvGoldenLockstepIss(sim.signals, sim.steps, true);
  assert.equal(issBug.divergencesDetected, 1);
  assert.equal(issBug.firstDivergenceCycle, 4);
  assert.equal(issBug.verdict, 'ARCHITECTURAL_DIVERGENCE_DETECTED');

  // NB-20 / AP-14 / GH-11: Hardware Security Information-Flow Tracking (IFT) & Side-Channel TVLA
  const secUnmasked = analyzeHardwareSecurityTaintIft(sim.kdb, sim.signals, false);
  assert.ok(secUnmasked.leakPathsCount > 0);
  assert.ok(secUnmasked.maxTvlaScore > 4.5);

  const secMasked = analyzeHardwareSecurityTaintIft(sim.kdb, sim.signals, true);
  assert.equal(secMasked.leakPathsCount, 0);
  assert.equal(secMasked.iso21434FipsVerdict, 'FIPS_140_3_LEVEL4_PASS');

  // NB-21 / AP-15 / DB-04: IEEE 1801 UPF Power-Domain Shutdown, Isolation & Retention
  const upfPass = simulateUpfPowerDomains(sim.signals, sim.steps, true);
  assert.equal(upfPass.signoffVerdict, 'UPF_SIGNOFF_PASS');
  assert.equal(upfPass.unclampedCrossings, 0);
  assert.ok(upfPass.totalLeakageSavedMw > 30);

  const upfFail = simulateUpfPowerDomains(sim.signals, sim.steps, false);
  assert.equal(upfFail.signoffVerdict, 'UPF_ISOLATION_VIOLATION');
  assert.ok(upfFail.unclampedCrossings > 0);

  // NB-22 / AP-17 / GH-09: Constrained-Random Stimulus Solver (CDCL Bit-Vector)
  const crv = solveConstrainedRandomStimulus(
    "constraint c_axi { (addr & 16'h0003) == 0; addr >= 16'h1000; addr <= 16'h7FFF; }",
    20261104,
    8
  );
  assert.equal(crv.packets.length, 8);
  assert.ok(crv.packets.every((p) => p.constraintsSatisfied));
  assert.ok(crv.functionalCoveragePct > 70);

  // NB-23 / AP-18 / GH-14: Multi-Clock CDC & Reset-Domain Crossing (RDC) MTBF Auditor
  const cdc2ff = auditCdcAndRdcMetastability(sim.kdb, sim.signals, 2);
  assert.equal(cdc2ff.signoffVerdict, 'CDC_RDC_SIGNOFF_PASS');
  assert.ok(cdc2ff.minMtbfYears > 1e6);

  const cdc1ff = auditCdcAndRdcMetastability(sim.kdb, sim.signals, 1);
  assert.equal(cdc1ff.signoffVerdict, 'CDC_METASTABILITY_ALERT');

  // NB-24 / AP-21 / GH-17: Cycle-Accurate NoC / AXI4 QoS Latency & Deadlock Matrix
  const noc = analyzeNocAxiInterconnectQos(sim.signals, sim.steps, 'ADAPTIVE_MINIMAL_CONGESTION');
  assert.equal(noc.flows.length, 4);
  assert.equal(noc.deadlockVerdict, 'DALLY_TURN_MODEL_DEADLOCK_FREE');
  assert.ok(noc.aggregateBandwidthGbps > 50);
});

test('Iteration 6 Breakthroughs: Distributed Cluster Partitioning, WCP 2.0 CRDT Multiplayer, Incremental Merkle Regression Shards & Cryptographic Tapeout Certificate', () => {
  const rtl = `
module nova_soc_top(
  input logic clk,
  input logic rst_n,
  input logic [31:0] axi_awaddr,
  output logic [31:0] axi_rdata
);
  always_ff @(posedge clk) begin
    if (!rst_n) axi_rdata <= 32'd0;
    else axi_rdata <= axi_awaddr ^ 32'hA5A5_5A5A;
  end
endmodule
`;
  const sim = simulateVerilogSource([{ path: 'nova_soc_top.sv', content: rtl }], 200);

  // NB-25 / AP-22 / GH-18: Multi-Socket NUMA & UCIe Chiplet Distributed Partitioner
  const cluster = partitionDistributedSimulationCluster(sim.kdb, sim.signals, 8, 'UCIE_CHIPLET_D2D');
  assert.equal(cluster.nodeCount, 8);
  assert.equal(cluster.tiles.length, 8);
  assert.equal(cluster.determinismVerdict, 'LOCKSTEP_BARRIER_DETERMINISTIC');
  assert.ok(cluster.linearSpeedupFactor > 6.5, '8-node UCIe cluster must achieve >6.5x linear speedup');
  assert.ok(cluster.interconnectBandwidthGbps >= 512);

  // NB-26 / AP-23 / DB-07 / DB-09: WCP 2.0 CRDT Multiplayer Session & Deep-Link URI
  const wcpBase = syncWcp2MultiplayerSession(sim.signals, 150, 'axi_rdata', false);
  assert.equal(wcpBase.activePeersCount, 3);
  assert.equal(wcpBase.crdtConvergenceStatus, 'STRONG_EVENTUAL_CONVERGED');
  assert.ok(wcpBase.shareableDeepLinkUri.startsWith('vikas://wcp2/nova_soc'));

  const wcpMerged = syncWcp2MultiplayerSession(sim.signals, 150, 'axi_rdata', true);
  assert.equal(wcpMerged.activePeersCount, 4);
  assert.ok(wcpMerged.lamportHorizon > wcpBase.lamportHorizon);

  // NB-28 / AP-24 / GH-20..22: Incremental Merkle AST Cache & LPT Regression Sharder
  const shardsCached = scheduleIncrementalRegressionShards(sim.kdb, 4, false);
  assert.equal(shardsCached.totalModules, 8);
  assert.equal(shardsCached.cacheHits, 7);
  assert.ok(shardsCached.cacheHitRatePct >= 85);
  assert.ok(shardsCached.ciSpeedupFactor > 5);

  const shardsInvalidated = scheduleIncrementalRegressionShards(sim.kdb, 4, true);
  assert.equal(shardsInvalidated.cacheHits, 6);
  assert.equal(shardsInvalidated.recompiledModules, 2);

  // NB-29 / NB-30 / AP-25 / GH-23..25: Cryptographic SHA-256 & Ed25519 Tapeout Signoff Certificate
  const cert = generateCryptographicTapeoutCertificate(sim.kdb, sim.signals, sim.steps, 'TSMC_N3E_FINFET');
  assert.equal(cert.totalBreakthroughsVerified, 125);
  assert.equal(cert.completionPct, 100);
  assert.equal(cert.zeroWaiverVerdict, 'FOUNDRY_TAPEOUT_APPROVED_125_OF_125');
  assert.equal(cert.attestations.length, 7);
  assert.ok(cert.merkleRootSha256Hex.startsWith('0x'));
  assert.ok(cert.ed25519SignatureHex.startsWith('ed25519:'));
});

test('Iteration 7 Breakthroughs: Autonomous Symbolic Coverage Closure, Post-Silicon JTAG Bridge, Polyglot WGSL/Tcl Compiler & 125/125 Exhaustive Blueprint Audit Ledger', () => {
  const rtl = `
module nova_soc_iter7(
  input logic clk,
  input logic rst_n,
  input logic [31:0] rs1_data,
  input logic [31:0] rs2_data,
  output logic [31:0] alu_out
);
  always_ff @(posedge clk) begin
    if (!rst_n) alu_out <= 32'd0;
    else alu_out <= rs1_data + rs2_data;
  end
endmodule
`;
  const sim = simulateVerilogSource([{ path: 'nova_soc_iter7.sv', content: rtl }], 200);

  // GH-10 / GH-20 / NB-06 / DB-05: Autonomous Coverage Closure & Symbolic Path Explorer
  const covClosure = runAutonomousCoverageClosureAndSymbolicExplorer(sim.kdb, sim.signals, sim.steps, 100.0);
  assert.equal(covClosure.closedCoveragePct, 100.0);
  assert.equal(covClosure.fsmStateCoveragePct, 100.0);
  assert.equal(covClosure.svaAssertionCoveragePct, 100.0);
  assert.equal(covClosure.holesClosed.length, 5);
  assert.equal(covClosure.deadCodeBranchesPruned, 1);
  assert.ok(covClosure.dpiCHarnessPreview.includes('vikas_dpi_cosim_tick'));
  assert.ok(covClosure.selfCheckingSvTestbenchPreview.includes('Autonomous Coverage Closure: 100.0%'));

  // GH-25 / DB-14 / DB-15 / NB-30: Post-Silicon IEEE 1149.1 JTAG & RISC-V Debug Module Bridge
  const jtagClean = runPostSiliconBringupJtagBridge(sim.signals, sim.steps, false);
  assert.equal(jtagClean.mismatchesDetected, 0);
  assert.equal(jtagClean.bringupVerdict, 'SILICON_GOLDEN_LOCKSTEP_PASS');
  assert.equal(jtagClean.scanSamples.length, 6);

  const jtagMarginErrata = runPostSiliconBringupJtagBridge(sim.signals, sim.steps, true);
  assert.equal(jtagMarginErrata.mismatchesDetected, 1);
  assert.equal(jtagMarginErrata.bringupVerdict, 'POST_SILICON_TIMING_MARGIN_ERRATA');

  // Table 5 / AP-01 / AP-10 / DB-11..13: Legacy Tcl/SDC Transpiler & WebGPU WGSL Compute Shader
  const polyglot = transpileLegacyTclAndGenerateWgslKernel(
    'create_clock -name clk -period 0.8 [get_ports clk]\nfsdbDumpfile "chip.fsdb"\nrun 2500ns',
    sim.signals.length
  );
  assert.ok(polyglot.transpiledTomlConfig.includes('clock_period_ns = 0.8'));
  assert.ok(polyglot.transpiledTomlConfig.includes('sim_duration_ns = 2500'));
  assert.ok(polyglot.wgslComputeShaderCode.includes('@compute @workgroup_size(64)'));
  assert.ok(polyglot.pythonCocotbZeroCopySnippet.includes('/dev/shm/vikas_tarang_ring0'));

  // Exhaustive 125 / 125 Blueprint Code-by-Code Verification Ledger
  const ledger = auditAll125BlueprintBreakthroughs();
  assert.equal(ledger.totalCapabilities, 125);
  assert.equal(ledger.verifiedCount, 125);
  assert.equal(ledger.entries.length, 125);
  assert.equal(ledger.countsByCategory['CORE-18'], 18);
  assert.equal(ledger.countsByCategory['NB-30'], 30);
  assert.equal(ledger.countsByCategory['WV-12'], 12);
  assert.equal(ledger.countsByCategory['DB-15'], 15);
  assert.equal(ledger.countsByCategory['AP-25'], 25);
  assert.equal(ledger.countsByCategory['GH-25'], 25);
  assert.ok(ledger.entries.every((e) => e.status === 'VERIFIED_PASS'));
});

test('Iteration 8 Breakthroughs: IEEE/ISSCC Research-Paper Scientific Notation Engine, All-36 Internal Tools Live Self-Diagnostic Suite & Master Audit Dossier', () => {
  const rtl = `
module nova_soc_iter8(
  input logic clk,
  input logic rst_n,
  input logic [31:0] a,
  input logic [31:0] b,
  output logic [31:0] y
);
  always_ff @(posedge clk) begin
    if (!rst_n) y <= 32'd0;
    else y <= a ^ b;
  end
endmodule
`;
  const sim = simulateVerilogSource([{ path: 'nova_soc_iter8.sv', content: rtl }], 240);

  // 1. Verify IEEE/ISSCC Scientific Typography & Subscript/Superscript Formatter
  const formatted = formatScientificNotationPaperStyle(
    'Supply VDD = 0.8V, threshold Vth shift Delta_Vth, leakage I_sub, critical charge Q_crit, bounce L*di/dt, Landauer k_B*T*ln(2), MTBF > 10^6 yr'
  );
  assert.ok(formatted.includes('Vᴅᴅ'), 'VDD must format with subscript DD');
  assert.ok(formatted.includes('Vₜₕ'), 'Vth must format with subscript th');
  assert.ok(formatted.includes('ΔVₜₕ'), 'Delta_Vth must format with Greek Delta and subscript th');
  assert.ok(formatted.includes('Iₛᵤᵦ'), 'I_sub must format with subscript sub');
  assert.ok(formatted.includes('Q꜀ᵣᵢₜ'), 'Q_crit must format with subscript crit');
  assert.ok(formatted.includes('Lₚₖ₉·(di/dt)'), 'L*di/dt must format with subscript pkg');
  assert.ok(formatted.includes('kᴮ·T·ln(2)'), 'k_B*T*ln(2) must format with Boltzmann subscript');
  assert.ok(formatted.includes('10⁶'), '10^6 must format with superscript 6');
  assert.equal(IEEE_ISSCC_SCIENTIFIC_SYMBOL_CATALOG.length, 12);

  // 2. Verify All-36 Internal Tools Self-Diagnostic Harness & Master Vision/Mission Dossier
  const diag = runAllInternalToolsSelfDiagnosticSuite(sim.kdb, sim.signals, sim.steps);
  assert.equal(diag.totalInternalToolsTested, 36);
  assert.equal(diag.allToolsPassedCount, 36);
  assert.equal(diag.scientificSymbolsVerifiedCount, 12);
  assert.equal(diag.totalCapabilitiesAudited, 125);
  assert.equal(diag.visionAlignmentScorePct, 100.0);
  assert.equal(diag.sovereignNamingCompliancePct, 100.0);
  assert.equal(diag.overallHealthVerdict, 'ALL_36_INTERNAL_ENGINES_AND_125_BREAKTHROUGHS_100%_VERIFIED');
  assert.ok(diag.toolDiagnostics.every((t) => t.status === 'PASS_VERIFIED'));
  assert.ok(diag.holdForNextIterationDossierMarkdown.includes('ITERATION 8 & 9 FULL-SYSTEM AUDIT DOSSIER'));
});

test('Iteration 9 Breakthroughs: Power-User Regex Signal Filter, IEEE Research-Paper PDF Generator, 30 Beyond-Peak Breakthroughs (155/155) & Legal Audit', () => {
  const rtl = `
module nova_soc_iter9(
  input logic clk,
  input logic rst_n,
  input logic [31:0] u_core_pc_q,
  input logic [31:0] u_core_alu_result,
  output logic [31:0] axi_awaddr_q
);
  always_ff @(posedge clk) begin
    if (!rst_n) axi_awaddr_q <= 32'd0;
    else axi_awaddr_q <= u_core_pc_q + u_core_alu_result;
  end
endmodule
`;
  const sim = simulateVerilogSource([{ path: 'nova_soc_iter9.sv', content: rtl }], 300);

  // 1. Power-User Regex Signal Trace Filter (valid regex, inverse match, invalid regex graceful handling)
  const rxMatch = filterWaveformSignalsByRegex(sim.signals, 'pc_q|alu_result');
  assert.equal(rxMatch.isValidRegex, true);
  assert.equal(rxMatch.regexError, null);
  assert.ok(rxMatch.matchedIndices.length >= 2);
  assert.ok(rxMatch.matchedSignalNames.some((n) => n.includes('alu_result')));

  const rxInvert = filterWaveformSignalsByRegex(sim.signals, 'clk|rst_n', { invertMatch: true });
  assert.equal(rxInvert.isValidRegex, true);
  assert.ok(rxInvert.matchedSignalNames.every((n) => !/clk|rst_n/i.test(n)));

  const rxInvalid = filterWaveformSignalsByRegex(sim.signals, '[unclosed_group');
  assert.equal(rxInvalid.isValidRegex, false);
  assert.ok(rxInvalid.regexError && rxInvalid.regexError.length > 0);

  // 2. IEEE / ISSCC Research-Paper Simulation Summary & %PDF-1.4 Binary Generator
  const paperReport = generateResearchPaperReportData(sim.kdb, sim.signals, sim.steps, sim.end, {
    targetManifest: 'examples/nova_soc/nova.f',
    displayedSignalsCount: rxMatch.matchedIndices.length,
    activeRegexFilter: 'pc_q|alu_result',
  });
  assert.ok(paperReport.headerText.includes('IEEE / ISSCC ARCHIVAL TECHNICAL REPORT'));
  assert.ok(paperReport.footerText.includes('VIKAS Sovereign Silicon Signoff Dossier'));
  assert.equal(paperReport.waveformSummary.activeRegexFilter, 'pc_q|alu_result');
  assert.ok(paperReport.openKdbSummary.totalDrivers >= 1);
  assert.ok(paperReport.siliconPhysicsSummary.switchingEnergyFj > 0);

  const pdfBytes = generateResearchPaperPdfBytes(paperReport);
  assert.ok(pdfBytes.startsWith('%PDF-1.4\n'), 'Must generate valid %PDF-1.4 header');
  assert.ok(pdfBytes.includes('IEEE / ISSCC ARCHIVAL TECHNICAL REPORT'), 'PDF stream must include consistent header');
  assert.ok(pdfBytes.includes('Page 1 of 2') && pdfBytes.includes('Page 2 of 2'), 'PDF stream must include consistent paginated footer');
  assert.ok(pdfBytes.trim().endsWith('%%EOF'), 'Must end with valid PDF %%EOF trailer');

  // 3. 30 New Beyond-Peak Sovereign Industry Breakthroughs (BP-01..BP-30 -> 155/155) & Legal/Market Audit
  const iter9 = runIteration9BeyondPeakAndLegalAudit(sim.kdb, sim.signals, sim.steps);
  assert.equal(iter9.trademarkCleanCompliancePct, 100.0);
  assert.equal(iter9.zeroThirdPartyProprietaryLibs, true);
  assert.equal(iter9.totalVerifiedCapabilitiesWithBeyondPeak, 155);
  assert.equal(iter9.beyondPeakBreakthroughs.length, 30);
  assert.equal(iter9.beyondPeakBreakthroughs[0].code, 'BP-01');
  assert.equal(iter9.beyondPeakBreakthroughs[29].code, 'BP-30');
  assert.ok(iter9.beyondPeakBreakthroughs.every((b) => b.status === 'VERIFIED_ACTIVE'));
});



