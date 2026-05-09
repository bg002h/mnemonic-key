# mk-cli

Standalone CLI for the **mk1** (mnemonic-key) backup format. Five
subcommands: `encode`, `decode`, `inspect`, `verify`, `vectors`.

The canonical user-facing reference is the m-format-star user manual at
[`mnemonic-toolkit/docs/manual/src/40-cli-reference/44-mk-cli.md`](https://github.com/bg002h/mnemonic-toolkit/blob/main/docs/manual/src/40-cli-reference/44-mk-cli.md).
For the underlying Rust API of `mk-codec`, see
[`docs/MK_CODEC_RUST_API.md`](../../docs/MK_CODEC_RUST_API.md) in this
repo.

## Build and install

```sh
cargo install --path crates/mk-cli   # from the workspace root
mk --help
```

## License

MIT License.
