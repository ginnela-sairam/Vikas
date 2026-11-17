# Polyglot AI cluster examples

The directory includes a Chisel 6 multiply-accumulate tile and a SpinalHDL
NoC-router example. Their Scala generator projects and pinned dependency
versions are under [`generators/`](generators/); the generated, checked-in
SystemVerilog is in [`rtl/generated/`](rtl/generated/). VIKAS directory runs
skip the generator build sources and use the RTL outputs, so running the
simulation does not require Java, sbt, or network access.

## Run the examples

From the repository root, run the full cluster and the two generator testbench
checks:

```sh
vikas run examples/polyglot_ai_cluster --max-ns 600
```

The testbench reports `POLYGLOT-EXAMPLES-PASS 2/2` after checking the MAC
result and a routed NoC flit. The existing cluster manifest remains available
when you only want its original smoke test:

```sh
vikas run examples/polyglot_ai_cluster/cluster.f --max-ns 600
```

To regenerate the checked-in RTL, see [`generators/README.md`](generators/README.md).
