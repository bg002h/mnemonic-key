#!/usr/bin/env bash
# RED/GREEN rows for design/FIX_mk_fd_derive_guards.md (bughunt-1119 F-D: bh-mk-C1, bh-mk-I18).
#
# What it does: builds `mk` (cargo build --locked -p mk-cli, the repo's pinned
# 1.85 toolchain, no --release), mints four public-key fixture cards with
# `mk encode`, then runs every CLI row the document prints and checks it against
# the MODE's expectation:
#   red   = the base commit's behaviour (each defect shows; controls pass)
#   green = the fixed behaviour (refusals exit 64, nothing on stdout but the
#           --json error envelope; controls unchanged). Since the exit-code spec
#           (2026-11-26) the fixed-key refusal is 2, and a bad --path is 64 before
#           the card is read (R2), so it prints the path's own error.
# Any row that does not match its expectation fails the script (exit 1).
#
# Exact-text rows: a MUST/ERRMUST starting with "=" compares the WHOLE stream to
# the rest (plus a newline; "=" alone means empty). The C1 refusal text is the
# FIX document's section 2.1 text (Brian's ruling, Option B, FIX r1 I-1/I-2), built
# here from the card's xpub string and the TYPE/NETWORK words per row.
#
# What it does NOT cover: the nextest rows in
# crates/mk-cli/tests/bh1119_derive_guards.rs; that the printed `mnemonic convert`
# command runs (the fold report runs the filled and the bare-word forms once); the
# I18 text beyond a substring; fmt, clippy, the full suite, the toolkit manual;
# non-Linux CI legs.
#
# Usage: scripts/mk-fd-derive-guards-rows.sh red|green   (run from the repo root)
set -euo pipefail
export PATH="$HOME/.cargo/bin:$PATH"
MODE=${1:?usage: $0 red|green}
case "$MODE" in red|green) ;; *) echo "usage: $0 red|green" >&2; exit 2;; esac
REPO=$(git rev-parse --show-toplevel)
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$REPO/target}
WORK=${WORK:-$REPO/target/mk-fd-rows}
case "$WORK$CARGO_TARGET_DIR" in /tmp*) echo "refusing /tmp" >&2; exit 2;; esac
cargo --version | grep -q '^cargo 1\.85\.' || { echo "cargo is not the 1.85 pin: $(cargo --version)" >&2; exit 2; }
(cd "$REPO" && cargo build --locked -q -p mk-cli)
MK="$CARGO_TARGET_DIR/debug/mk"
mkdir -p "$WORK"
cd "$WORK"
rm -f leaf.mk1 leafm.mk1 acct.mk1 master.mk1

# Fixtures (public keys only).
# LEAF: the generator point G with an all-zero chain code at depth 0 = the toolkit
# fixed-key card shape (mnemonic-toolkit cmd/bundle.rs {wif} slot).
LEAF=tpubD6NzVbkrYhZ4WLczPJWReQycCJdd6YVWXubbVUFnJ5KgU5MDQrD998ZJLT5hqNKRA9cJzkh4D7oXwCpwFUWWWkHR9zPeBk1hnWBJKNhyYsn
# LEAFM: the same shape with mainnet version bytes (generator point, zero chain code).
LEAFM=xpub661MyMwAqRbcEYS8w7XLSVeEsBXy79zSzH1J8vCdxAZningWLdN3zgtU6QzvJsNBNF5QPBBBg1yVF2LKrcfGdJq86PeLWDMUCYatZPzQu8R
# X84: BIP-84 account xpub (mk-codec corpus V2), origin m/84'/0'/0'.
X84=xpub6BmeGmRo4LosAcU21HDaGcvtaQ7GrqQcY48nBkE22qM6KVwQUjRJ1BGzk84SFVHgLcd61Vcnhr8petHexjjn5WbQ9PriVrRhphw4oCp2z6a
# TV1M: BIP-32 test vector 1 master xpub (depth 0, real chain code): must keep deriving.
TV1M=xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8
"$MK" encode --xpub "$LEAF" --origin-path m --privacy-preserving --policy-id-stub deadbeef --out leaf.mk1 2>/dev/null
"$MK" encode --xpub "$LEAFM" --origin-path m --privacy-preserving --policy-id-stub deadbeef --out leafm.mk1 2>/dev/null
"$MK" encode --xpub "$X84" --origin-path "m/84'/0'/0'" --origin-fingerprint 73c5da0a --policy-id-stub deadbeef --out acct.mk1 2>/dev/null
"$MK" encode --xpub "$TV1M" --origin-path m --origin-fingerprint 3442193e --policy-id-stub deadbeef --out master.mk1 2>/dev/null

