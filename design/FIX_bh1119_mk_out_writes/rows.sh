#!/usr/bin/env bash
# RED/GREEN rows for design/FIX_bh1119_mk_out_writes.md (bughunt-1119 F-B: bh-mk-I1, bh-mk-I6).
# Author: Claude Opus 5.5 (claude-opus-5-5), 2026-11-24. The builder moves it to
# scripts/mk-out-writes-rows.sh in the GREEN commit (and adds its catalog row).
#
# What it does: builds `mk` (cargo build --locked -p mk-cli, the 1.85 pin, no
# --release) unless MK names a binary, writes public-key fixture files (one
# testnet key record, one keyless md1 template) under WORK, then runs the
# twenty-one CLI rows of the FIX document section 4 and checks each against MODE:
#   red   = the base commit 58701fb (each defect shows; controls pass)
#   green = the fixed behaviour (R1-R4 of the FIX document)
# Any row that does not match fails the script (exit 1); the last line is
# "rows: N passed, M failed".
# "FSIZE8" rows run mk under a file-size limit of 8 bytes with SIGXFSZ ignored,
# so a write past 8 bytes returns EFBIG (a stand-in for a full disk or quota);
# their stdout and stderr go through pipes so the limit does not touch them.
#
# What it does NOT cover: the nextest rows (crates/mk-cli/tests/
# bh1119_mk_out_writes.rs, the builder writes them from section 4); a run
# killed mid-write (the temporary it leaves is io-lib's row, FIX #7); non-Linux
# legs; fmt, clippy, the full suite, vendor freshness, the toolkit manual.
#
# Usage: rows.sh red|green   (from the repo root; WORK and CARGO_TARGET_DIR
# default under target/, never /tmp)
set -uo pipefail
export PATH="$HOME/.cargo/bin:$PATH"
MODE=${1:?usage: $0 red|green}
case "$MODE" in red|green) ;; *) echo "usage: $0 red|green" >&2; exit 2;; esac
REPO=$(git rev-parse --show-toplevel)
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$REPO/target}
WORK=${WORK:-$REPO/target/mk-out-writes-rows}
case "$WORK$CARGO_TARGET_DIR" in /tmp*) echo "refusing /tmp" >&2; exit 2;; esac
if [ -z "${MK:-}" ]; then
  cargo --version | grep -q '^cargo 1\.85\.' || { echo "cargo is not the 1.85 pin: $(cargo --version)" >&2; exit 2; }
  (cd "$REPO" && CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS:-6} cargo build --locked -q -p mk-cli) || exit 2
  MK="$CARGO_TARGET_DIR/debug/mk"
fi
MK=$(realpath "$MK")
mkdir -p "$WORK/ro"; chmod 755 "$WORK/ro"; cd "$WORK" || exit 2
rm -f ro.orig keys.orig F.orig set.orig card.ref

# testnet account key m/84'/1'/0', master fingerprint 73c5da0a (mk-cli fixture V15_84_TEST; public only)
T=tpubDC9Go1KDateW3gS8VXZ6DD1Xu7PgoTdPcf1MX9Z6qVLiHbaeDJ78swPyuQ8YQY19QjtrzkfkZSXwqCcb7XArtid1iLq8Vy55Ydfm4giZh6X
KEYS="[73c5da0a/84'/1'/0']$T"
fresh() {
  rm -f keys F G hl sl lk s.txt set.md1 o.txt e.txt ro/F
  printf '%s\n' "$KEYS" > keys; printf 'PRIOR-CARD\n' > F; chmod 600 F
  printf 'md1yqpqqxzq2qwfv8urt848e\n' > set.md1; cp keys keys.orig; cp F F.orig; cp set.md1 set.orig
}
fsize8() { # stdout/stderr through pipes; the limit binds mk's own file writes only
  bash -c "trap '' XFSZ; exec prlimit --fsize=8 \"\$@\"" _ "$@" > >(cat > o.txt) 2> >(cat > e.txt)
  local r=$?; sleep 0.5; return $r
}
fresh; "$MK" encode --in keys --policy-id-stub deadbeef > card.ref 2>/dev/null

