#!/usr/bin/env bash
# RED/GREEN rows for design/FIX_mk_fd_derive_guards.md (bughunt-1119 F-D: bh-mk-C1, bh-mk-I18).
#
# What it does: builds `mk` (cargo build --locked -p mk-cli, the repo's pinned
# 1.85 toolchain, no --release), mints three public-key fixture cards with
# `mk encode`, then runs every CLI row the document prints and checks it against
# the MODE's expectation:
#   red   = the base commit's behaviour (each defect shows; controls pass)
#   green = the fixed behaviour (refusals exit 64, nothing on stdout but the
#           --json error envelope; controls unchanged)
# Any row that does not match its expectation fails the script (exit 1).
#
# What it does NOT cover: the exact refusal wording of C1 (open: see the FIX
# document's STOP; green checks only exit 64 and that no child address/xpub is
# printed); the nextest rows in crates/mk-cli/tests/bh1119_derive_guards.rs; fmt,
# clippy, the full suite, the toolkit manual; non-Linux CI legs.
#
# Usage: scripts/mk-fd-derive-guards-rows.sh red|green   (run from the repo root)
set -euo pipefail
export PATH="$HOME/.cargo/bin:$PATH"
MODE=${1:?usage: $0 red|green}
case "$MODE" in red|green) ;; *) echo "usage: $0 red|green" >&2; exit 2;; esac
REPO=$(git rev-parse --show-toplevel)
WORK=${WORK:-/scratch/cache/mk-fd-derive-guards-rows}
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$REPO/target}
case "$WORK$CARGO_TARGET_DIR" in /tmp*) echo "refusing /tmp" >&2; exit 2;; esac
cargo --version | grep -q '^cargo 1\.85\.' || { echo "cargo is not the 1.85 pin: $(cargo --version)" >&2; exit 2; }
(cd "$REPO" && cargo build --locked -q -p mk-cli)
MK="$CARGO_TARGET_DIR/debug/mk"
mkdir -p "$WORK"
cd "$WORK"
rm -f leaf.mk1 acct.mk1 master.mk1

# Fixtures (public keys only).
# LEAF: the generator point G with an all-zero chain code at depth 0 = the toolkit
# fixed-key card shape (mnemonic-toolkit cmd/bundle.rs {wif} slot).
LEAF=tpubD6NzVbkrYhZ4WLczPJWReQycCJdd6YVWXubbVUFnJ5KgU5MDQrD998ZJLT5hqNKRA9cJzkh4D7oXwCpwFUWWWkHR9zPeBk1hnWBJKNhyYsn
# X84: BIP-84 account xpub (mk-codec corpus V2), origin m/84'/0'/0'.
X84=xpub6BmeGmRo4LosAcU21HDaGcvtaQ7GrqQcY48nBkE22qM6KVwQUjRJ1BGzk84SFVHgLcd61Vcnhr8petHexjjn5WbQ9PriVrRhphw4oCp2z6a
# TV1M: BIP-32 test vector 1 master xpub (depth 0, real chain code): must keep deriving.
TV1M=xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8
"$MK" encode --xpub "$LEAF" --origin-path m --privacy-preserving --policy-id-stub deadbeef --out leaf.mk1 2>/dev/null
"$MK" encode --xpub "$X84" --origin-path "m/84'/0'/0'" --origin-fingerprint 73c5da0a --policy-id-stub deadbeef --out acct.mk1 2>/dev/null
"$MK" encode --xpub "$TV1M" --origin-path m --origin-fingerprint 3442193e --policy-id-stub deadbeef --out master.mk1 2>/dev/null

FAIL=0
# row NAME EXPECTED_RC STDOUT_MUST STDOUT_MUST_NOT STDERR_MUST -- ARGS...
# ("-" = no check)
row() {
  local name=$1 want=$2 must=$3 mustnot=$4 errmust=$5; shift 6
  local rc=0 f=${name//\//_}
  "$MK" "$@" >"$f.out" 2>"$f.err" || rc=$?
  local ok=1
  [ "$rc" = "$want" ] || ok=0
  [ "$must" = - ] || grep -qF -- "$must" "$f.out" || ok=0
  [ "$mustnot" = - ] || ! grep -qF -- "$mustnot" "$f.out" || ok=0
  [ "$errmust" = - ] || grep -qF -- "$errmust" "$f.err" || ok=0
  if [ $ok = 1 ]; then echo "OK   $MODE $name rc=$rc"; else
    echo "FAIL $MODE $name rc=$rc (want $want)"; sed 's/^/  out: /' "$f.out"; sed 's/^/  err: /' "$f.err"; FAIL=1; fi
}
CHILD=tb1qgl57xjd595rrxmpwys85q40xh4nmqunqrmrmwp
if [ "$MODE" = red ]; then
  row fd_c1_address_refuses_fixed_key_card 0 "$CHILD" - "not the canonical account depth 3" -- address --in leaf.mk1 --address-type p2wpkh --network signet --count 1
  row fd_c1_derive_refuses_fixed_key_card 0 "child_xpub:" - - -- derive --in leaf.mk1 --index 0
  for p in '' m m/; do
    row "fd_i18_derive_refuses_empty_path[${p:-empty}]" 0 "\"child_xpub\":\"$X84\"" - - -- derive --in acct.mk1 --path "$p" --json
  done
else
  row fd_c1_address_refuses_fixed_key_card 64 - "$CHILD" - -- address --in leaf.mk1 --address-type p2wpkh --network signet --count 1
  row fd_c1_address_refuses_fixed_key_card_json 64 '"kind":"UsageError"' "$CHILD" - -- address --in leaf.mk1 --address-type p2wpkh --network signet --count 1 --json
  row fd_c1_derive_refuses_fixed_key_card 64 - "child_xpub" - -- derive --in leaf.mk1 --index 0
  row fd_c1_derive_refuses_fixed_key_card_path 64 - "child_xpub" - -- derive --in leaf.mk1 --path m/0/0
  for p in '' m m/; do
    row "fd_i18_derive_refuses_empty_path[${p:-empty}]" 64 '"kind":"UsageError"' "child_xpub" - -- derive --in acct.mk1 --path "$p" --json
  done
  row "fd_i18_derive_refuses_empty_path[text]" 64 - "child_xpub" "derive needs at least one step" -- derive --in acct.mk1 --path ''
fi
# Controls: identical in both modes.
row fd_c1_master_card_still_derives_address 0 bc1qp5wfcq48h6d63wyy9qz0awtpfqwwv4sma86mhz - "not the canonical account depth 3" -- address --in master.mk1 --address-type p2wpkh --count 1
row fd_c1_master_card_still_derives_xpub 0 xpub6AvUGrnEpfvJ8L7GLRkBTByQ9uBvUHp9o5VxHrFxhvzV4dSWkySpNaBoLR9FpbnwRmTa69yLHF3QfcaxbWT7gWdwws5k4dpmJvqpEuMWwnj - - -- derive --in master.mk1 --index 0 --json
row fd_i18_nonempty_path_still_derives 0 xpub6Fec2dLaQAfWYmg8iGSbMv9ay8WfvB8Rc8H9Koys6THpADfT7ewgxtBUc7hvvA8WarLUqwZqwniTmpFCpqmdoS2y3KGSLoNk2TpEQv8oYaf - - -- derive --in acct.mk1 --path m/0/0 --json
[ $FAIL = 0 ] && echo "ALL ROWS MATCH $MODE" || { echo "ROWS DIFFER FROM $MODE"; exit 1; }
