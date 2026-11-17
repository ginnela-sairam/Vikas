---
title: "VIKAS: A Rust and TypeScript Platform for Multicore RTL Simulation, Static Latch/CDC Signoff, and Binary Waveform Analysis"
authors:
  - name: Ginnela Someswara Sai Ram Prasad
    affiliation: 1
affiliations:
  - name: Vasireddy Venkatadri Institute of Technology / GINNELA, India
    index: 1
date: 17 November 2026
bibliography: paper.bib
---

# Summary

VIKAS is an open-source Rust command-line application and TypeScript/React workbench for selected register-transfer-level (RTL) simulation and analysis workflows. Its Rust workspace contains a simulation engine, waveform-format support, static analysis utilities, analytical physics examples, workbench protocol support, Linux-oriented helpers, and the CLI. The web application provides a browser interface to project examples and server-backed workflows.

The repository includes executable multi-file examples and CI checks for a multifile file list, a mixed-language example cluster, and a NOVA-SoC design. Simulation runs can produce binary `.vwf` and `.vwm` waveform files and VCD interchange data. Additional commands implement project-specific inferred-latch, CDC, FSM, and signoff checks. The latter are static checks and report generators, not a standards certification or formal proof.

VIKAS also contains analytical switching-energy and voltage-dependent estimates, plus an RK4 mixed-signal example. These are software models, not measured silicon results or foundry-calibrated analyses. Broader experimental command scenarios are documented separately and are not presented as validated physical models.

# Statement of need

Open-source hardware projects often combine RTL source, simulation outputs, static checks, and design exploration across separate tools. VIKAS packages a small, reproducible set of these workflows in one Rust workspace and a web workbench. Its checked-in examples make it possible to run the same representative input designs locally and in continuous integration, while the binary waveform formats provide project-native output alongside VCD interchange.

VIKAS is intended as an experimental and educational tool for hardware developers, students, and researchers who want to inspect and extend these workflows. It is not a replacement for mature standards-compliant simulators, formal verification systems, commercial signoff tools, or measured power analysis. The project documents those limits so that modeled outputs are not confused with independent benchmarks or engineering signoff.

# Software description

The seven Rust crates separate the CLI, simulation engine, waveform support, static-analysis data structures, analytical physics examples, workbench protocol, and Linux-oriented runtime helpers. The TypeScript/React workbench is served by an Express application. Users can build the CLI with Cargo, run the web workbench with Node.js, or use the published container image for either entry point.

The repository's CI exercises the example designs and Rust unit tests, checks an offline build, and compiles with Rust 1.70. Example commands and expected outputs are documented in the README and workflow files. The project-specific latch/CDC/FSM checks are heuristic static analyses; no claim is made that they fully implement IEEE language semantics or prove design correctness.

The physics crate offers model calculations such as switching-energy estimates and an RK4 mixed-signal example. Their output is illustrative model data and should not be interpreted as measured device behavior. Architectural and exploratory command scenarios are explicitly segregated in `docs/EXPERIMENTAL_MODELS.md`.

# Availability

VIKAS is available at <https://github.com/ginnela-sairam/Vikas> under the MIT OR Apache-2.0 license. Tagged CLI releases and prebuilt Linux x86_64 and Apple Silicon macOS binaries are published on the GitHub Releases page. The source repository contains the tests, sample designs, build instructions, and CI configuration needed to reproduce the documented checks.

# Acknowledgements

The project builds on ideas and publicly available work from the open-source hardware ecosystem, including Verilator, Yosys, OpenROAD, Surfer, and Icarus Verilog [@verilator; @yosys; @openroad; @surfer; @icarus]. These projects are cited for context; VIKAS does not claim compatibility or equivalence with them.
