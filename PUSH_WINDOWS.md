# Pushing to this repo from Windows — the 2-minute guide

Windows filesystems (NTFS) cannot store the Linux executable bit, and Git on
Windows runs with `core.filemode=false`. That combination has one nasty
side effect that has already bitten this repo once:

> **A plain `git add -A` from Windows silently resets `build_linux.sh`,
> `install_linux.sh` and `test_matrix.sh` to non-executable (mode `100644`).**
> The next friend cloning on Linux/WSL then gets `Permission denied` on their
> very first command — the worst possible first impression.

The CI **exec-bits guard** job catches this within seconds of a bad push and
prints the self-heal commands below. This page makes sure you never need it.

## The safe push recipe (5 commands)

Run these in the repo root — Git Bash, PowerShell with git, or WSL all work:

```bash
git pull                                # sync to latest
# ... overlay / edit files as usual ...
git add -A
git update-index --chmod=+x build_linux.sh install_linux.sh test_matrix.sh
git commit -m "your message" && git push
```

The `git update-index --chmod=+x` line is the whole trick: it sets the
executable bit **in the Git index itself**, so it survives the push and lands
on GitHub as mode `100755` — exactly what a Linux fresh clone needs. It works
on Windows because it edits the index, not the filesystem.

## Verifying it worked

1. Push, then open the repo on GitHub — the **exec-bits guard** job must be
   green. If it is red, it prints the exact self-heal commands; run them,
   push again.
2. Optional local check without pushing:
   `git ls-files -s build_linux.sh` must show `100755`, not `100644`.

## About the CRLF warnings

Lines like `warning: LF will be replaced by CRLF in ...` during `git add` are
**harmless**: with `core.autocrlf=true`, Git stores clean LF in the repo and
only converts in your working copy. The exec-bits guard additionally fails
loudly if real CRLF ever lands inside the three shell scripts, so a Windows
push can never corrupt them silently.

## Release tags

The `v1.0` tag was created before the CI was green (it points at `eb82d74`,
whose CI run failed). Nothing has been distributed from that tag yet, so the
cleanest fix is to move it to the current green commit (pick **one** option):

```bash
# Option A — move v1.0 to the fixed commit (recommended: tag = "the release")
git tag -f v1.0 main && git push -f origin v1.0

# Option B — keep v1.0 untouched, cut a fresh tag instead
git tag v1.1 main && git push origin v1.1
```

## After you push — tell your friends just this

From this commit onward the friend experience is self-healing:

```bash
cd ~/Vikas && vikas update && vikas gui
```

`vikas update` pulls, rebuilds and reinstalls by itself — and any command
now warns loudly if someone pulls code but forgets to rebuild (the exact
failure that cost a friend an afternoon). `vikas doctor` diagnoses any
environment in seconds. See `TROUBLESHOOTING.md` for the full guide.
