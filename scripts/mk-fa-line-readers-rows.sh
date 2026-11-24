#!/usr/bin/env bash
# RED/GREEN rows for design/FIX_mk_fa_line_readers.md (bughunt-1119 F-A: bh-mk-I3, bh-mk-I4).
#
# What it does: builds `mk` (cargo build --locked -p mk-cli, the repo's pinned
# 1.85 toolchain, no --release), writes public-key fixture files (key records for
# `mk encode --keys/--in`, md1 sets for `mk encode --from-md1-set`) in each
# line-ending variant, then runs every CLI row the document prints and checks it
# against the MODE's expectation:
#   red   = the base commit's behaviour (each defect shows; controls pass)
#   green = the fixed behaviour (Brian's ruling B1: a leading BOM and CRLF are
#           normalised, every other CR and every other U+FEFF is refused at exit
#           64 naming the line; controls unchanged)
# Any row that does not match its expectation fails the script (exit 1).
#
# Stream specs: "-" no check; "=TEXT" the whole stream is TEXT plus a newline
# ("=" alone: empty); "~TEXT" contains TEXT; "@FILE" byte-identical to FILE;
# "!@FILE" differs from FILE.
#
# What it does NOT cover: the nextest rows in
# crates/mk-cli/tests/bh1119_line_readers.rs (the builder writes them); the
# mk1 `--in` reader of decode/inspect/verify/address/derive (out of scope, see
# the FIX document section 1); fmt, clippy, the full suite, the toolkit manual;
# non-Linux CI legs.
#
# Usage: scripts/mk-fa-line-readers-rows.sh red|green   (run from the repo root)
set -euo pipefail
export PATH="$HOME/.cargo/bin:$PATH"
MODE=${1:?usage: $0 red|green}
case "$MODE" in red|green) ;; *) echo "usage: $0 red|green" >&2; exit 2;; esac
REPO=$(git rev-parse --show-toplevel)
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$REPO/target}
WORK=${WORK:-$REPO/target/mk-fa-rows}
case "$WORK$CARGO_TARGET_DIR" in /tmp*) echo "refusing /tmp" >&2; exit 2;; esac
cargo --version | grep -q '^cargo 1\.85\.' || { echo "cargo is not the 1.85 pin: $(cargo --version)" >&2; exit 2; }
(cd "$REPO" && cargo build --locked -q -p mk-cli)
MK="$CARGO_TARGET_DIR/debug/mk"
mkdir -p "$WORK"
cd "$WORK"

# Fixtures (public keys only; the mk-cli test corpus: X0/X1 multisig cosigner
# xpubs, X84 the BIP-84 account xpub, A/B the template-md1 pair).
X0=xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf
X1=xpub6DzhyrnFFYQ1HimDiM388xHnDiRPNdZJFBmmxge3Y1WWcHLtMJLfRuhRHqnQCPbTj3fGKTuKFLHzzwpJkp5Dtc3UtLKZKaVZe1yqMBXd6Vk
X84=xpub6BmeGmRo4LosAcU21HDaGcvtaQ7GrqQcY48nBkE22qM6KVwQUjRJ1BGzk84SFVHgLcd61Vcnhr8petHexjjn5WbQ9PriVrRhphw4oCp2z6a
A=md1yqpqqxzq2qwfv8urt848e
B=md1ypfdsssj5qqcyxppggah9rg9v9gjh4
R0="[73c5da0a/48'/0'/0'/2']$X0"
R1="[73c5da0a/48'/0'/1'/2']$X1"
BOM=$'\xef\xbb\xbf'
CR=$'\r'
LF=$'\n'
put() { printf '%s' "$2" >"$1"; }
# Key-record files (--keys / --in).
put k_lf.txt        "$R0 # first$LF$R1$LF"
put k_cr_comment.txt "$R0 # first$CR$R1$LF"
put k_trailing_cr.txt "$R0$LF$R1$CR"
put k_bom_mid.txt   "$R0$LF$BOM$R1$LF"
put k_crlf.txt      "$R0$CR$LF$R1$CR$LF"
put k_bom.txt       "$BOM$R0$LF$R1$LF"
put k_cr_only.txt   "$R0$CR$R1$CR"
# md1-set files (--from-md1-set).
put s_plain.txt      "$A$LF$B$LF"
put s_bom.txt        "$BOM$A$LF$B$LF"
put s_cr_comment.txt "# note$CR$A$LF$B$LF"
put s_bom_mid.txt    "$A$LF$BOM$B$LF"
put s_cr_only.txt    "$A$CR$B$CR"
put s_trailing_cr.txt "$A$LF$B$CR"
put s_crlf.txt       "$A$CR$LF$B$CR$LF"
# A four-chunk keyed policy (from_md1_set.rs KEYED_POLICY_A) and a cosigner of it (V1).
KA=(md1fj5r4pspq2tvyyy4qqxppsg2z7z883w6pt24menw3tsf9m5rru59s2su80aw2q4wgdpapguu6w2y2jygprc
    md1fj5r4psf875x67p5s3wem7sgluxl3d2a3syx3m7halwd7s7d5e8l2xm3y3xzfmadfj6e2wnj3gvx34m0wnt
    md1fj5r4pshsdlkvt6f6cthyl98xtqcj3lluycagp8vv3nmlgam2ug04zw29zsq0u7st858yuyz646z0r98kyg
    md1fj5r4pslq8kxupyjz229sgx620d93cwcs6he5skltczfzylx0ndtm9fdvtp6hyhaccqrfrshqj9459eg)
