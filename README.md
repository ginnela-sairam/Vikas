# VIKAS: RTL Simulation, Static Analysis, and Waveform Tools

[![License: MIT / Apache-2.0](https://img.shields.io/badge/License-MIT%20%2F%20Apache--2.0-blue.svg)](LICENSE)
[![Latest release](https://img.shields.io/github/v/release/ginnela-sairam/Vikas)](https://github.com/ginnela-sairam/Vikas/releases/latest)
[![GitHub CI](https://img.shields.io/github/actions/workflow/status/ginnela-sairam/Vikas/ci.yml?branch=main&label=CI)](.github/workflows/ci.yml)

**VIKAS 3.0.0** is an open-source Rust command-line toolkit and TypeScript/React workbench for selected RTL simulation, static checks, waveform handling, and analytical models.

The repository contains reproducible workflows for:

- Multi-file RTL examples: `examples/nova_soc/nova.f`, `examples/multifile_proj/project.f`, and `examples/polyglot_ai_cluster/cluster.f`.
- Three-layer inferred-latch detection plus project-specific CDC, FSM, and signoff checks through commands such as `vikas latch`, `vikas cdc`, `vikas fsm`, and `vikas signoff`.
- Binary waveform outputs including `.vwf` and `.vwm`, alongside VCD interchange.
- Analytical switching-energy/sub-threshold estimates and an RK4 mixed-signal example. These are software models, not measured silicon results.
- A TypeScript/React web workbench served by the Node.js application.

These checks are not a claim of complete IEEE language compliance, formal equivalence, or tapeout qualification. Benchmark-like values and exploratory command scenarios are not independent measurements. See [Experimental models](docs/EXPERIMENTAL_MODELS.md) for a clear inventory of analytical and architectural scenarios.

## Project links

- [Open this repository in GitHub Codespaces](https://codespaces.new/ginnela-sairam/Vikas)
- [Download the VIKAS CLI release](https://github.com/ginnela-sairam/Vikas/releases/latest)
- [Beginner getting-started guide](docs/GETTING_STARTED.md)
- [Experimental analytical and architectural models](docs/EXPERIMENTAL_MODELS.md)
- [JOSS software paper draft](paper/paper.md)
- [Contributing guide](CONTRIBUTING.md)
- [Security policy](SECURITY.md)
- [Code of conduct](CODE_OF_CONDUCT.md)

The repository includes a composite GitHub Action in [`action.yml`](action.yml),
a development container in [`.devcontainer/`](.devcontainer/), and a workflow
for tagged release binaries. The full workbench requires its Node.js server;
GitHub Pages cannot host its API. Run `npm run dev` to use the complete workbench.

For general questions, contact [sairamginnela@gmail.com](mailto:sairamginnela@gmail.com).

## Table of Contents

1. [Installation & Compilation Guide](#1-installation--compilation-guide)
2. [Quick-Start Verification](#2-quick-start-verification)
3. [GitHub Actions CI](#3-github-actions-ci)
4. [Implemented Workflows](#4-implemented-workflows)
5. [Repository Layout](#5-repository-layout)
6. [License](#6-license-trademark-hygiene)

---

## 1. Installation & Compilation Guide

The Rust workspace uses vendored dependencies for offline Cargo builds. The prebuilt CLI installer supports Linux x86_64 and Apple Silicon macOS; source builds need Rust/Cargo 1.70 or newer and a C++20 compiler.

### Run the web workbench

Requires Node.js 18 or newer:
```bash
npm install
npm run dev
```
Open `http://localhost:3000` in a browser.

### Install or build the CLI

Use the prebuilt binary on supported platforms:
```bash
curl -fsSL https://raw.githubusercontent.com/ginnela-sairam/Vikas/main/install.sh | sh
```

Or build from source:
```bash
cargo build --release --locked --offline
./target/release/vikas version
```

The repository also includes the source installer [`install_linux.sh`](install_linux.sh).

### Run the container

The main-branch and version-tag workflow publishes `ghcr.io/ginnela-sairam/vikas`:
```bash
docker run --rm ghcr.io/ginnela-sairam/vikas:latest version
```

To run the workbench instead of the default CLI entrypoint:
```bash
docker run --rm -p 3000:3000 --entrypoint npm ghcr.io/ginnela-sairam/vikas:latest --prefix /app run start
```

## 2. Quick-Start Verification

From the repository root, build and run the checked-in examples:
```bash
cargo build --release --locked --offline
./target/release/vikas run examples/multifile_proj/project.f
./target/release/vikas run examples/polyglot_ai_cluster/cluster.f --max-ns 600
./target/release/vikas run examples/nova_soc/nova.f --max-ns 8000
./target/release/vikas latch examples/demo_latch_trap.sv
./target/release/vikas signoff examples/nova_soc/nova.f --max-ns 8000
```

## 3. GitHub Actions CI

Every push and pull request to `main` runs six jobs in [`.github/workflows/ci.yml`](.github/workflows/ci.yml): executable/shell checks, Rust build and tests, a CLI test matrix, example workflows, an offline build, and a Rust 1.70 build. Run the local checks with:

```bash
./ci_local_sim.sh
npm run lint
npm test
npm run build
```

## 4. Implemented Workflows

### Multi-file RTL simulation

The checked-in examples exercise the simulator and its file-list handling:

```bash
cargo build --release --locked --offline
./target/release/vikas run examples/multifile_proj/project.f
./target/release/vikas run examples/polyglot_ai_cluster/cluster.f --max-ns 600
./target/release/vikas run examples/nova_soc/nova.f --max-ns 8000
```

The project CI runs these workflows and checks their expected example outputs. See [the CI workflow](.github/workflows/ci.yml) for the exact commands.

### Static checks and signoff reports

The CLI includes project-specific latch, CDC, FSM, and signoff checks. For example, `vikas latch examples/demo_latch_trap.sv` checks the supplied latch demonstration, while `vikas cdc examples/demo_cdc.sv` and `vikas fsm examples/demo_fsm_uart.sv` exercise the corresponding check paths. These are static heuristics and report generators, not a complete standards-compliance suite or formal CDC proof.

For machine-readable latch results, use `vikas latch <files> --sarif`. The
command emits SARIF 2.1.0 JSON and returns exit code 2 when it finds inferred
latches; without `--sarif`, the existing human-readable report is unchanged.
To upload results with GitHub code scanning, configure repository workflow
permissions and a SARIF upload step explicitly, for example:

```yaml
permissions:
  contents: read
  security-events: write

steps:
  - run: |
      set +e
      ./target/release/vikas latch examples/demo_latch_trap.sv --sarif > results.sarif
      status=$?
      set -e
      if [ "$status" -gt 2 ]; then exit "$status"; fi
  - uses: github/codeql-action/upload-sarif@v3
    with:
      sarif_file: results.sarif
```

This example shows the required workflow setup; enabling code scanning and
granting `security-events: write` are repository/organization configuration
decisions and are not enabled automatically by VIKAS.

### Waveform formats

Simulation runs write waveform data that can be inspected with the CLI. The waveform crate implements binary `.vwf` and `.vwm` formats and VCD interchange; format round-trip tests are in the Rust workspace. Format support does not imply a validated throughput claim for arbitrary trace sizes or hardware.

To benchmark the checked-in RV32I VCD fixture with bounded-memory conversion,
run `vikas vcd-bench`. You can supply an input trace and output path as
`vikas vcd-bench input.vcd output.vwm`; the command reports measured byte
counts, elapsed time, and throughput for that run.

The Chisel 6 and SpinalHDL examples are documented in
[`examples/polyglot_ai_cluster/README.md`](examples/polyglot_ai_cluster/README.md).

### Analytical physics examples

The physics crate contains switching-energy and voltage-dependent analytical models and an RK4 mixed-signal example. Their output is model output only; it is not a measurement, foundry-calibrated result, or signoff guarantee. Other exploratory model scenarios are listed in [docs/EXPERIMENTAL_MODELS.md](docs/EXPERIMENTAL_MODELS.md).

### TypeScript workbench

The Node.js server exposes the web workbench and local project examples. Start it with `npm install && npm run dev`, then open `http://localhost:3000`. The workbench needs its Node.js server and is not hosted by GitHub Pages.

---

## 5. Repository Layout

```text
Vikas/
├── Cargo.toml                  # Seven-crate Rust workspace
├── Cargo.lock                  # Locked dependency graph
├── .cargo/config.toml          # Vendored Cargo dependencies for offline builds
├── crates/
│   ├── vikas-sim/              # RTL simulation engine
│   ├── vikas-wave/             # Waveform formats and readers
│   ├── vikas-kdb/              # Static analysis and design graph utilities
│   ├── vikas-physics/          # Analytical energy and RK4 examples
│   ├── vikas-linux/            # Linux-oriented runtime helpers
│   ├── vikas-wcp/              # Workbench protocol and session bundles
│   └── vikas-cli/              # `vikas` command-line application
├── examples/                   # Reproducible simulation and analysis inputs
├── src/                        # TypeScript/React workbench
├── server.ts                   # Express application and workbench API
└── docs/                       # User and model documentation
```

---

## 6. License & Trademark Hygiene

VIKAS is dual-licensed under **MIT OR Apache-2.0**, at your option. See
[`LICENSE-MIT`](LICENSE-MIT) and [`LICENSE-APACHE`](LICENSE-APACHE).
Third-party components in `vendor/` retain their own license terms.

