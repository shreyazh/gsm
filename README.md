# gsm — Git Stash Manager

An interactive terminal TUI for managing git stashes. Built with Rust + Ratatui.

```
┌ gsm  branch: main  stashes: 4 ─────────────────────────────────────────────┐
│                                                                            │
│ 0   main             |    WIP auth middleware       |    2 hours ago        │
│▶ 1   feature/login   |     half-done login form    |     yesterday        │
│ 2   main             |    quick hotfix attempt      |   3 days ago        │
│ 3   fix/styles       |    CSS tweaks                |  last week         │
│                                                                           │
└───────────────────────────────────────────────────────────────────────────┘
```
```
┌──────────────────────────────────────────────────────────────────────────┐
│ [↑↓/jk] navigate  [Enter/d] diff  [f] files  [a] apply  [p] pop        │
│ [x] drop  [n] new  [/] search  [q] quit                                 │
└──────────────────────────────────────────────────────────────────────────┘
```

## Why?

`git stash` is powerful but painful to use once you have more than 2 stashes. The default CLI gives you:

```sh
git stash list        # a wall of text
git stash show -p 2   # if you remember the index
git stash drop 2      # pray you got the number right
```

`gsm` gives you an interactive TUI with diff preview, fuzzy search, and safe confirmations.

## Features

- **List** all stashes with branch, message, and relative date
- **Diff preview** — syntax-colored unified diff, scrollable
- **File summary** — see which files changed without the full diff
- **Apply** — apply stash, keep it in the list
- **Pop** — apply and remove (with confirmation)
- **Drop** — delete with confirmation (no accidents)
- **New stash** — create a named stash with optional untracked files
- **Fuzzy search** — filter by message or branch name
- **No dependencies** — single binary, no runtime required

## Install

### From source (requires Rust)

```sh
git clone https://github.com/shreyazh/gsm
cd gsm
cargo install --path .
```

### Homebrew (once published)

```sh
brew install gsm
```

### Direct binary

Download from [releases](https://github.com/shreyazh/gsm/releases).

## Keybindings

| Key          | Action                          |
|--------------|---------------------------------|
| `↑↓` / `jk`  | Navigate stash list             |
| `Enter` / `d`| View diff (colored)             |
| `f`          | View changed files summary      |
| `a`          | Apply stash (keep in list)      |
| `p`          | Pop stash (apply + remove)      |
| `x` / `Del`  | Drop (delete) stash             |
| `n`          | Create new named stash          |
| `/`          | Search / filter stashes         |
| `Esc`        | Back / cancel                   |
| `q`          | Quit                            |

In diff/file view:
| Key           | Action         |
|---------------|----------------|
| `↑↓` / `jk`   | Scroll         |
| `PgUp/PgDn`   | Fast scroll    |
| `Esc` / `q`   | Back to list   |

## Build

```sh
cargo build --release
# binary at ./target/release/gsm
```

## Tech Stack

- **Language:** Rust
- **TUI:** [Ratatui](https://ratatui.rs) + Crossterm
- **Git operations:** `git` subprocess (no libgit2, no extra deps)

## License

MITetcetera
