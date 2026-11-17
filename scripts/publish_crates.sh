#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root_dir"

crates=(
  vikas-wave
  vikas-sim
  vikas-kdb
  vikas-wcp
  vikas-physics
  vikas-linux
  vikas-cli
)

if [[ "${PUBLISH_LIVE:-0}" == "1" ]]; then
  live=1
  echo "Publishing crates to crates.io in dependency order."
else
  live=0
  echo "Dry run only; listing package contents without uploading."
  echo "Set PUBLISH_LIVE=1 to publish to crates.io."
fi

for crate in "${crates[@]}"; do
  echo "==> $crate"
  if [[ "$live" == "1" ]]; then
    CARGO_NET_OFFLINE=false cargo publish --registry crates-io --locked --manifest-path "crates/$crate/Cargo.toml"
  else
    cargo package --list --allow-dirty --locked --manifest-path "crates/$crate/Cargo.toml"
  fi
done