FAIL=0
# cmpx TEXT FILE: FILE is exactly TEXT + newline (TEXT empty: FILE empty).
cmpx() { if [ -z "$1" ]; then [ ! -s "$2" ]; else [ "$(cat "$2"; echo x)" = "$1"$'\n'x ]; fi; }
# row NAME EXPECTED_RC STDOUT_MUST STDOUT_MUST_NOT STDERR_MUST -- ARGS...
# ("-" = no check)
row() {
  local name=$1 want=$2 must=$3 mustnot=$4 errmust=$5; shift 6
  local rc=0 f=${name//\//_}
  "$MK" "$@" >"$f.out" 2>"$f.err" || rc=$?
  local ok=1
  [ "$rc" = "$want" ] || ok=0
  if [ "${must:0:1}" = "=" ]; then cmpx "${must:1}" "$f.out" || ok=0
  else [ "$must" = - ] || grep -qF -- "$must" "$f.out" || ok=0; fi
  [ "$mustnot" = - ] || ! grep -qF -- "$mustnot" "$f.out" || ok=0
  if [ "${errmust:0:1}" = "=" ]; then cmpx "${errmust:1}" "$f.err" || ok=0
  else [ "$errmust" = - ] || grep -qF -- "$errmust" "$f.err" || ok=0; fi
  if [ $ok = 1 ]; then echo "OK   $MODE $name rc=$rc"; else
    echo "FAIL $MODE $name rc=$rc (want $want)"; sed 's/^/  out: /' "$f.out"; sed 's/^/  err: /' "$f.err"; FAIL=1; fi
}
CHILD=tb1qgl57xjd595rrxmpwys85q40xh4nmqunqrmrmwp
CHILDM=bc1qgl57xjd595rrxmpwys85q40xh4nmqunqfacg4j
# The C1 refusal text (FIX section 2.1, Brian's ruling B, FIX r1 I-1/I-2).
C1HEAD="this card holds one fixed key (depth 0, all-zero chain code; the toolkit writes a WIF key this way) and has no child keys any wallet watches, so mk derives nothing from it. If this key is one of the keys in a multisig wallet, that wallet's addresses come from its md1 policy card (md address), not from this key alone. The key's own address: mnemonic convert --from xpub="
# c1main TYPE; c1test TYPE NET (the words after --script-type / --network).
c1main() { printf '%s' "${C1HEAD}${LEAFM} --to address --path m --script-type $1"; }
c1test() { printf '%s' "${C1HEAD}${LEAF} --to address --path m --script-type $1 --network $2"; }
# Green: the exit-code spec (2026-11-26) moved the fixed-key refusal from 64 to 2.
ENV1='{"error":{"details":null,"exit_code":2,"kind":"InvalidInput","message":"'
ENV2='"},"schema_version":1}'
if [ "$MODE" = red ]; then
  row fd_c1_address_refuses_fixed_key_card 0 "$CHILD" - "not the canonical account depth 3" -- address --in leaf.mk1 --address-type p2wpkh --network signet --count 1
  row fd_c1_address_refuses_fixed_key_card_mainnet 0 "$CHILDM" - "not the canonical account depth 3" -- address --in leafm.mk1 --address-type p2wpkh --count 1
  row fd_c1_derive_refuses_fixed_key_card 0 "child_xpub:" - - -- derive --in leaf.mk1 --index 0
  row fd_c1_derive_refuses_fixed_key_card_path 0 "child_xpub:" - - -- derive --in leaf.mk1 --path m/0/0
  row fd_c1_derive_refuses_fixed_key_card_mainnet 0 "child_xpub:" - - -- derive --in leafm.mk1 --index 0
  # Order controls: at the base each of these errors fires first (exit 64, its own text) or, for the
  # empty path (I18), the command succeeds; green: the C1 refusal fires first in every case.
  row "fd_c1_refusal_precedes_other_errors[address-type]" 64 - - "--address-type required" -- address --in leaf.mk1
  row "fd_c1_refusal_precedes_other_errors[network]" 64 - - "disagrees with the xpub's network" -- address --in leaf.mk1 --address-type p2wpkh --network mainnet
  row "fd_c1_refusal_precedes_other_errors[hardened]" 64 - - "cannot derive hardened children" -- derive --in leaf.mk1 --path "m/0'"
  row "fd_c1_refusal_precedes_other_errors[invalid]" 64 - - "invalid derivation path" -- derive --in leaf.mk1 --path /
  row "fd_c1_refusal_precedes_other_errors[empty]" 0 "child_xpub:" - - -- derive --in leaf.mk1 --path ''
  for p in '' m m/; do
    row "fd_i18_derive_refuses_empty_path[${p:-empty}]" 0 "\"child_xpub\":\"$X84\"" - - -- derive --in acct.mk1 --path "$p" --json
  done
else
  row fd_c1_address_refuses_fixed_key_card 2 = - "=error: $(c1test p2wpkh signet)" -- address --in leaf.mk1 --address-type p2wpkh --network signet --count 1
  row fd_c1_address_refuses_fixed_key_card_json 2 "=$ENV1$(c1test p2wpkh signet)$ENV2" - = -- address --in leaf.mk1 --address-type p2wpkh --network signet --count 1 --json
  row fd_c1_address_refuses_fixed_key_card_mainnet 2 = - "=error: $(c1main p2wpkh)" -- address --in leafm.mk1 --address-type p2wpkh --count 1
  row fd_c1_derive_refuses_fixed_key_card 2 = - "=error: $(c1test TYPE NETWORK)" -- derive --in leaf.mk1 --index 0
  row fd_c1_derive_refuses_fixed_key_card_path 2 = - "=error: $(c1test TYPE NETWORK)" -- derive --in leaf.mk1 --path m/0/0
  row fd_c1_derive_refuses_fixed_key_card_mainnet 2 = - "=error: $(c1main TYPE)" -- derive --in leafm.mk1 --index 0
  row "fd_c1_refusal_precedes_other_errors[address-type]" 2 = - "=error: $(c1test TYPE NETWORK)" -- address --in leaf.mk1
  row "fd_c1_refusal_precedes_other_errors[network]" 2 = - "=error: $(c1test p2wpkh NETWORK)" -- address --in leaf.mk1 --address-type p2wpkh --network mainnet
  row "fd_c1_refusal_precedes_other_errors[hardened]" 64 = - "cannot derive hardened children" -- derive --in leaf.mk1 --path "m/0'"
  row "fd_c1_refusal_precedes_other_errors[invalid]" 64 = - "invalid derivation path" -- derive --in leaf.mk1 --path /
  row "fd_c1_refusal_precedes_other_errors[empty]" 64 = - "derive needs at least one step" -- derive --in leaf.mk1 --path ''
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
