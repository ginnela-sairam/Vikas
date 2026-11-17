#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "Usage: $0 --apply [--yes]" >&2
  echo "Without --apply, no issues or pull requests are created." >&2
}

apply=0
assume_yes=0
for arg in "$@"; do
  case "$arg" in
    --apply) apply=1 ;;
    --yes) assume_yes=1 ;;
    -h|--help) usage; exit 0 ;;
    *) usage; exit 2 ;;
  esac
done

if [[ "$apply" != "1" ]]; then
  usage
  exit 0
fi

if [[ "$assume_yes" != "1" ]]; then
  read -r -p "Create a Garden of EDA issue and propose external directory PRs? [y/N] " answer
  [[ "$answer" == "y" || "$answer" == "Y" ]] || {
    echo "No external changes made."
    exit 0
  }
fi

command -v gh >/dev/null
command -v git >/dev/null
command -v python3 >/dev/null
owner="$(gh api user --jq .login)"
account_id="$(gh api "users/$owner" --jq .id)"
timestamp="$(date -u +%Y%m%d%H%M%S)"
tmp_root="$(mktemp -d)"
trap 'rm -rf "$tmp_root"' EXIT

garden_title="Suggest VIKAS for simulator and waveform viewer categories"
garden_issue="$(gh issue list --repo trishinraj/eda-tools-suggestions --state open --json number,title \
  --jq ".[] | select(.title == \"$garden_title\") | .number" | head -n 1)"
if [[ -z "$garden_issue" ]]; then
  gh issue create \
    --repo trishinraj/eda-tools-suggestions \
    --title "$garden_title" \
    --body "Project: https://github.com/ginnela-sairam/Vikas

VIKAS is a Rust CLI and TypeScript workbench with checked-in multi-file RTL simulation examples, project-specific latch/CDC/FSM checks, and binary waveform formats.

Suggested categories for maintainer review: simulator and waveform viewer. The RTL checks are project-specific heuristics, and the physics commands produce analytical model outputs rather than measured results."
else
  echo "Garden of EDA suggestion already exists as issue #$garden_issue."
fi

