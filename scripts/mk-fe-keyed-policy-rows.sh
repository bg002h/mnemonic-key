#!/usr/bin/env bash
# RED/GREEN rows for design/FIX_mk_fe_keyed_policy.md (bughunt-1119 F-E: bh-mk-I2,
# bh-mk-I15, sweep-1122 row 15 mk-keys-contradictory-records, mk-dup-stub).
#
# What it does: builds `mk` (cargo build --locked -p mk-cli, the repo's pinned
# 1.85 toolchain, no --release), writes six public md1 fixture policies and
# the key files below under $WORK, runs every CLI row the document prints and
# checks it against the MODE's expectation:
#   red   = the base commit's behaviour (each defect mints at exit 0; controls pass)
#   green = the fixed behaviour (each refusal exits 64 -- since the exit-code spec 1 for an origin
#           that contradicts its policy, 2 for the rest -- with the exact section-2
#           text on stderr and nothing on stdout; controls unchanged)
# Rows: 32 (red and green). Any row that does not match its expectation fails the script (exit 1).
#
# Exact-text rows: a MUST/ERRMUST starting with "=" compares the WHOLE stream to
# the rest (plus a newline; "=" alone means empty).
#
# What it does NOT cover: the nextest-only rows (crates/mk-cli/tests/bh1119_keyed_policy.rs:
# the --in - stdin row and the three I15 plant-proven tests);
# the I15 plants themselves; fmt, clippy, the full suite, the toolkit manual;
# non-Linux CI legs.
#
# Usage: scripts/mk-fe-keyed-policy-rows.sh red|green   (run from the repo root)
set -euo pipefail
export PATH="$HOME/.cargo/bin:$PATH"
MODE=${1:?usage: $0 red|green}
case "$MODE" in red|green) ;; *) echo "usage: $0 red|green" >&2; exit 2;; esac
REPO=$(git rev-parse --show-toplevel)
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$REPO/target}
WORK=${WORK:-$REPO/target/mk-fe-rows}
case "$WORK$CARGO_TARGET_DIR" in /tmp*) echo "refusing /tmp" >&2; exit 2;; esac
cargo --version | grep -q '^cargo 1\.85\.' || { echo "cargo is not the 1.85 pin: $(cargo --version)" >&2; exit 2; }
(cd "$REPO" && cargo build --locked -q -p mk-cli)
MK="$CARGO_TARGET_DIR/debug/mk"
mkdir -p "$WORK"
cd "$WORK"

