# lumen-lens-backend

Rust engine for Lumen Lens — parses Soroban contract WASM binaries, decodes the embedded XDR interface spec, and generates typed client SDKs in TypeScript, Go, Python, and Dart via a Tera template engine. Includes breaking-change diffing, mock-client generation, and a CLI.

## Workspace crates

| Crate | Description |
|---|---|
| `crates/wasm-parser` | Reads `contractspecv0` WASM section → IR |
| `crates/codegen` | Tera template engine + per-language rendering |
| `crates/diff` | Breaking-change detector between two contract versions |
| `crates/mock-gen` | Mock-client stub generator for frontend teams |
| `cli` | `lumenlens` binary — all user-facing commands |

## Install

```bash
cargo install --path cli
```

## Usage

```bash
# Generate a TypeScript SDK from a local WASM
lumenlens generate --wasm ./my_contract.wasm --language typescript --output ./ts-sdk

# Diff two WASM versions
lumenlens diff --old v1.wasm --new v2.wasm

# Generate a mock client (frontend-first development)
lumenlens mock --wasm ./my_contract.wasm --language typescript --output ./ts-mock

# Inspect the parsed spec as JSON
lumenlens inspect --wasm ./my_contract.wasm
```

## Supported languages

- `typescript` / `ts`
- `go`
- `python` / `py`
- `dart`

## Templates

Templates live in `templates/<language>/`. Each `.tera` file produces one output file. Files prefixed with `_` are Tera partials (not rendered directly).

## Development

```bash
cargo test --all
cargo clippy --all-targets
```
