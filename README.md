# git-hist

[![](https://github.com/arkark/git-hist/workflows/Rust/badge.svg)](https://github.com/arkark/git-hist/actions)
[![crates.io](https://img.shields.io/crates/v/git-hist.svg)](https://crates.io/crates/git-hist)
[![license: MIT](https://img.shields.io/badge/license-MIT-yellow.svg)](https://github.com/arkark/git-hist/blob/master/LICENSE)

A CLI to browse the history of files in any git repository **on a terminal**. This project is inspired by [git-history](https://github.com/pomber/git-history).

<div align="center">
    <img src="screenshots/screenshot_01.png" />
</div>

## Installation

```sh
cargo install git-hist
```

## Usage

```sh
git hist <file>
```

- `Left`/`Right`: Go to a previous/next commit.
- `Up`/`Down` or mouse scrolls: Scroll up/down.
- `PageUp`/`PageDown`: Scroll page up/down.
- `Home`/`End`: Scroll to the top/bottom.
- `q`/`Ctrl+c`/`Ctrl+d`: Exit.