# Keys (public only). X0, X1, X2 = keys_batch.rs KEYS[0..3] xpubs.
X0=xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf
X1=xpub6DzhyrnFFYQ1HimDiM388xHnDiRPNdZJFBmmxge3Y1WWcHLtMJLfRuhRHqnQCPbTj3fGKTuKFLHzzwpJkp5Dtc3UtLKZKaVZe1yqMBXd6Vk
X2=xpub6EGx8sPr9FxPPE1rbZazhqWwpMXA3Hf5DYKtZbL7c4BSddzmQktp96UaTvecEkoCZysuaj79GMCFZYT1KKk7Ph2M3Kf5g8B82KZ8TZ9SKQR
# TV1M: BIP-32 test vector 1 master (fp 3442193e); LEAFM: generator point, zero chain code, depth 0.
TV1M=xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8
LEAFM=xpub661MyMwAqRbcEYS8w7XLSVeEsBXy79zSzH1J8vCdxAZningWLdN3zgtU6QzvJsNBNF5QPBBBg1yVF2LKrcfGdJq86PeLWDMUCYatZPzQu8R
# Policies: wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*)), shared origin m/48'/0'/0'/2'.
# A (stub 38bd7cec): @0 = [73c5da0a]X0, @1 = [aabbccdd]X1 (from_md1_set.rs KEYED_POLICY_A).
cat > a.md1 <<'EOF'
md1fj5r4pspq2tvyyy4qqxppsg2z7z883w6pt24menw3tsf9m5rru59s2su80aw2q4wgdpapguu6w2y2jygprc
md1fj5r4psf875x67p5s3wem7sgluxl3d2a3syx3m7halwd7s7d5e8l2xm3y3xzfmadfj6e2wnj3gvx34m0wnt
md1fj5r4pshsdlkvt6f6cthyl98xtqcj3lluycagp8vv3nmlgam2ug04zw29zsq0u7st858yuyz646z0r98kyg
md1fj5r4pslq8kxupyjz229sgx620d93cwcs6he5skltczfzylx0ndtm9fdvtp6hyhaccqrfrshqj9459eg
EOF
# SPARSE (stub 3a7f6520): only @0 keyed ([73c5da0a]X0); @1 has no key (md 0.20.7 `md encode ... --key @0=X0
# --fingerprint @0=73c5da0a --path m/48'/0'/0'/2' --force-chunked`).
cat > sparse.md1 <<'EOF'
md1fmpkgpqpq2tvyyy4qqxppsg2z6zw0za5zs4gy4m5rru59s2su80aq5sat59s7xnghe
md1fmpkgpq09q2hyxs7s5yl6smtcxjz9m806prlsm794tkxqs6806lhs8mg2jx2us05u6
md1fmpkgpqkumapumfj075dhzfzvynh66n94j5lcxlmx9ayav9mj0jjqgknzrt66ggpxt
EOF
# REP (stub 58e73a11): A with @1's key replaced by X0 (md_codec 0.42 reassemble, edit, split;
# `md encode` refuses this shape, md-codec 0.42 decodes it).
cat > rep.md1 <<'EOF'
md1fyw09pspq2tvyyy4qqxppsg2z7z883w6pt24menw3tsf9m5rru59s2su80aw2q4wgdpapgv29jfttksg5j3
md1fyw09psf875x67p5s3wem7sgluxl3d2a3syx3m7halwd7s7d5e8l2xm3y3xzfmadfj6e2wryw50fs300mzz
md1fyw09pshsdlkvt6f6cthyl98waqcl9pvz58pmltjs9tjrg0g2z0agd4urfpzanhaq3lcd7yx9mq49jevw95
md1fyw09psut2hvvpp5wl4l0mn058ndxfl63kufyfsjwlt2vkk2nlqmlvch5n4shwf72gqmn8m0alwh8lcz
EOF
# NOFP: A with no fingerprint TLV (same strings as `md encode` with --key @0=X0 --key @1=X1, no --fingerprint).
cat > nofp.md1 <<'EOF'
md1fhd3vpspq2tvyyy4qqxppsg29wpyhwsv0jskp2rsal4egz4ep5859p875x67p5s3w6jy5p92uf5xrp
md1fhd3vpswwl5z8lphut2hvvpp5wl4l0mn058ndxfl63kufyfsjwlt2vkk2nlqmlvch0wqccjd2vncvg
md1fhd3vps4yav9mj0jnn9svfgll7zvw5qnkxgeal5wa4wy86389z3gq870g9n6rjwuqgwh986tdk7dxr
md1fhd3vpslkxupyjz229sgx620d93cwcs6he5skltczfzylx0ndtm9fdvtp6hyhacczakufnwezg4mz
EOF
# ELIDED (stub 38bd7cec): the same policy as A with the origin elided on the wire (md 0.20.7 `md encode
# 'wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))' --key @0=X0 --key @1=X1 --fingerprint @0=73c5da0a --fingerprint
# @1=aabbccdd`, no --path; --network signet with the tpub spellings prints the same strings).
cat > elided.md1 <<'EOF'
md1fwlhkpspqggqpsgvzzshs3eutks264w7vm52uzfwaqcl9pvz58pmltjs9tjrg0g2z0agqv9emggu04pqgg
md1fwlhkpsw67p5s3wem7sgluxl3d2a3syx3m7halwd7s7d5e8l2xm3y3xzfmadfj6e20urqdw3qtwekpkf7e
md1fwlhkpsh7e30f8tpwunu5uevrz28llsnr4qya3jx00arhdt3p75feg52qpln6pv7sunhqpxvxsstm2usu6
md1fwlhkpscrmrwqjfp99zcyrd98kjcu8vgdtu6gt04upy3z0n8ek4aj5kk9satjt7uvlfas3hpkyv6wk
EOF
# SIGNET (stub 00912ee3): the same command plus --path "m/48'/1'/0'/2'" (inline-origin route).
cat > signet.md1 <<'EOF'
md1fx6kzpspq2tvywz2sqrqscy9p0prnchdq442auehg4cyja6p372zc9gwrh7h9q2hyxs7s5mmuyg5wnu8kyf
md1fx6kzpsgnl2rd0q6gghvalgy07r0ck4wcczrgalt7lhxlg0x6vnl4rdcjgnpya7k5edv4x5l8gpre2xywzw
md1fx6kzpshcxlmx9ayav9mj0jnn9svfgll7zvw5qnkxgeal5wa4wy86389z3gq870g9n6rjwpwmec4za9gk4u
md1fx6kzpsmsrmrwqjfp99zcyrd98kjcu8vgdtu6gt04upy3z0n8ek4aj5kk9satjt7uvqwz5x5s5nw4z9m
EOF
# X0D0: X0's chain code and key re-serialized at depth 0 (parent fp 00000000, child 0).
X0D0=xpub661MyMwAqRbcGQnC8zMGwRc4EXYJCgXrxx9kXtw1RXqu4TcW26PxHqssgp6sU4N5CR5o9QcUZPG31fzPeUHEVPkFit1WpVZTmqZLzpvZG2s
TPL=md1yqpqqxzq2qwfv8urt848e   # keyless template wpkh, stub 559e64b2
P0="48'/0'/0'/2'"
printf '[73c5da0a/%s]%s\n[73c5da0a/%s]%s\n' "$P0" "$X0" "$P0" "$X0" > twice.txt
printf '[73c5da0a/%s]%s\n# same record, h spelling\n[73c5da0a/48h/0h/0h/2h]%s\n' "$P0" "$X0" "$X0" > twice_h.txt
printf "[73c5da0a/%s]%s\n[73c5da0a/48'/0'/1'/2']%s\n[73c5da0a/%s]%s\n" "$P0" "$X0" "$X1" "$P0" "$X0" > rep13.txt
printf "[73c5da0a/%s]%s\n[73c5da0a/48'/0'/1'/2']%s\n" "$P0" "$X0" "$X0" > twoorigins.txt
printf '[73c5da0a/%s]%s\n\n[73c5da0a/%s]%s\n' "$P0" "$X0" "$P0" "$X1" > twokeys.txt
printf '[3442193e/]%s\n[3442193e/]%s\n' "$TV1M" "$LEAFM" > twomasters.txt
printf "[73c5da0a/48'/0'/9'/2']%s\n" "$X0" > batch9.txt
printf "[73c5da0a/%s]%s\n[73c5da0a/48'/0'/1'/2']%s\n[73c5da0a/48'/0'/2'/2']%s\n" "$P0" "$X0" "$X1" "$X2" > distinct.txt
printf '[73c5da0a/%s]%s\n[aabbccdd/%s]%s\n' "$P0" "$X0" "$P0" "$X1" > cosigners.txt

