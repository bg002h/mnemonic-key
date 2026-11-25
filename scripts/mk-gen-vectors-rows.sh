#!/usr/bin/env bash
# RED/GREEN rows for design/FIX_mk_gen_vectors_overwrite.md (bughunt-1119 bh-mk-I16:
# gen_mk_vectors writes the vectors corpus, then writes the csid_ext corpus over the
# same file, and prints "wrote N vectors" for a file that no longer holds them).
#
# What it does: builds the gen_mk_vectors binary (feature gen-vectors, --locked,
# CARGO_BUILD_JOBS=6 unless set) and runs rows R1-R4 plus control C1 against it,
# each in a fresh directory under target/. Checks each row against MODE:
#   red   = the base commit's behaviour (each defect shows; C1 passes)
#   green = the fixed behaviour (exact exit code and exact stderr per the FIX doc)
# In green mode it then runs the nextest rows in
# crates/mk-codec/tests/gen_vectors_output.rs (--features gen-vectors).
# Any row that does not match its expectation fails the script (exit 1).
#
# What it does NOT cover: fmt, clippy, the full workspace suite, doc-tests,
# non-Linux legs (R4 uses a symlink; case-insensitive filesystems are not run),
# the generator's other panics (bad arguments, I/O errors), and CI (the default
# `cargo nextest run --workspace` does not enable gen-vectors, so CI never runs
# the nextest rows; see the FIX doc section 7).
#
# Usage: scripts/mk-gen-vectors-rows.sh red|green   (run from the repo root)
set -euo pipefail
export PATH="$HOME/.cargo/bin:$PATH"
export CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS:-6}
MODE=${1:?usage: $0 red|green}
case "$MODE" in red|green) ;; *) echo "usage: $0 red|green" >&2; exit 2;; esac
REPO=$(git rev-parse --show-toplevel)
case "$REPO" in /tmp*) echo "refusing /tmp" >&2; exit 2;; esac
cd "$REPO"
cargo build --locked -q -p mk-codec --features gen-vectors --bin gen_mk_vectors
BIN=$REPO/target/debug/gen_mk_vectors
CORPUS=$REPO/crates/mk-codec/src/test_vectors
N=$(python3 -c 'import json,sys; print(len(json.load(open(sys.argv[1]))["vectors"]))' "$CORPUS/v0.1.json")
ROWS=$(python3 -c 'import json,sys; print(len(json.load(open(sys.argv[1]))["rows"]))' "$CORPUS/csid_ext_v0.1.json")
mkdir -p "$REPO/target"
W=$(mktemp -d "$REPO/target/mk-gv-rows.XXXXXX")
FAIL=0
ok()  { echo "OK   $MODE $1"; }
bad() { echo "FAIL $MODE $1: $2"; FAIL=1; }
# run DIR ARGS...: runs the binary in DIR; sets RC, OUT (stdout), ERR (stderr).
run() { local d=$1; shift; set +e; (cd "$d" && "$BIN" "$@" >"$W/out" 2>"$W/err"); RC=$?; set -e
  OUT=$(cat "$W/out"); ERR=$(cat "$W/err"); }
holds_vectors() { grep -q '"vectors"' "$1" 2>/dev/null; }
t1() { echo "gen_mk_vectors: refused: --output $1 is named csid_ext_v0.1.json, the csid_ext file this tool writes beside it; nothing was written"; }
t2() { echo "gen_mk_vectors: error: $1 no longer holds the $N vectors after $2 was written; it is the same file as $2 or was changed meanwhile"; }
check() { # name rc_want err_want
  if [ "$RC" != "$2" ]; then bad "$1" "exit $RC, want $2"; return; fi
  if [ -n "$OUT" ]; then bad "$1" "stdout not empty"; return; fi
  if [ "$ERR" != "$3" ]; then bad "$1" "stderr was: $ERR"; return; fi
  ok "$1"; }

# R1: --output <dir>/csid_ext_v0.1.json over an existing file.
d=$W/r1; mkdir "$d"; P=$d/csid_ext_v0.1.json; printf 'sentinel\n' > "$P"
run "$d" --output "$P"
if [ "$MODE" = red ]; then
  if [ "$RC" = 0 ] && grep -qF "wrote $N vectors to $P" <<<"$ERR" && ! holds_vectors "$P"; then
    ok "R1_output_named_csid_ext (defect: exit 0, claims $N vectors, file holds none)"
  else bad R1_output_named_csid_ext "defect not shown (exit $RC)"; fi