put c_plain.txt "$(printf '%s\n' "${KA[@]}")$LF"
put c_bom.txt   "$BOM$(printf '%s\n' "${KA[@]}")$LF"
ENC_V1=(encode --xpub "$X0" --origin-fingerprint 73c5da0a --origin-path "m/48'/0'/0'/2'")
"$MK" "${ENC_V1[@]}" --from-md1-set c_plain.txt >want_chunk.out 2>/dev/null
[ -s want_chunk.out ] || { echo "chunked control mint produced nothing" >&2; exit 2; }
ENC_SET=(encode --xpub "$X84" --origin-path "m/84'/0'/0'" --origin-fingerprint 73c5da0a)
"$MK" "${ENC_SET[@]}" --from-md1-set s_plain.txt >want_set.out 2>/dev/null
[ -s want_set.out ] || { echo "control mint produced nothing" >&2; exit 2; }

FAIL=0
# spec SPEC FILE: check FILE against one stream spec (see header).
spec() {
  local s=$1 f=$2
  case "$s" in
    -) return 0 ;;
    =) [ ! -s "$f" ] ;;
    =*) [ "$(cat "$f"; echo x)" = "${s:1}"$'\n'x ] ;;
    "~"*) grep -qF -- "${s:1}" "$f" ;;
    "!@"*) ! cmp -s "${s:2}" "$f" ;;
    @*) cmp -s "${s:1}" "$f" ;;
    *) echo "bad spec $s" >&2; return 1 ;;
  esac
}
# row NAME EXPECTED_RC STDOUT_SPEC STDERR_SPEC -- ARGS...   (stdin from $STDIN, default /dev/null)
row() {
  local name=$1 want=$2 so=$3 se=$4; shift 5
  local rc=0 f=${name//[\/\[\]]/_}
  "$MK" "$@" <"${STDIN:-/dev/null}" >"$f.out" 2>"$f.err" || rc=$?
  local ok=1
  [ "$rc" = "$want" ] || ok=0
  spec "$so" "$f.out" || ok=0
  spec "$se" "$f.err" || ok=0
  if [ $ok = 1 ]; then echo "OK   $MODE $name rc=$rc"; else
    echo "FAIL $MODE $name rc=$rc (want $want)"; sed 's/^/  out: /' "$f.out"; sed 's/^/  err: /' "$f.err"; FAIL=1; fi
}
TWO='~"card_count":2'
ONE='~"card_count":1'
# The section 2 texts (PROPOSED, architect confirms).
kcr() { printf '%s' "$1 $2:$3: this line holds a carriage return (CR) with no line feed (LF) after it, so the text after the CR would be read as part of line $3 and a key record after it could be lost. Convert the file to LF or CRLF line endings."; }
scr() { printf '%s' "--from-md1-set $1:$2: this line holds a carriage return (CR) with no line feed (LF) after it, so the text after the CR would be read as part of line $2 and an md1 string after it could be skipped. Convert the file to LF or CRLF line endings."; }
kbom() { printf '%s' "$1 $2:$3: this line holds a byte-order mark (U+FEFF), which is accepted only as the first character of the file. Remove it (joining two files can leave the second file's mark inside the result)."; }
sbom() { printf '%s' "--from-md1-set $1:$2: this line holds a byte-order mark (U+FEFF), which is accepted only as the first character of the file. Remove it (joining two files can leave the second file's mark inside the result)."; }
ENV1='{"error":{"details":null,"exit_code":64,"kind":"UsageError","message":"'
ENV2='"},"schema_version":1}'
KEYS=(--policy-id-stub deadbeef --json)

if [ "$MODE" = red ]; then
  row fa_i3_keys_file_bare_cr_after_comment_refused 0 "$ONE" - -- encode --keys k_cr_comment.txt "${KEYS[@]}"
  STDIN=k_cr_comment.txt row fa_i3_keys_stdin_bare_cr_after_comment_refused 0 "$ONE" - -- encode --keys - "${KEYS[@]}"
  row fa_i3_in_file_bare_cr_after_comment_refused 0 "$ONE" - -- encode --in k_cr_comment.txt "${KEYS[@]}"
  row fa_i3_keys_trailing_bare_cr_refused 0 "$TWO" - -- encode --keys k_trailing_cr.txt "${KEYS[@]}"
  row fa_i3_keys_bom_after_first_char_refused 64 = "=error: k_bom_mid.txt:2: expected BIP-380 origin notation \`[fingerprint/path]xpub\`, got \"\\u{feff}$R1\"" -- encode --keys k_bom_mid.txt --policy-id-stub deadbeef
  row fa_i4_md1_set_leading_bom_normalised 0 "!@want_set.out" - -- "${ENC_SET[@]}" --from-md1-set s_bom.txt
  row fa_i4_md1_set_leading_bom_chunked_normalised 1 = "=error: md1 input rejected: chunk set incomplete: got 3 chunks, expected 4" -- "${ENC_V1[@]}" --from-md1-set c_bom.txt
  row fa_i4_md1_set_bare_cr_after_comment_refused 0 "!@want_set.out" - -- "${ENC_SET[@]}" --from-md1-set s_cr_comment.txt
  row fa_i4_md1_set_bom_after_first_char_refused 0 "!@want_set.out" - -- "${ENC_SET[@]}" --from-md1-set s_bom_mid.txt
  row fa_i4_md1_set_cr_only_refused 1 = "=error: md1 input rejected: codex32 decode error: character '1' not in codex32 alphabet" -- "${ENC_SET[@]}" --from-md1-set s_cr_only.txt
  row fa_i4_md1_set_trailing_bare_cr_refused 0 "@want_set.out" - -- "${ENC_SET[@]}" --from-md1-set s_trailing_cr.txt
else
  row fa_i3_keys_file_bare_cr_after_comment_refused 64 = "=error: $(kcr --keys k_cr_comment.txt 1)" -- encode --keys k_cr_comment.txt --policy-id-stub deadbeef
  row fa_i3_keys_file_bare_cr_after_comment_refused_json 64 "=$ENV1$(kcr --keys k_cr_comment.txt 1)$ENV2" = -- encode --keys k_cr_comment.txt "${KEYS[@]}"
  STDIN=k_cr_comment.txt row fa_i3_keys_stdin_bare_cr_after_comment_refused 64 = "=error: $(kcr --keys '<stdin>' 1)" -- encode --keys - --policy-id-stub deadbeef
  row fa_i3_in_file_bare_cr_after_comment_refused 64 = "=error: $(kcr --in k_cr_comment.txt 1)" -- encode --in k_cr_comment.txt --policy-id-stub deadbeef
  row fa_i3_keys_trailing_bare_cr_refused 64 = "=error: $(kcr --keys k_trailing_cr.txt 2)" -- encode --keys k_trailing_cr.txt --policy-id-stub deadbeef
  row fa_i3_keys_bom_after_first_char_refused 64 = "=error: $(kbom --keys k_bom_mid.txt 2)" -- encode --keys k_bom_mid.txt --policy-id-stub deadbeef
  row fa_i4_md1_set_leading_bom_normalised 0 "@want_set.out" - -- "${ENC_SET[@]}" --from-md1-set s_bom.txt
  row fa_i4_md1_set_leading_bom_chunked_normalised 0 "@want_chunk.out" - -- "${ENC_V1[@]}" --from-md1-set c_bom.txt
  row fa_i4_md1_set_bare_cr_after_comment_refused 64 = "=error: $(scr s_cr_comment.txt 1)" -- "${ENC_SET[@]}" --from-md1-set s_cr_comment.txt
  row fa_i4_md1_set_bom_after_first_char_refused 64 = "=error: $(sbom s_bom_mid.txt 2)" -- "${ENC_SET[@]}" --from-md1-set s_bom_mid.txt
  row fa_i4_md1_set_cr_only_refused 64 = "=error: $(scr s_cr_only.txt 1)" -- "${ENC_SET[@]}" --from-md1-set s_cr_only.txt
  row fa_i4_md1_set_trailing_bare_cr_refused 64 = "=error: $(scr s_trailing_cr.txt 2)" -- "${ENC_SET[@]}" --from-md1-set s_trailing_cr.txt
fi
# Controls: identical in both modes.
row fa_control_keys_lf_mints_two 0 "$TWO" - -- encode --keys k_lf.txt "${KEYS[@]}"
row fa_control_keys_crlf_mints_two 0 "$TWO" - -- encode --keys k_crlf.txt "${KEYS[@]}"
row fa_control_keys_leading_bom_mints_two 0 "$TWO" - -- encode --keys k_bom.txt "${KEYS[@]}"
row fa_control_keys_cr_only_text_unchanged 64 = "=error: --keys k_cr_only.txt: the file uses CR-only line endings (classic Mac), so every record after the first would be read as part of line 1. Convert it to LF or CRLF." -- encode --keys k_cr_only.txt --policy-id-stub deadbeef
row fa_control_md1_set_crlf_equals_lf 0 "@want_set.out" - -- "${ENC_SET[@]}" --from-md1-set s_crlf.txt
[ $FAIL = 0 ] && echo "ALL ROWS MATCH $MODE" || { echo "ROWS DIFFER FROM $MODE"; exit 1; }