FAIL=0
cmpx() { if [ -z "$1" ]; then [ ! -s "$2" ]; else [ "$(cat "$2"; echo x)" = "$1"$'\n'x ]; fi; }
# row NAME EXPECTED_RC STDOUT_MUST STDOUT_MUST_NOT STDERR_MUST -- ARGS...   ("-" = no check)
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
# Section 2 texts (T-PATH, T-PATH-STD, T-FP, T-REPKEY, T-REC, T-KEY2, T-ORIGIN2, T-STUB).
tpath() { printf '%s' "xpub $1 is @$2 of the wallet policy with stub $3, whose origin path for it is $4; the card declares $5. A card whose origin contradicts its own policy is refused; no card was minted."; }
tpathstd() { printf '%s' "xpub $1 is @$2 of the wallet policy with stub $3, which states no origin path for it, so its policy id uses the standard origin path for its template, $4; the card declares $5. A card whose origin contradicts its own policy is refused; no card was minted. If $5 is this key's origin, re-encode the md1 with the keys' real origins (md encode --path, or the origin in the template) and mint against the new md1."; }
tfp() { printf '%s' "xpub $1 is @$2 of the wallet policy with stub $3, whose master fingerprint for it is $4; the card declares $5. A card whose origin contradicts its own policy is refused; no card was minted."; }
trepkey() { printf '%s' "the wallet policy with stub $1 holds xpub $2 at $3; BIP 388 requires the keys of a policy to be pairwise distinct, so the policy is refused and no card was minted."; }
trec() { printf '%s' "$1:$2: this record repeats line $3; a key is carded once, so a repeated record is refused and no card was minted."; }
tkey2() { printf '%s' "$1:$2: this record's key is also on line $3, with origin $4; one key has one origin, so the two records contradict each other and no card was minted."; }
torigin2() { printf '%s' "$1:$2: this record's origin $3 is also on line $4, with a different key; one origin derives one key (BIP 32), so the two records contradict each other and no card was minted."; }
tstub() { printf '%s' "policy stub $1 is bound twice; a card lists each policy once, so a stub repeated across --policy-id-stub, --from-md1 and --from-md1-set is refused and no card was minted."; }
M9="m/48'/0'/9'/2'"; M0="m/$P0"; M1="m/48'/0'/1'/2'"
if [ "$MODE" = red ]; then
  row fe_i2_path_contradicts_policy 0 mk1 - - -- encode --xpub "$X0" --origin-path "$M9" --origin-fingerprint 73c5da0a --from-md1-set a.md1
  row fe_i2_path_contradicts_policy_json 0 '"mk1_strings"' - - -- encode --xpub "$X0" --origin-path "$M9" --origin-fingerprint 73c5da0a --from-md1-set a.md1 --json
  row fe_i2_fingerprint_contradicts_policy 0 mk1 - - -- encode --xpub "$X0" --origin-path "$M0" --origin-fingerprint deadbeef --from-md1-set a.md1
  row fe_i2_second_slot_contradicts_policy 0 mk1 - - -- encode --xpub "$X1" --origin-path "$M1" --origin-fingerprint 73c5da0a --from-md1-set a.md1
  row fe_i2_sparse_slot_contradicts_policy 0 mk1 - - -- encode --xpub "$X0" --origin-path "$M9" --origin-fingerprint 73c5da0a --from-md1-set sparse.md1
  row fe_i2_batch_record_contradicts_policy 0 mk1 - - -- encode --keys batch9.txt --from-md1-set a.md1
  row fe_i2_policy_repeats_key 0 mk1 - - -- encode --xpub "$X0" --origin-path "$M0" --origin-fingerprint 73c5da0a --from-md1-set rep.md1
  row fe_i2_elided_origin_path_contradicts_policy 0 mk1 - - -- encode --xpub "$X0" --origin-path "$M9" --origin-fingerprint 73c5da0a --from-md1-set elided.md1
  row fe_i2_elided_origin_signet_card 0 mk1 - - -- encode --xpub "$X0" --origin-path "m/48'/1'/0'/2'" --origin-fingerprint 73c5da0a --from-md1-set elided.md1
  row fe_i2_elided_origin_master_card 0 mk1 - - -- encode --xpub "$X0D0" --origin-path "m" --privacy-preserving --from-md1-set elided.md1
  row fe_keys_identical_record_twice 0 mk1 - "card 2 of 2" -- encode --keys twice.txt --policy-id-stub deadbeef
  row fe_keys_identical_record_h_spelling 0 mk1 - "card 2 of 2" -- encode --keys twice_h.txt --policy-id-stub deadbeef
  row fe_keys_identical_record_non_adjacent 0 mk1 - "card 3 of 3" -- encode --keys rep13.txt --policy-id-stub deadbeef
  row fe_keys_one_key_two_origins 0 mk1 - "card 2 of 2" -- encode --keys twoorigins.txt --policy-id-stub deadbeef
  row fe_keys_one_origin_two_keys 0 mk1 - "card 2 of 2" -- encode --keys twokeys.txt --policy-id-stub deadbeef
  row fe_keys_one_origin_two_masters 64 = - "does not match the xpub" -- encode --keys twomasters.txt --policy-id-stub deadbeef
  row fe_stub_repeated_policy_id_stub 0 mk1 - - -- encode --xpub "$X0" --origin-path "$M0" --origin-fingerprint 73c5da0a --policy-id-stub deadbeef --policy-id-stub deadbeef
  row fe_stub_repeated_non_adjacent 0 mk1 - - -- encode --xpub "$X0" --origin-path "$M0" --origin-fingerprint 73c5da0a --policy-id-stub deadbeef --policy-id-stub 11111111 --policy-id-stub deadbeef
  row fe_stub_repeated_non_adjacent_tail 0 mk1 - - -- encode --xpub "$X0" --origin-path "$M0" --origin-fingerprint 73c5da0a --policy-id-stub deadbeef --policy-id-stub 11111111 --policy-id-stub 11111111
  row fe_stub_repeated_from_md1 0 mk1 - - -- encode --xpub "$X0" --origin-path "$M0" --origin-fingerprint 73c5da0a --from-md1 "$TPL" --from-md1 "$TPL"
  row fe_stub_repeated_across_channels 0 mk1 - - -- encode --xpub "$X0" --origin-path "$M0" --origin-fingerprint 73c5da0a --policy-id-stub 559e64b2 --from-md1 "$TPL"
