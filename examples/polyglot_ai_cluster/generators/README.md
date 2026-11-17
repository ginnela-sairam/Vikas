# Chisel and SpinalHDL generators

These are small, independent hardware-generator projects. They keep Scala build
sources out of directory-based VIKAS RTL runs; the checked-in SystemVerilog
outputs under `rtl/generated/` are the simulation inputs.

## Prerequisites

- JDK 17
- sbt 1.9 or newer
- Network access on the first sbt run to resolve the pinned dependencies

Regenerate each checked-in RTL module from this directory:

```sh
cd examples/polyglot_ai_cluster/generators/chisel
sbt run
cd ../spinal
sbt run
cd ../../../..
```

The generated files are written to `examples/polyglot_ai_cluster/rtl/generated/`.
Then run the checked-in polyglot example and its testbench:

```sh
vikas run examples/polyglot_ai_cluster
```

The generator source files are not compiled by VIKAS. The generated RTL is
reviewable and checked into the example so the VIKAS testbench remains
reproducible without a Scala toolchain.
