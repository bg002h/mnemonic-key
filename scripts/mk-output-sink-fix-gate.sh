#!/usr/bin/env bash
# Gate for design/FIX_mk_output_sink.md (mk output sink, crash sweep F1).
#
# What it does:
#   1. Extracts the document's blocks (section 3 plain `diff` blocks, section 3.7
#      `diff followups`, section 4 `diff` rows and the `sh` RED commands) and checks
#      that the code blocks and the rows hash to the sha256 the document states.
#      If the architect's files exist (GRADE, default /scratch/cache/architect-output-sink-ro/mk),
#      it also cmp's the code with fix.diff and the rows with rows-combined-i3.diff.
#   2. In its own detached worktree at the base commit: applies the rows, runs every
#      RED command as printed and requires each to FAIL with its `## expect:`
#      substring; the source-scan row must PASS on the base (it is a guard).
#   3. Applies the code blocks and FOLLOWUPS block; requires the seven rows green, the
#      full workspace suite "406 tests run: 406 passed", doc tests, fmt, clippy.
#   4. Self-test: the checker must reject a wrong expectation and a row that passes,
#      or the gate dies.
#
# What it does NOT cover: the plants P1 to P12 (not re-run); the Windows, macOS,
# musl, release-mode and FreeBSD CI legs; the toolkit manual sentence; the
# coordinator's mutation-gate.sh; any review. fmt/clippy run on the repo's pinned
# toolchain (the grade ran fmt on 1.95.0).
#
# Usage: scripts/mk-output-sink-fix-gate.sh [DOC]   (run from the repo root)
set -euo pipefail
export PATH="$HOME/.cargo/bin:$PATH"
DOC=${1:-design/FIX_mk_output_sink.md}
REPO=$(git rev-parse --show-toplevel)
DOC=$(realpath "$DOC")
BASE=${BASE:-2fc367c560013667180ee19572bbfad544d8cd88}
W=${W:-/scratch/cache/mk-output-sink-gate-wt}
OUT=${OUT:-/scratch/cache/mk-output-sink-gate-out}
GRADE=${GRADE:-/scratch/cache/architect-output-sink-ro/mk}
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-/scratch/cache/mk-output-sink-gate-target}
case "$W$OUT$CARGO_TARGET_DIR" in /tmp*) echo "refusing /tmp" >&2; exit 2;; esac
mkdir -p "$OUT"
die() { echo "GATE FAIL: $*" >&2; exit 1; }
cargo --version | grep -q '^cargo 1\.85\.' || die "cargo is not the 1.85 pin: $(cargo --version)"

# 1. extract
python3 -I - "$DOC" "$OUT" <<'PY'
import sys,re,hashlib
doc,out=sys.argv[1:3]
t=open(doc).read()
def section(a,b):
    i=t.index(a); j=t.index(b,i); return t[i:j]
def blocks(s,tag):
    return re.findall(r'(?m)^```'+tag+r'\n(.*?)^```\n',s,re.S)
s3=section('\n## 3. ','\n## 4. ')
s4=section('\n## 4. ','\n## 5. ')
code=''.join(blocks(s3,'diff'))
fu=''.join(blocks(s3,'diff followups'))
rows=''.join(blocks(s4,'diff'))
red=''.join(blocks(s4,'sh'))
want=re.search(r'sha256 `([0-9a-f]{64})`, checked',s3).group(1)
wantr=re.search(r'sha256 `([0-9a-f]{64})`\)',s4).group(1)
assert hashlib.sha256(code.encode()).hexdigest()==want,'code blocks differ from the stated sha256'
assert hashlib.sha256(rows.encode()).hexdigest()==wantr,'rows block differs from the stated sha256'
assert len(blocks(s3,'diff'))==6 and len(blocks(s3,'diff followups'))==1
assert len([l for l in red.splitlines() if l.startswith('cargo nextest')])==6
for n,v in (('code.diff',code),('rows.diff',rows),('followups.diff',fu),('red.sh',red)):
    open(out+'/'+n,'w').write(v)