else
  row fe_i2_path_contradicts_policy 1 = - "=error: $(tpath "$X0" 0 38bd7cec "$M0" "$M9")" -- encode --xpub "$X0" --origin-path "$M9" --origin-fingerprint 73c5da0a --from-md1-set a.md1
  row fe_i2_path_contradicts_policy_json 1 "={\"error\":{\"details\":null,\"exit_code\":1,\"kind\":\"Mismatch\",\"message\":\"$(tpath "$X0" 0 38bd7cec "$M0" "$M9")\"},\"schema_version\":1}" - = -- encode --xpub "$X0" --origin-path "$M9" --origin-fingerprint 73c5da0a --from-md1-set a.md1 --json
  row fe_i2_fingerprint_contradicts_policy 1 = - "=error: $(tfp "$X0" 0 38bd7cec 73c5da0a deadbeef)" -- encode --xpub "$X0" --origin-path "$M0" --origin-fingerprint deadbeef --from-md1-set a.md1
  row fe_i2_second_slot_contradicts_policy 1 = - "=error: $(tpath "$X1" 1 38bd7cec "$M0" "$M1")" -- encode --xpub "$X1" --origin-path "$M1" --origin-fingerprint 73c5da0a --from-md1-set a.md1
  row fe_i2_sparse_slot_contradicts_policy 1 = - "=error: $(tpath "$X0" 0 3a7f6520 "$M0" "$M9")" -- encode --xpub "$X0" --origin-path "$M9" --origin-fingerprint 73c5da0a --from-md1-set sparse.md1
  row fe_i2_batch_record_contradicts_policy 1 = - "=error: $(tpath "$X0" 0 38bd7cec "$M0" "$M9")" -- encode --keys batch9.txt --from-md1-set a.md1
  row fe_i2_policy_repeats_key 2 = - "=error: $(trepkey 58e73a11 "$X0" "@0, @1")" -- encode --xpub "$X0" --origin-path "$M0" --origin-fingerprint 73c5da0a --from-md1-set rep.md1
  row fe_i2_elided_origin_path_contradicts_policy 1 = - "=error: $(tpathstd "$X0" 0 38bd7cec "$M0" "$M9")" -- encode --xpub "$X0" --origin-path "$M9" --origin-fingerprint 73c5da0a --from-md1-set elided.md1
  row fe_i2_elided_origin_signet_card 1 = - "=error: $(tpathstd "$X0" 0 38bd7cec "$M0" "m/48'/1'/0'/2'")" -- encode --xpub "$X0" --origin-path "m/48'/1'/0'/2'" --origin-fingerprint 73c5da0a --from-md1-set elided.md1
  row fe_i2_elided_origin_master_card 1 = - "=error: $(tpathstd "$X0D0" 0 38bd7cec "$M0" m)" -- encode --xpub "$X0D0" --origin-path "m" --privacy-preserving --from-md1-set elided.md1
  row fe_keys_identical_record_twice 2 = - "=error: $(trec twice.txt 2 1)" -- encode --keys twice.txt --policy-id-stub deadbeef
  row fe_keys_identical_record_h_spelling 2 = - "=error: $(trec twice_h.txt 3 1)" -- encode --keys twice_h.txt --policy-id-stub deadbeef
  row fe_keys_identical_record_non_adjacent 2 = - "=error: $(trec rep13.txt 3 1)" -- encode --keys rep13.txt --policy-id-stub deadbeef
  row fe_keys_one_key_two_origins 2 = - "=error: $(tkey2 twoorigins.txt 2 1 "[73c5da0a/$P0]")" -- encode --keys twoorigins.txt --policy-id-stub deadbeef
  row fe_keys_one_origin_two_keys 2 = - "=error: $(torigin2 twokeys.txt 3 "[73c5da0a/$P0]" 1)" -- encode --keys twokeys.txt --policy-id-stub deadbeef
  row fe_keys_one_origin_two_masters 2 = - "=error: $(torigin2 twomasters.txt 2 "[3442193e/]" 1)" -- encode --keys twomasters.txt --policy-id-stub deadbeef
  row fe_stub_repeated_policy_id_stub 2 = - "=error: $(tstub deadbeef)" -- encode --xpub "$X0" --origin-path "$M0" --origin-fingerprint 73c5da0a --policy-id-stub deadbeef --policy-id-stub deadbeef
  row fe_stub_repeated_non_adjacent 2 = - "=error: $(tstub deadbeef)" -- encode --xpub "$X0" --origin-path "$M0" --origin-fingerprint 73c5da0a --policy-id-stub deadbeef --policy-id-stub 11111111 --policy-id-stub deadbeef
  row fe_stub_repeated_non_adjacent_tail 2 = - "=error: $(tstub 11111111)" -- encode --xpub "$X0" --origin-path "$M0" --origin-fingerprint 73c5da0a --policy-id-stub deadbeef --policy-id-stub 11111111 --policy-id-stub 11111111
  row fe_stub_repeated_from_md1 2 = - "=error: $(tstub 559e64b2)" -- encode --xpub "$X0" --origin-path "$M0" --origin-fingerprint 73c5da0a --from-md1 "$TPL" --from-md1 "$TPL"
  row fe_stub_repeated_across_channels 2 = - "=error: $(tstub 559e64b2)" -- encode --xpub "$X0" --origin-path "$M0" --origin-fingerprint 73c5da0a --policy-id-stub 559e64b2 --from-md1 "$TPL"
