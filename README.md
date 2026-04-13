# Varlink Language Server

Language server for [Varlink](https://varlink.org).
Using [Auto LSP](https://github.com/adclz/auto-lsp) and [tree-sitter-varlink](https://github.com/bachorp/tree-sitter-varlink).

## Features

- Diagnostics
    - Syntax errors
    - Missing/duplicate declarations
- Completion
- Formatting via [`varlinkfmt`](https://crates.io/crates/varlinkfmt)
- Go to definition
- Highlight
- Hover
- References
- Rename symbol
- Selection range
- Semantic tokens
- Symbols

## Known Issues and Limitations

- No workspace support (https://github.com/adclz/auto-lsp/issues/47#issuecomment-4231703174)
- Works only on files with extension `.varlink` (https://github.com/adclz/auto-lsp/issues/47)
- Wrong positions might be reported on lines that contain special characters (https://github.com/adclz/auto-lsp/issues/39#issuecomment-4114707968)