PY
if [ -f "$GRADE/fix.diff" ]; then
  cmp "$OUT/code.diff" "$GRADE/fix.diff" || die "code blocks differ from the architect's fix.diff"
  cmp "$OUT/rows.diff" "$GRADE/rows-combined-i3.diff" || die "rows differ from the architect's rows-combined-i3.diff"
  echo "cmp: blocks identical to the architect's files"
else echo "cmp: architect files absent, sha256 only"; fi

# worktree at the base
[ -e "$W/.git" ] || git -C "$REPO" worktree add --detach "$W" "$BASE" >/dev/null
reset() { git -C "$W" reset -q --hard "$BASE"; rm -f "$W/crates/mk-cli/tests/output_errors.rs" "$W/crates/mk-cli/src/out.rs"; }
reset
git -C "$W" rev-parse HEAD | grep -q "^$BASE" || die "worktree is not at the base"

# checker: 0 only if the run FAILED and its output holds the substring
red_check() { # $1 log, $2 rc, $3 expect
  [ "$2" -ne 0 ] && grep -qF -- "$3" "$1"
}
# 2. RED on the base
git -C "$W" apply "$OUT/rows.diff"
n=0; first_log=
while IFS= read -r line; do
  case "$line" in cargo\ nextest*) ;; *) continue;; esac
  cmd=${line%% ## expect: *}; exp=${line#* ## expect: }
  n=$((n+1)); log="$OUT/red-$n.log"
  rc=0; (cd "$W" && eval "$cmd") >"$log" 2>&1 || rc=$?
  red_check "$log" "$rc" "$exp" || die "RED row $n did not fail for its stated reason: $cmd (rc=$rc, expect: $exp)"
  echo "RED $n ok (rc=$rc): $cmd"
  [ -n "$first_log" ] || { first_log=$log; first_rc=$rc; }
done < "$OUT/red.sh"
[ "$n" -eq 6 ] || die "expected 6 RED commands, ran $n"
# whole-file RED and the scan guard
rc=0; (cd "$W" && cargo nextest run --locked -j 8 -E 'binary(output_errors)') >"$OUT/red-all.log" 2>&1 || rc=$?
grep -q '7 tests run: 1 passed, 6 failed' "$OUT/red-all.log" || die "whole-file RED is not 1 passed, 6 failed"
grep -q 'PASS .*only_out_rs_names_stdout' "$OUT/red-all.log" || die "the scan row did not pass on the base"
echo "RED whole file: $(grep -o '7 tests run: 1 passed, 6 failed' "$OUT/red-all.log")"
# self-test of the checker
red_check "$first_log" "$first_rc" "NO-SUCH-TEXT-XYZ" && die "self-test: a wrong expectation was accepted"
red_check "$OUT/red-all.log" 0 "7 tests run" && die "self-test: a passing run was accepted as RED"
echo "self-test ok: the checker rejects a broken row"

# 3. GREEN
git -C "$W" apply "$OUT/code.diff"
git -C "$W" apply "$OUT/followups.diff"
grep -q 'mk-output-sink' "$W/design/FOLLOWUPS.md" || die "FOLLOWUPS entry missing"
run() { # name cmd...
  local name=$1; shift
  (cd "$W" && "$@") >"$OUT/$name.log" 2>&1 || { tail -n 30 "$OUT/$name.log" >&2; die "$name failed"; }
}
run rows cargo nextest run --locked -j 8 -E 'binary(output_errors)'
grep -q '7 tests run: 7 passed' "$OUT/rows.log" || die "rows are not 7/7"
run suite cargo nextest run --locked -j 8
grep -q '406 tests run: 406 passed' "$OUT/suite.log" || die "suite is not 406/406: $(grep 'tests run' "$OUT/suite.log")"
run doc cargo test --locked --doc
run fmt cargo fmt --check
run clippy cargo clippy --locked --workspace --all-targets -- -D warnings
echo "GREEN rows: $(grep -o '7 tests run: 7 passed' "$OUT/rows.log")"
echo "GREEN suite: $(grep -o '406 tests run: 406 passed' "$OUT/suite.log")"
echo "GREEN doc, fmt, clippy: ok"
reset
echo "GATE PASS"