fi
# Controls: identical in both modes, except two exit codes the exit-code spec
# (2026-11-26) moved after this FIX: a non-member 64 -> 1, an unreadable md1 1 -> 2.
NONMEMBER_RC=1; MD1_RC=2; [ "$MODE" = red ] && { NONMEMBER_RC=64; MD1_RC=1; }
row fe_control_true_origin_mints 0 mk1 - "1 of them carded here, 1 not carded" -- encode --xpub "$X0" --origin-path "$M0" --origin-fingerprint 73c5da0a --from-md1-set a.md1
row fe_control_privacy_preserving_mints 0 mk1 - - -- encode --xpub "$X0" --origin-path "$M0" --privacy-preserving --from-md1-set a.md1
row fe_control_policy_without_fingerprints_mints 0 mk1 - - -- encode --xpub "$X0" --origin-path "$M0" --origin-fingerprint deadbeef --from-md1-set nofp.md1
row fe_control_sparse_unfilled_slot_mints 0 mk1 - - -- encode --xpub "$X1" --origin-path "$M1" --origin-fingerprint 73c5da0a --from-md1-set sparse.md1
row fe_control_complete_policy_nonmember_refused "$NONMEMBER_RC" = - "is not a cosigner of the wallet policy with stub 38bd7cec" -- encode --xpub "$X2" --origin-path "m/48'/0'/2'/2'" --origin-fingerprint 73c5da0a --from-md1-set a.md1
row fe_control_distinct_records_mint 0 mk1 - "card 3 of 3" -- encode --keys distinct.txt --policy-id-stub deadbeef
row fe_control_both_cosigners_mint 0 mk1 - "card 2 of 2" -- encode --keys cosigners.txt --from-md1-set a.md1
row fe_control_two_distinct_stubs_mint 0 mk1 - - -- encode --xpub "$X0" --origin-path "$M0" --origin-fingerprint 73c5da0a --policy-id-stub deadbeef --from-md1 "$TPL"
row fe_control_chunk_set_given_twice_unchanged "$MD1_RC" = - "chunk set incomplete: got 8 chunks, expected 4" -- encode --xpub "$X0" --origin-path "$M0" --origin-fingerprint 73c5da0a --from-md1-set a.md1 --from-md1-set a.md1
row fe_control_md_default_elided_origin_mints 0 mk1 - "policy 38bd7cec has 2 cosigner(s); 1 of them carded here, 1 not carded" -- encode --xpub "$X0" --origin-path "$M0" --origin-fingerprint 73c5da0a --from-md1-set elided.md1
row fe_control_signet_card_against_path_md1_mints 0 mk1 - "policy 00912ee3 has 2 cosigner(s); 1 of them carded here, 1 not carded" -- encode --xpub "$X0" --origin-path "m/48'/1'/0'/2'" --origin-fingerprint 73c5da0a --from-md1-set signet.md1
[ $FAIL = 0 ] && echo "ALL ROWS MATCH $MODE" || { echo "ROWS DIFFER FROM $MODE"; exit 1; }
