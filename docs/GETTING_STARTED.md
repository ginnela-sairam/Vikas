# Getting Started with VIKAS

VIKAS has two ways to get started: the web workbench and the native Rust CLI.
The web workbench needs its Node.js server; it is not hosted on GitHub Pages.

## Option 1: Run the workbench in Codespaces

1. Open [VIKAS in Codespaces](https://codespaces.new/ginnela-sairam/Vikas) and create a Codespace.
2. In the VS Code terminal, run:

   ```bash
   npm run dev
   ```

3. Open the forwarded port 3000 when Codespaces offers it. If needed, use the
   **Ports** tab and open port 3000 in your browser.

The development container installs Node.js dependencies when it is created.
If setup has not run, use `npm install` once before starting the server.

## Option 2: Run the native CLI

Install Rust 1.70 or newer, then run these commands from the repository root:

```bash
cargo build --release --locked --offline
./target/release/vikas version
./target/release/vikas run examples/nova_soc/nova.f --max-ns 8000
```

The dependencies are vendored in this repository, so the build command uses
Cargo's offline mode. If Cargo reports a missing dependency or toolchain, first
confirm Rust is installed and that the repository's `vendor/` directory is
present.

## Run the checks

```bash
npm run lint
npm test
npm run build
```

For contribution instructions, see [CONTRIBUTING.md](../CONTRIBUTING.md).

## Important limitations

VIKAS is a software project for simulation and EDA-related workflows. Results
and performance depend on the implementation, design, and hardware. Project
benchmarks are not independent third-party benchmarks or silicon measurements.
Validate results with your own designs and tools; do not treat VIKAS as a
replacement for qualified production signoff.
