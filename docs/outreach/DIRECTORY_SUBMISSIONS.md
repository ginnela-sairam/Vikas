# EDA directory submission notes

These are neutral draft entries and submission locations, not endorsements or evidence that a directory has accepted VIKAS. Each maintainer controls their own inclusion criteria. Confirm the current repository instructions and avoid describing VIKAS as a complete IEEE implementation, formal signoff system, physical-design flow, or measured benchmark.

## Suggested neutral description

> VIKAS is a Rust CLI and TypeScript workbench with checked-in multi-file RTL simulation examples, project-specific latch/CDC/FSM checks, and binary waveform formats.

Use the direct source repository link: <https://github.com/ginnela-sairam/Vikas>.

## Targets and format-specific notes

1. **Garden of EDA** — submit an issue at <https://github.com/trishinraj/eda-tools-suggestions/issues>. The repository README asks for suggestions through issues. Propose the `simulator` and `waveform viewer` categories and let maintainers determine placement.
2. **drom/awesome-hdl** — under `Other Simulation tools`, add the one-line entry:
   `* [VIKAS](https://github.com/ginnela-sairam/Vikas) - RTL simulation and static analysis toolkit with binary waveform support.`
3. **hdl/awesome** — this is not a one-line README list. Its contribution guide requires one resource file per PR under `content/items/`, using the project metadata template and a discussion issue (`talk` field). Create/discuss the proposal first, then prepare a metadata file with `Tools:Simulators`, `Tools:Verification`, and `Tools:Waveform Viewer` categories only if maintainers agree those fit.
4. **ben-marshall/awesome-open-hardware-verification** — the current README uses `Simulation` and `Linting and Parsing` lists, rather than headings called “Simulators” and “Linters.” VIKAS has runnable demos and CI checks, but its static checks are project-specific heuristics. Ask maintainers whether the project fits their verification-only scope; do not imply formal verification.
5. **mattvenn/awesome-opensource-asic-resources** — the README has a `Digital focussed tools` list and no separate simulation subsection. If proposed, use the direct repository URL and describe only the checked-in simulation examples and waveform support.
6. **aolofsson/awesome-opensource-hardware** — its README requests working open-source projects, alphabetical ordering per category, direct source-repository links, and one tagline sentence per project. It lists `Simulators` in its table of contents and has a `Linters` section; follow the current README structure and keep the tagline neutral.
7. **ishandutta2007/Awesome-EDA** — under `HDL Simulation & Verification`, use the existing table columns. A factual row is:
   `| [VIKAS](https://github.com/ginnela-sairam/Vikas) | Rust CLI and TypeScript workbench with multi-file RTL examples, project-specific static checks, and binary waveforms. | 2026 | — | Not a synthesis or physical-design flow. |`
8. **clin99/awesome-eda** — this list is grouped by project year, not by an RTL-simulation heading. If proposing VIKAS, add a `2026` project entry and update its year index and summary counts; do not place a 2026 project under an unrelated year.
9. **rust-unofficial/awesome-rust** — its current `Simulation` subsection is under `Libraries`, not `Applications`. Its contribution guide requires alphabetical order and at least 50 GitHub stars or 2,000 crates.io downloads (or equivalent evidence). At the time this note was prepared, VIKAS had 2 GitHub stars and 5 release-asset downloads, so it does not meet that threshold. Do not submit until the requirement is met; when eligible, omit any crates.io link until a crate is actually published.

## Automation

[`../../scripts/submit_global_eda_directories.sh`](../../scripts/submit_global_eda_directories.sh) creates a Garden of EDA suggestion issue and proposes PRs for the supported README-based lists using `gh`, forks, and branches. It prompts before making external changes and skips `rust-unofficial/awesome-rust` unless the documented popularity threshold is met. `hdl/awesome` is left as a manual metadata-based submission; the script does not create its required discussion issue or guess its `talk` field. Review every generated diff and the destination maintainer instructions before creating PRs.
