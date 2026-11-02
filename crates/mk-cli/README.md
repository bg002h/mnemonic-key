# mk-cli

Standalone CLI for the **mk1** (mnemonic-key) backup format. `mk` keeps `encode` and `decode`. Verbs that several formats share belong to the toolkit (`mnemonic`). `repair` moves first: `mk repair` is planned for removal, and `mnemonic repair <mk1 strings>` repairs mk1 strings today. `inspect`, `verify`, `address` and `derive` are candidates to move later. `vectors`, `gui-schema` and `gen-man` describe `mk` itself and stay. Full table: [Which tool owns which verb](https://github.com/bg002h/mnemonic-toolkit/blob/master/docs/manual/src/50-comparing/51-format-decision.md#which-tool-owns-which-verb).

The canonical user-facing reference is the m-format-star user manual at
[`mnemonic-toolkit/docs/manual/src/40-cli-reference/44-mk-cli.md`](https://github.com/bg002h/mnemonic-toolkit/blob/master/docs/manual/src/40-cli-reference/44-mk-cli.md).
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
