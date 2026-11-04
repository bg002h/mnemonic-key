Full catalog: https://github.com/bg002h/mnemonic-refugium/blob/coordinator/coordinator/scripts/README.md

| script | path | purpose | usage |
|---|---|---|---|
| code-docs-check.sh | ci/code-docs-check.sh | Fails if a scanned code file names a file that ci/docs-only.sh treats as documentation. | `bash ci/code-docs-check.sh` |
| code-docs-check.test.sh | ci/code-docs-check.test.sh | Scenario tests for code-docs-check.sh, one throwaway git repo per row. | `bash ci/code-docs-check.test.sh` |
| docs-only.sh | ci/docs-only.sh | Writes docs_only=true to $GITHUB_OUTPUT only when every changed file is documentation (env GITHUB_EVENT_NAME, GITHUB_BASE_REF, GITHUB_OUTPUT). | `bash ci/docs-only.sh` |
| docs-only.test.sh | ci/docs-only.test.sh | Scenario tests for docs-only.sh. | `bash ci/docs-only.test.sh` |
| vendor-freshness.sh | ci/repro/vendor-freshness.sh | Fails iff the committed vendor/ cannot satisfy Cargo.lock under the --offline --locked source-replacement config (three-block form: crates-io, the mnemonic-io-lib git source, vendored-sources); fails closed if no git source or a second one appears. | `bash ci/repro/vendor-freshness.sh` |
