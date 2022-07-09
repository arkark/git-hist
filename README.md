# git-hist

[![](https://github.com/arkark/git-hist/workflows/Rust/badge.svg)](https://github.com/arkark/git-hist/actions)
[![crates.io](https://img.shields.io/crates/v/git-hist.svg)](https://crates.io/crates/git-hist)
[![license: MIT](https://img.shields.io/badge/license-MIT-yellow.svg)](https://github.com/arkark/git-hist/blob/master/LICENSE)

A CLI tool to quickly browse the git history of files **on a terminal**. This project is inspired by [git-history](https://github.com/pomber/git-history).

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

You can use `git-hist` as a git subcommand, so the hyphen is not required.

### Keymap

- <kbd>Left</kbd> / <kbd>Right</kbd> : Go to a previous/next commit.
- <kbd>Up</kbd> / <kbd>Down</kbd> or mouse scrolls: Scroll up/down.
- <kbd>PageUp</kbd> / <kbd>PageDown</kbd> : Scroll page up/down.
- <kbd>Home</kbd> / <kbd>End</kbd> : Scroll to the top/bottom.
- <kbd>q</kbd>, <kbd>Ctrl</kbd>+<kbd>c</kbd>, <kbd>Ctrl</kbd>+<kbd>d</kbd> : Exit.

### Help

```sh
$ git-hist --help
git-hist {{ version }}
A CLI tool to quickly browse the git history of files on a terminal

USAGE:
    git-hist [OPTIONS] <file>

ARGS:
    <file>    Set a target file path

OPTIONS:
        --beyond-last-line        Set whether the view will scroll beyond the last line
        --date-format <format>    Set date format: ref. https://docs.rs/chrono/0.4.19/chrono/format/strftime/index.html [default: [%Y-%m-%d]]
        --date-of <user>          Use whether authors or committers for dates [default: author] [possible values: author, committer]
        --emphasize-diff          Set whether the view will emphasize different parts
        --full-hash               Show full commit hashes instead of abbreviated commit hashes
    -h, --help                    Print help information
        --name-of <user>          Use whether authors or committers for names [default: author] [possible values: author, committer]
        --tab-size <size>         Set the number of spaces for a tab character (\t) [default: 4]
    -v, --version                 Print version information
```
