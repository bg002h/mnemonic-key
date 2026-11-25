#!/usr/bin/env bash
# RED/GREEN rows for design/FIX_mk_ff_stale_docs.md (bughunt-1119 F-F: bh-mk-I5, bh-mk-I14).
#
# What it does: runs every row the document prints and checks it against the
# MODE's expectation:
#   red   = the base commit's text (each defect shows)
#   green = the fixed text
# D rows read the SPEC and the BIP draft with grep/awk (no build).
# C rows build a throwaway consumer crate from the Rust API guide AS PRINTED:
# the first ```toml block under "## Cargo dependency" becomes its
# [dependencies], the first ```rust block under "## Encoding an mk1 card from a
# `KeyCard`" becomes the body of
# `fn main() -> Result<(), Box<dyn std::error::Error>> { ... Ok(()) }`, then
# `cargo run` on the 1.85 pin. The script adds only a [package] header and an
# empty [workspace] table (so the crate under target/ is not taken for a member
# of this workspace). C rows need network access (the manifest is a git
# dependency on github.com and crates.io).
# Any row that does not match its expectation fails the script (exit 1).
#
# What it does NOT cover: the nextest rows in
# crates/mk-cli/tests/bh1119_stale_docs.rs (the builder adds them); --locked for
# the consumer (it has no lockfile by construction, like any first-time reader);
# the README, DECISIONS and the BIP glossary/naming-note wording beyond the
# grep strings below; fmt, clippy, the full suite; non-Linux CI legs.
#
# Usage: scripts/mk-ff-stale-docs-rows.sh red|green   (run from the repo root)
set -euo pipefail
export PATH="$HOME/.cargo/bin:$PATH"
MODE=${1:?usage: $0 red|green}
case "$MODE" in red|green) ;; *) echo "usage: $0 red|green" >&2; exit 2;; esac
REPO=$(git rev-parse --show-toplevel)
WORK=${WORK:-$REPO/target/mk-ff-rows}
case "$WORK" in /tmp*) echo "refusing /tmp" >&2; exit 2;; esac
cargo +1.85.0 --version | grep -q '^cargo 1\.85\.' || { echo "cargo +1.85.0 is missing" >&2; exit 2; }
SPEC=$REPO/design/SPEC_mk_v0_1.md
BIP=$REPO/bip/bip-mnemonic-key.mediawiki
GUIDE=$REPO/docs/MK_CODEC_RUST_API.md
mkdir -p "$WORK"
FAIL=0
ok()   { echo "OK   $MODE $1"; }
bad()  { echo "FAIL $MODE $1: $2"; FAIL=1; }
# want NAME RED_COND GREEN_COND: each COND is "yes" or "no" for "defect present".
want() { local name=$1 present=$2 exp; if [ "$MODE" = red ]; then exp=yes; else exp=no; fi
  if [ "$present" = "$exp" ]; then ok "$name (defect present: $present)"; else bad "$name" "defect present: $present, want $exp"; fi; }
has() { if grep -qF -- "$1" "$2"; then echo yes; else echo no; fi; }

# D1: SPEC and BIP name an md-codec helper that does not exist at the pinned md-codec.
p=no; for f in "$SPEC" "$BIP"; do [ "$(has compute_wallet_instance_id "$f")" = yes ] && p=yes; done
want D1_docs_name_missing_helper $p
# D2: SPEC and BIP print the 78-byte xpub-concatenation recipe.
p=no; for f in "$SPEC" "$BIP"; do [ "$(has canonical_xpub_serialization "$f")" = yes ] && p=yes; done
want D2_docs_print_xpub_concat_recipe $p
# D3: the recovery step 4 text (SPEC §5 item "4. ", BIP recovery-flow item
# "# Reconstruct ...") does not name compute_wallet_policy_id.
awk '/^## §5\. Linkage to MD/{s=1} s&&/^4\. /{p=1} s&&/^5\. /{exit} p' "$SPEC" > "$WORK/spec-step4.txt"
awk '/^# Reconstruct the full descriptor/{p=1} p&&/^# Reject if mismatch/{exit} p' "$BIP" > "$WORK/bip-step4.txt"
for f in spec bip; do
  [ -s "$WORK/$f-step4.txt" ] || bad "D3_${f}_step4" "step 4 not found"
  if grep -qF compute_wallet_policy_id "$WORK/$f-step4.txt"; then want "D3_${f}_step4_lacks_wallet_policy_id" no
  else want "D3_${f}_step4_lacks_wallet_policy_id" yes; fi