else
  check R1_output_named_csid_ext 64 "$(t1 "$P")"
  [ "$(cat "$P")" = sentinel ] || bad R1_sentinel "existing file was changed"
  [ "$(ls "$d")" = csid_ext_v0.1.json ] || bad R1_nothing_written "directory gained a file"
fi

# R2: bare --output csid_ext_v0.1.json from an empty directory.
d=$W/r2; mkdir "$d"
run "$d" --output csid_ext_v0.1.json
if [ "$MODE" = red ]; then
  if [ "$RC" = 0 ] && grep -qF "wrote $N vectors to csid_ext_v0.1.json" <<<"$ERR" && ! holds_vectors "$d/csid_ext_v0.1.json"; then
    ok "R2_bare_csid_ext_name (defect: exit 0, claims $N vectors, file holds none)"
  else bad R2_bare_csid_ext_name "defect not shown (exit $RC)"; fi
else
  check R2_bare_csid_ext_name 64 "$(t1 csid_ext_v0.1.json)"
  [ -z "$(ls "$d")" ] || bad R2_nothing_written "directory is not empty"
fi

# R3: hard link v.json to an existing csid_ext_v0.1.json.
d=$W/r3; mkdir "$d"; C=$d/csid_ext_v0.1.json; P=$d/v.json; printf 'x\n' > "$C"; ln "$C" "$P"
run "$d" --output "$P"
if [ "$MODE" = red ]; then
  if [ "$RC" = 0 ] && grep -qF "wrote $N vectors to $P" <<<"$ERR" && ! holds_vectors "$P"; then
    ok "R3_hard_link_alias (defect: exit 0, claims $N vectors, file holds none)"
  else bad R3_hard_link_alias "defect not shown (exit $RC)"; fi
else
  check R3_hard_link_alias 1 "$(t2 "$P" "$C")"
fi

# R4: symlink v.json -> csid_ext_v0.1.json (same directory).
d=$W/r4; mkdir "$d"; C=$d/csid_ext_v0.1.json; P=$d/v.json; printf 'x\n' > "$C"; ln -s csid_ext_v0.1.json "$P"
run "$d" --output "$P"
if [ "$MODE" = red ]; then
  if [ "$RC" = 0 ] && grep -qF "wrote $N vectors to $P" <<<"$ERR" && ! holds_vectors "$P"; then
    ok "R4_symlink_alias (defect: exit 0, claims $N vectors, file holds none)"
  else bad R4_symlink_alias "defect not shown (exit $RC)"; fi
else
  check R4_symlink_alias 1 "$(t2 "$P" "$C")"
fi

# C1 (control, both modes): --output <dir>/v0.1.json writes both corpora, byte-equal
# to the committed files, and prints the two wrote lines in order.
d=$W/c1; mkdir "$d"; P=$d/v0.1.json; C=$d/csid_ext_v0.1.json
run "$d" --output "$P"
want="wrote $N vectors to $P ($(wc -c <"$CORPUS/v0.1.json") bytes)
wrote $ROWS csid_ext rows to $C ($(wc -c <"$CORPUS/csid_ext_v0.1.json") bytes)"
check C1_writes_both_corpora 0 "$want"
cmp -s "$P" "$CORPUS/v0.1.json" || bad C1_v01_bytes "differs from committed v0.1.json"
cmp -s "$C" "$CORPUS/csid_ext_v0.1.json" || bad C1_csid_bytes "differs from committed csid_ext_v0.1.json"

if [ "$MODE" = green ]; then
  if cargo nextest run --locked -p mk-codec --features gen-vectors --test gen_vectors_output \
      --build-jobs "$CARGO_BUILD_JOBS" -j 6 >"$W/nextest.log" 2>&1; then
    ok "nextest gen_vectors_output ($(grep -o '[0-9]* passed' "$W/nextest.log" | tail -1))"
  else bad nextest "see $W/nextest.log"; fi
fi

echo "rows dir: $W"
if [ "$FAIL" = 0 ]; then echo "ALL OK ($MODE)"; else echo "FAILURES ($MODE)"; exit 1; fi
