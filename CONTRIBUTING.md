# Contributing

Thanks for your interest in contributing to VIKAS.

## Development setup

Install Rust and Node.js, then run these commands from the repository root:

```bash
cargo build --release
cargo test --workspace
npm install
npm run lint
npm test
```

The local CI simulator is available as `bash ci_local_sim.sh`.

## Pull requests

1. Create a branch for your change.
2. Keep changes focused and add tests for behavior you change.
3. Run the relevant checks above and include their results in your pull request.
4. Describe any user-facing changes and update documentation when needed.

For bugs and feature ideas, open an issue using the repository's issue templates.
For other questions, email [sairamginnela@gmail.com](mailto:sairamginnela@gmail.com).
