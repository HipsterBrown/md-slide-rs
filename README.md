# md-slide

A markdown to slide generator CLI

Built in Rust using [structopt](https://lib.rs/crates/structopt) and [pulldown-cmark](https://lib.rs/crates/pulldown-cmark).

## Motivation

I enjoy writing my presentations in Markdown and wanted to be able generate the slides from that content. After trying [`mdx-deck`](https://github.com/jxnblk/mdx-deck), [`marp`](https://marp.app), and [`remark`](https://github.com/gnab/remark), I wanted something a bit simpler for my workflow of writing in vim then presenting pretty basic slides from those notes. The tools I mentioned are great for flexibility and powerful layouts that I don't normally need.

## TODO

- [X] finish initial default slide theme with styles
- [ ] allow for keyboard navigation of slides with a bit of JS
- [X] reorganize CLI into `build`, `serve` commands
- [ ] add `watch` command
- [ ] testing (?) through GitHub Actions
- [ ] publish v0.1


## Potential Features

- built-in syntax highlighting for code blocks
- custom slide layouts
- presenter view with footnotes
- toml config file support for presentation settings
