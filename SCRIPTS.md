Full catalog: https://github.com/bg002h/mnemonic-refugium/blob/coordinator/coordinator/scripts/README.md

| script | path | purpose | usage |
|---|---|---|---|
| code-docs-check.sh | ci/code-docs-check.sh | Fails if a scanned code file names a file that ci/docs-only.sh treats as documentation. | `bash ci/code-docs-check.sh` |
| code-docs-check.test.sh | ci/code-docs-check.test.sh | Scenario tests for code-docs-check.sh, one throwaway git repo per row. | `bash ci/code-docs-check.test.sh` |
| docs-only.sh | ci/docs-only.sh | Writes docs_only=true to $GITHUB_OUTPUT only when every changed file is documentation (env GITHUB_EVENT_NAME, GITHUB_BASE_REF, GITHUB_OUTPUT). | `bash ci/docs-only.sh` |
| docs-only.test.sh | ci/docs-only.test.sh | Scenario tests for docs-only.sh. | `bash ci/docs-only.test.sh` |
| vendor-freshness.sh | ci/repro/vendor-freshness.sh | Four-check vendor/ gate (shared file): resolution under an empty CARGO_HOME, file integrity, registry provenance, and each git source re-derived from upstream at its Cargo.lock rev (rows in ci/repro/vendor-git-sources.txt). | `bash ci/repro/vendor-freshness.sh` |
| vendor-freshness.test.sh | ci/repro/vendor-freshness.test.sh | Mutation tests for vendor-freshness.sh (shared file; needs python3, tar, git, cargo and network). | `bash ci/repro/vendor-freshness.test.sh` |
| mk-fe-keyed-policy-rows.sh | scripts/mk-fe-keyed-policy-rows.sh | RED/GREEN CLI rows for design/FIX_mk_fe_keyed_policy.md (bh-mk-I2 origin vs keyed policy, sweep row 15 repeated/contradictory key records, mk-dup-stub) plus controls; builds mk with --locked on the 1.85 pin. | `scripts/mk-fe-keyed-policy-rows.sh red\|green` |
| mk-fd-derive-guards-rows.sh | scripts/mk-fd-derive-guards-rows.sh | RED/GREEN CLI rows for design/FIX_mk_fd_derive_guards.md (bh-mk-C1 fixed-key card, bh-mk-I18 empty --path) plus controls; builds mk with --locked on the 1.85 pin. | `scripts/mk-fd-derive-guards-rows.sh red\|green` |
| mk-ff-stale-docs-rows.sh | scripts/mk-ff-stale-docs-rows.sh | RED/GREEN rows for design/FIX_mk_ff_stale_docs.md (bh-mk-I5 recovery recipe in SPEC and BIP, bh-mk-I14 Rust API guide consumer built as printed; needs network, cargo +1.85.0). | `scripts/mk-ff-stale-docs-rows.sh red\|green` |
| mk-fa-line-readers-rows.sh | scripts/mk-fa-line-readers-rows.sh | RED/GREEN CLI rows for design/FIX_mk_fa_line_readers.md (bh-mk-I3 bare CR in --keys/--in key files, bh-mk-I4 BOM and bare CR in --from-md1-set) plus LF/CRLF/leading-BOM controls; builds mk with the 1.85 pin, no --release. | `scripts/mk-fa-line-readers-rows.sh red\|green` |
| mk-fg-ci-gates-rows.sh | scripts/mk-fg-ci-gates-rows.sh | RED/GREEN rows for design/FIX_mk_fg_ci_gates.md (bh-mk-I11 lane toolchains and beta label, I12 release-create gates in three workflows, I13 aarch64 recipe) with mocked gh and cross; needs the stable and 1.95.0 toolchains installed. | `scripts/mk-fg-ci-gates-rows.sh red\|green` |