add_readme_pr() {
  local repo="$1"
  local kind="$2"
  local base fork_slug repo_name clone_dir branch

  base="$(gh api "repos/$repo" --jq .default_branch)"
  repo_name="${repo##*/}"
  fork_slug="$owner/$repo_name"

  if [[ "$kind" == "rust" ]]; then
    local stars
    stars="$(gh api repos/ginnela-sairam/Vikas --jq .stargazers_count)"
    if (( stars < 50 )); then
      echo "Skipping $repo: VIKAS has $stars stars; the directory requires at least 50 stars or 2,000 crate downloads."
      return 0
    fi
  fi

  if ! gh api "repos/$fork_slug" >/dev/null 2>&1; then
    gh repo fork "$repo" --clone=false
  fi

  clone_dir="$tmp_root/${repo//\//-}"
  gh repo clone "$fork_slug" "$clone_dir"
  git -C "$clone_dir" remote add upstream "https://github.com/$repo.git"
  git -C "$clone_dir" fetch upstream "$base"
  branch="docs/add-vikas-$timestamp"
  git -C "$clone_dir" checkout -B "$branch" "upstream/$base"

  if python3 - "$clone_dir/README.md" "$kind" <<'PY'
from pathlib import Path
import sys

path = Path(sys.argv[1])
kind = sys.argv[2]
text = path.read_text()
url = "https://github.com/ginnela-sairam/Vikas"
if url in text:
    print(f"VIKAS is already listed in {path}")
    raise SystemExit(10)

entries = {
    "drom": (
        "## Other Simulation tools\n",
        "* [VIKAS](https://github.com/ginnela-sairam/Vikas) - RTL simulation and static analysis toolkit with binary waveform support.\n",
    ),
    "aolofsson": (
        "## Simulators\n",
        "* [VIKAS](https://github.com/ginnela-sairam/Vikas) - RTL simulation with checked-in examples and binary waveform support.\n",
    ),
    "ben": (
        "#### Simulation:\n",
        "- [VIKAS](https://github.com/ginnela-sairam/Vikas) - RTL simulator with runnable examples and project-specific static checks.\n",
    ),
    "mattvenn": (
        "## Digital focussed tools\n",
        "* [VIKAS](https://github.com/ginnela-sairam/Vikas) - RTL simulation examples, project-specific static checks, and binary waveform support.\n",
    ),
    "ishandutta": (
        "### 📡 HDL Simulation & Verification\n",
        "| [VIKAS](https://github.com/ginnela-sairam/Vikas) | Rust CLI and TypeScript workbench with multi-file RTL examples, project-specific static checks, and binary waveforms. | 2026 | — | Not a synthesis or physical-design flow. |\n",
    ),
    "clin99": (
        "* [2019](#2019) \n",
        "* [2026](#2026)\n",
    ),
    "rust": (
        "* [nyx-space](https://crates.io/crates/nyx-space)",
        "* [ginnela-sairam/Vikas](https://github.com/ginnela-sairam/Vikas) - Rust CLI and workspace libraries for RTL simulation and waveform handling.\n",
    ),
}

if kind == "drom":
    anchor, entry = entries[kind]
    if anchor not in text:
        raise SystemExit(f"Could not find section in {path}: {anchor.strip()}")
    text = text.replace(anchor, anchor + "\n" + entry, 1)
elif kind == "aolofsson":
    import re

    def insert_alphabetically(section, entry):
        global text
        heading = re.search(rf"(?m)^## {re.escape(section)}\s*$", text)
        if heading is None:
            raise SystemExit(f"Could not find {section} section.")
        start = heading.end()
        next_heading = re.search(r"(?m)^## ", text[start:])
        end = start + next_heading.start() if next_heading else len(text)
        lines = text[start:end].splitlines(keepends=True)
        insertion = len(lines)
        for index, line in enumerate(lines):
            item = re.match(r"\s*[-*]\s+\[([^\]]+)\]", line)
            if item and item.group(1).casefold() > "vikas":
                insertion = index
                break
        lines.insert(insertion, entry)
        text = text[:start] + "".join(lines) + text[end:]

    insert_alphabetically("Simulators", entries[kind][1])
    insert_alphabetically(
        "Linters",
        "* [VIKAS](https://github.com/ginnela-sairam/Vikas) - Project-specific latch, CDC, and FSM checks for supplied designs.\n",
    )
elif kind == "ben":
    anchor, entry = entries[kind]
    if anchor not in text:
        raise SystemExit("Could not find Simulation list.")
    text = text.replace(anchor, anchor + "\n" + entry, 1)
    lint_anchor = "#### Linting and Parsing:\n"
    lint_entry = "- [VIKAS](https://github.com/ginnela-sairam/Vikas) - Project-specific latch, CDC, and FSM checks.\n"
    if lint_anchor not in text:
        raise SystemExit("Could not find Linting and Parsing list.")
    text = text.replace(lint_anchor, lint_anchor + "\n" + lint_entry, 1)
elif kind == "mattvenn":
    anchor, entry = entries[kind]
    if anchor not in text:
        raise SystemExit("Could not find Digital focussed tools section.")
    end = text.find("\n## ", text.index(anchor) + len(anchor))
    if end == -1:
        end = len(text)
    text = text[:end] + "\n" + entry.rstrip() + "\n" + text[end:]
elif kind == "ishandutta":
    anchor, entry = entries[kind]
    start = text.find(anchor)
    if start == -1:
        raise SystemExit("Could not find HDL Simulation & Verification section.")
    table_start = text.find("| Tool |", start)
    if table_start == -1:
        raise SystemExit("Could not find the HDL table header.")
    table_end = text.find("\n\n", table_start)
    if table_end == -1:
        raise SystemExit("Could not find the end of the HDL table.")
    text = text[:table_end] + "\n" + entry.rstrip() + text[table_end:]
elif kind == "clin99":
    toc_anchor, toc_entry = entries[kind]
    if toc_anchor not in text:
        raise SystemExit("Could not find the 2019 table-of-contents entry.")
    text = text.replace(toc_anchor, toc_anchor + toc_entry, 1)
    section_anchor = "## Summary\n"
    section_entry = (
        "### 2026\n"
        "* [VIKAS](https://github.com/ginnela-sairam/Vikas) - Rust RTL simulation, project-specific static checks, and waveform tooling.\n\n"
    )
    if section_anchor not in text or "### 2026\n" in text:
        raise SystemExit("Could not find a unique Summary insertion point.")
    text = text.replace(section_anchor, section_entry + section_anchor, 1)
    summary_anchor = "| [2019](#2019) | 15  | 68 |"
    if summary_anchor not in text:
        raise SystemExit("Could not find the year summary row.")
    text = text.replace(summary_anchor, summary_anchor + "\n| [2026](#2026) | 1  | 69 |", 1)
elif kind == "rust":
    anchor, entry = entries[kind]
    if anchor not in text:
        raise SystemExit("Could not find the alphabetical Simulation insertion point.")
    text = text.replace(anchor, entry + anchor, 1)
else:
    raise SystemExit(f"Unknown directory target: {kind}")

path.write_text(text)
print(f"Prepared {kind} entry in {path}")
PY
  then
    :
  else
    local result=$?
    if [[ "$result" == "10" ]]; then
      return 0
    fi
    return "$result"
  fi

  git -C "$clone_dir" config user.name "$owner"
  git -C "$clone_dir" config user.email "${account_id}+${owner}@users.noreply.github.com"
  git -C "$clone_dir" add README.md
  git -C "$clone_dir" diff --cached --check
  git -C "$clone_dir" commit -m "docs: add VIKAS to the directory"
  git -C "$clone_dir" push --set-upstream origin "$branch"
  gh pr create \
    --repo "$repo" \
    --base "$base" \
    --head "$owner:$branch" \
    --title "docs: add VIKAS to the directory" \
    --body "Add a factual entry for VIKAS, a Rust CLI and TypeScript workbench with reproducible RTL examples, project-specific static checks, and waveform formats. The README and CI document the supported scope and limitations."
}

add_readme_pr drom/awesome-hdl drom
add_readme_pr aolofsson/awesome-opensource-hardware aolofsson
add_readme_pr ben-marshall/awesome-open-hardware-verification ben
add_readme_pr mattvenn/awesome-opensource-asic-resources mattvenn
add_readme_pr ishandutta2007/Awesome-EDA ishandutta
add_readme_pr clin99/awesome-eda clin99
add_readme_pr rust-unofficial/awesome-rust rust

echo "Directory suggestions processed. Review each issue/PR and respond to maintainers."