PASS=0; FAIL=0
# check ROW RC WANT_RC STDERR_SPEC then "test-expr|label" facts
#   STDERR_SPEC: "=TEXT" stderr is exactly TEXT + newline; "~TEXT" contains TEXT; "-" no check
check() {
  local row=$1 rc=$2 want=$3 spec=$4; shift 4
  local bad=""
  [ "$rc" = "$want" ] || bad+=" rc=$rc want=$want;"
  local err; err=$(cat e.txt)
  case "$spec" in
    -) ;;
    =*) [ "$err" = "${spec#=}" ] || bad+=" stderr=[$err];" ;;
    ~*) [[ "$err" == *"${spec#\~}"* ]] || bad+=" stderr=[$err];" ;;
  esac
  local f
  for f in "$@"; do eval "${f%%|*}" || bad+=" ${f#*|};"; done
  if [ -z "$bad" ]; then PASS=$((PASS+1)); echo "PASS $row"; else FAIL=$((FAIL+1)); echo "FAIL $row:$bad"; fi
}
same() { cmp -s "$1" "$2"; }
notmp() { [ -z "$(find . -maxdepth 2 -name '*.tmp' -print -quit)" ]; }
R="so writing the card would replace the input mk is reading. Write to a new name."
S="so the card would go into the file mk reads its input from. A > redirect empties that file before mk starts, so check that it still holds what you expect. Write to a new name."
ENC=(encode --in keys --policy-id-stub deadbeef)

# --- I1: a failed --out write (R1, R3)
fresh; fsize8 "$MK" "${ENC[@]}" --out F; rc=$?
if [ "$MODE" = red ]; then check o1 $rc 64 "=error: --out F: File too large (os error 27)" "! same F F.orig|F destroyed"
else check o1 $rc 1 "=mk: cannot write the output: F: File too large (os error 27). F is unchanged." "same F F.orig|F unchanged" "notmp|no temp left"; fi
fresh; rm -f F; fsize8 "$MK" "${ENC[@]}" --out F; rc=$?
if [ "$MODE" = red ]; then check o2 $rc 64 "=error: --out F: File too large (os error 27)" "[ -e F ]|partial F left"
else check o2 $rc 1 "=mk: cannot write the output: F: File too large (os error 27). Nothing was written." "[ ! -e F ]|F absent" "notmp|no temp left"; fi
fresh; "$MK" "${ENC[@]}" --out nodir/F >o.txt 2>e.txt; rc=$?
if [ "$MODE" = red ]; then check o3 $rc 64 "=error: --out nodir/F: No such file or directory (os error 2)"
else check o3 $rc 1 "=mk: cannot write the output: nodir/F: No such file or directory (os error 2). Nothing was written." "[ ! -s o.txt ]|stdout empty"; fi
fresh; fsize8 "$MK" "${ENC[@]}" --json --out F; rc=$?
if [ "$MODE" = red ]; then check o4 $rc 64 "=" "grep -q '\"kind\":\"UsageError\"' o.txt|envelope on stdout" "! same F F.orig|F destroyed"
else check o4 $rc 1 "=mk: cannot write the output: F: File too large (os error 27). F is unchanged." "[ ! -s o.txt ]|stdout empty" "same F F.orig|F unchanged"; fi
fresh; printf 'PRIOR\n' > ro/F; chmod 644 ro/F; cp ro/F ro.orig; chmod 555 ro; "$MK" "${ENC[@]}" --out ro/F >o.txt 2>e.txt; rc=$?; chmod 755 ro
if [ "$MODE" = red ]; then check o12 $rc 0 "~note: stdout is watch-only" "same ro/F card.ref|written in place"
else check o12 $rc 1 "=mk: cannot write the output: ro/F: Permission denied (os error 13). ro/F is unchanged." "same ro/F ro.orig|ro/F unchanged"; fi
fresh; printf 'OLD BACKUP\n' > s.txt; bash -c "trap '' XFSZ; exec prlimit --fsize=8 \"\$@\"" _ "$MK" "${ENC[@]}" --out /dev/stdout >>s.txt 2> >(cat > e.txt); rc=$?; sleep 0.5
if [ "$MODE" = red ]; then check o20 $rc 64 "=error: --out /dev/stdout: File too large (os error 27)"
else check o20 $rc 1 "=mk: cannot write the output: /dev/stdout: File too large (os error 27)"; fi