done

# Consumer crate from the guide.
awk '/^## Cargo dependency/{s=1} s&&/^```toml/{p=1;next} p&&/^```/{exit} p' "$GUIDE" > "$WORK/manifest.txt"
awk '/^## Encoding an mk1 card from a `KeyCard`/{s=1} s&&/^```rust/{p=1;next} p&&/^```/{exit} p' "$GUIDE" > "$WORK/example.rs"
[ -s "$WORK/manifest.txt" ] && [ -s "$WORK/example.rs" ] || { bad C0_extract "guide blocks not found"; exit 1; }
# consumer NAME MANIFEST_FILE: builds and runs; sets RC, OUT, ERR.
consumer() {
  local d=$WORK/$1; rm -rf "$d"; mkdir -p "$d/src"
  { printf '[package]\nname = "ffconsumer"\nversion = "0.0.0"\nedition = "2021"\n\n'; cat "$2"; printf '\n[workspace]\n'; } > "$d/Cargo.toml"
  { echo 'fn main() -> Result<(), Box<dyn std::error::Error>> {'; cat "$WORK/example.rs"; echo 'Ok(()) }'; } > "$d/src/main.rs"
  RC=0; (cd "$d" && CARGO_TARGET_DIR=$WORK/ctarget cargo +1.85.0 run -q) > "$d/out" 2> "$d/err" || RC=$?
  OUT=$d/out; ERR=$d/err
}
consumer C1 "$WORK/manifest.txt"
if [ "$MODE" = red ]; then
  if [ "$RC" != 0 ] && grep -qF 'mk-codec-v0.2.2' "$ERR"; then ok "C1_guide_as_printed rc=$RC (tag mk-codec-v0.2.2 not found)"
  else bad C1_guide_as_printed "rc=$RC; err: $(tail -3 "$ERR")"; fi
  # C2: the tag alone corrected: the example still does not compile (no bitcoin dependency).
  sed 's/mk-codec-v0\.2\.2/mk-codec-v0.5.1/' "$WORK/manifest.txt" > "$WORK/m2.txt"
  consumer C2 "$WORK/m2.txt"
  if [ "$RC" = 101 ] && grep -qF 'E0433' "$ERR" && grep -qF '`bitcoin`' "$ERR"; then ok "C2_tag_fixed_only rc=$RC (E0433 bitcoin)"
  else bad C2_tag_fixed_only "rc=$RC; err: $(tail -3 "$ERR")"; fi
  # C3: tag and bitcoin corrected: the elided xpub panics at parse.
  { cat "$WORK/m2.txt"; echo 'bitcoin = "0.32"'; } > "$WORK/m3.txt"
  consumer C3 "$WORK/m3.txt"
  if [ "$RC" = 101 ] && grep -qF 'InvalidCharacterError { invalid: 46 }' "$ERR"; then ok "C3_tag_and_bitcoin_fixed rc=$RC (elided xpub panics)"
  else bad C3_tag_and_bitcoin_fixed "rc=$RC; err: $(tail -3 "$ERR")"; fi
else
  n=$(grep -c . "$OUT" || true); m=$(grep -c '^mk1' "$OUT" || true)
  if [ "$RC" = 0 ] && [ "$n" -ge 1 ] && [ "$n" = "$m" ]; then ok "C1_guide_as_printed rc=0 ($n mk1 line(s))"
  else bad C1_guide_as_printed "rc=$RC lines=$n mk1=$m; err: $(tail -3 "$ERR")"; fi
fi

if [ "$FAIL" = 0 ]; then echo "ALL ROWS MATCH $MODE"; else echo "ROWS FAILED $MODE"; exit 1; fi
