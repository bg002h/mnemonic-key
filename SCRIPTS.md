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