# --- I6: --out names an input (R2)
fresh; "$MK" "${ENC[@]}" --out keys >o.txt 2>e.txt; rc=$?
if [ "$MODE" = red ]; then check o5 $rc 0 "-" "same keys card.ref|keys replaced by the card"
else check o5 $rc 64 "=error: --out keys is the same file as --in keys, $R" "same keys keys.orig|keys unchanged" "[ ! -s o.txt ]|stdout empty"; fi
fresh; ln keys hl; "$MK" "${ENC[@]}" --out hl >o.txt 2>e.txt; rc=$?
if [ "$MODE" = red ]; then check o6 $rc 0 "-" "same keys card.ref|keys replaced"
else check o6 $rc 64 "=error: --out hl is the same file as --in keys, $R" "same keys keys.orig|keys unchanged"; fi
fresh; ln -s keys sl; "$MK" "${ENC[@]}" --out sl >o.txt 2>e.txt; rc=$?
if [ "$MODE" = red ]; then check o7 $rc 0 "-" "same keys card.ref|keys replaced"
else check o7 $rc 64 "=error: --out sl is the same file as --in keys, $R" "same keys keys.orig|keys unchanged"; fi
fresh; "$MK" encode --keys keys --policy-id-stub deadbeef --out ./keys >o.txt 2>e.txt; rc=$?
if [ "$MODE" = red ]; then check o8 $rc 0 "-" "same keys card.ref|keys replaced"
else check o8 $rc 64 "=error: --out ./keys is the same file as --keys keys, $R" "same keys keys.orig|keys unchanged"; fi
fresh; "$MK" encode --in keys --from-md1-set set.md1 --out set.md1 >o.txt 2>e.txt; rc=$?
if [ "$MODE" = red ]; then check o9 $rc 0 "-" "! same set.md1 set.orig|set.md1 replaced"
else check o9 $rc 64 "=error: --out set.md1 is the same file as --from-md1-set set.md1, $R" "same set.md1 set.orig|set.md1 unchanged"; fi
fresh; "$MK" encode --in - --policy-id-stub deadbeef --out keys <keys >o.txt 2>e.txt; rc=$?
if [ "$MODE" = red ]; then check o10 $rc 0 "-" "same keys card.ref|keys replaced"
else check o10 $rc 64 "=error: --out keys is the same file as standard input (--in -), $R" "same keys keys.orig|keys unchanged"; fi
fresh; "$MK" "${ENC[@]}" --json --out keys >o.txt 2>e.txt; rc=$?
if [ "$MODE" = red ]; then check o21 $rc 0 "-" "! same keys keys.orig|keys replaced by JSON"
else check o21 $rc 64 "=" "grep -qF '\"message\":\"--out keys is the same file as --in keys, $R\"' o.txt|envelope on stdout" "same keys keys.orig|keys unchanged"; fi

# --- I6 widened to stdout (R4): encode without --out
fresh; "$MK" "${ENC[@]}" >>keys 2>e.txt; rc=$?
if [ "$MODE" = red ]; then check o15 $rc 0 "-" "[ \$(wc -l <keys) = 3 ]|card appended to keys"
else check o15 $rc 64 "=error: standard output is the same file as --in keys, $S" "same keys keys.orig|keys unchanged"; fi
fresh; "$MK" "${ENC[@]}" --json >>keys 2>e.txt; rc=$?
if [ "$MODE" = red ]; then check o16 $rc 0 "-" "! same keys keys.orig|JSON appended to keys"
else check o16 $rc 64 "=error: standard output is the same file as --in keys, $S" "same keys keys.orig|keys unchanged (no envelope written into it)"; fi
fresh; "$MK" "${ENC[@]}" >keys 2>e.txt; rc=$?
if [ "$MODE" = red ]; then check o17 $rc 64 "=error: --in keys: no key records found (blank lines and \`#\` comments are ignored)"
else check o17 $rc 64 "=error: standard output is the same file as --in keys, $S"; fi
fresh; "$MK" encode --in - --policy-id-stub deadbeef <keys >>keys 2>e.txt; rc=$?
if [ "$MODE" = red ]; then check o18 $rc 0 "-" "[ \$(wc -l <keys) = 3 ]|card appended to keys"
else check o18 $rc 64 "=error: standard output is the same file as standard input (--in -), $S" "same keys keys.orig|keys unchanged"; fi

# --- controls (both modes)
fresh; "$MK" "${ENC[@]}" --out G >o.txt 2>e.txt; rc=$?
check o11 $rc 0 "=note: stdout is watch-only — public keys only, cannot spend" "same G card.ref|G is the stdout card" "[ \$(stat -c %a G) = 600 ]|G is 0600" "[ ! -s o.txt ]|stdout empty"
fresh; ln -s F lk; "$MK" "${ENC[@]}" --out lk >o.txt 2>e.txt; rc=$?
check o13 $rc 0 "-" "same F card.ref|link target replaced" "[ -L lk ]|lk still a symlink"
fresh; chmod 644 F; "$MK" "${ENC[@]}" --out F >o.txt 2>e.txt; rc=$?
check o14 $rc 0 "-" "same F card.ref|F overwritten" "[ \$(stat -c %a F) = 600 ]|F tightened to 0600" "notmp|no temp left"
fresh; "$MK" "${ENC[@]}" 2>e.txt | cat >o.txt; rc=${PIPESTATUS[0]}
check o19 $rc 0 "-" "same o.txt card.ref|card on the pipe" "same keys keys.orig|keys untouched"

echo "rows: $PASS passed, $FAIL failed"
[ "$FAIL" = 0 ]
