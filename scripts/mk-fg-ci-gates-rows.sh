#!/usr/bin/env bash
# scripts/mk-fg-ci-gates-rows.sh red|green -- RED/GREEN rows for
# design/FIX_mk_fg_ci_gates.md (bughunt-1119 F-G: bh-mk-I11, I12, I13).
#
# What it runs, against the files in this checkout:
#   I11  For the ci.yml jobs build (lane stable), fmt (1.95.0) and
#        vectors-roundtrip (stable): emulates the lane the way
#        dtolnay/rust-toolchain leaves it (`rustup default <label>`, in a
#        scratch RUSTUP_HOME whose toolchains/ links to yours), applies the
#        job's own "Use the lane's toolchain" step to a scratch GITHUB_ENV,
#        then runs the job's own "Check the lane runs its labelled compiler"
#        step, and asserts unqualified `rustc --version` equals
#        `rustc +<label> --version`, and runs the check step again without
#        the pin step, which must fail. A static row checks the build job's
#        beta label and continue-on-error, and that CHECK_NAME equals the name.
#   I12  Extracts the release-creating command of ci.yml, musl-binaries.yml
#        and release.yml and runs it under `bash -e` with a mocked `gh` in
#        four cases: exists, created, raced (create fails, a later view finds
#        it), still-missing (must exit nonzero).
#   I13  Extracts the first sh fence after "### 9.3 " in
#        docs/verify-reproducibility.md, substitutes <tagged-commit-SHA> ->
#        HEAD and <VER> -> rows, and runs it in a scratch git repo with
#        `cross` mocked: build ok (member mtime = commit time, cross saw the
#        exported epoch), a stale SOURCE_DATE_EPOCH in the caller, cross
#        fails (nonzero, no archive), cross ok but no binary (nonzero), and
#        the caller's shell options after a failure (no set -e leak).
#
# What it does NOT cover: hosted runners (the real dtolnay action, rustup on
# GitHub images, Windows/macOS shells), the '1.85' and beta lanes (labels not
# installed here; the script never installs a toolchain), GitHub's
# continue-on-error semantics, `gh` against GitHub, real cross/docker/QEMU
# builds and archive hashes, and the x86_64 recipe in section 4.
#
# Needs: bash, git, awk, sed, tar, gzip, GNU date, rustup with the stable and
# 1.95.0 toolchains installed. Scratch files under $WORK (default
# <repo>/target/mk-fg-rows). Exit 0 and "ALL ROWS MATCH <mode>" when every
# row shows the mode's expectation.
set -u
mode=${1:-}
case "$mode" in red|green) ;; *) echo "usage: $0 red|green" >&2; exit 2 ;; esac
repo=$(git rev-parse --show-toplevel) || exit 2
WORK=${WORK:-$repo/target/mk-fg-rows}
mkdir -p "$WORK"
ci=$repo/.github/workflows/ci.yml
musl=$repo/.github/workflows/musl-binaries.yml
rel=$repo/.github/workflows/release.yml
doc=$repo/docs/verify-reproducibility.md
mismatch=0

# row <name> <pass 0|1>: in green mode every row must pass; in red mode every
# row must fail (each row is a defect row; controls are folded into rows).
row() {
  local name=$1 pass=$2 detail=$3
  local want; [ "$mode" = green ] && want=1 || want=0
  if [ "$pass" = "$want" ]; then
    echo "MATCH $mode  $name: $detail"
  else
    echo "MISMATCH $mode  $name: $detail"; mismatch=1
  fi
}

# ---------- I11 ----------
job_block() { # <job id>: the lines of that job in ci.yml
  awk -v j="  $1:" '$0==j{f=1;print;next} f&&/^  [a-z]/{exit} f{print}' "$ci"
}
step_run() { # <job block on stdin> <step name>: the step's run text, dedented
  awk -v n="- name: $1" '
    index($0,n){f=1;next}
    f&&/^      - /{exit}
    f&&/^        run: \|/{r=1;next}
    f&&/^        run: /{sub(/^        run: /,"");print;exit}
    r&&/^          /{sub(/^          /,"");print;next}
    r{exit}'
}
real_rh=${RUSTUP_HOME:-$HOME/.rustup}
i11_lane() { # <job id> <label>
  local job=$1 label=$2 rh=$WORK/rh-$1 envf=$WORK/github-env-$1
  mkdir -p "$rh"; ln -sfn "$real_rh/toolchains" "$rh/toolchains"
  [ -e "$rh/update-hashes" ] || ln -sfn "$real_rh/update-hashes" "$rh/update-hashes"
  printf 'version = "12"\ndefault_toolchain = "%s"\nprofile = "minimal"\n' "$label" >"$rh/settings.toml"
  : >"$envf"
  local blk pin chk got want chk_rc=skip
  blk=$(job_block "$job")
  pin=$(printf '%s\n' "$blk" | step_run "Use the lane's toolchain, not rust-toolchain.toml")
  chk=$(printf '%s\n' "$blk" | step_run "Check the lane runs its labelled compiler")
  # the pin step must take the label from the action's own output
  local wired=0
  printf '%s\n' "$blk" | grep -q 'id: toolchain' &&
    [ "$(printf '%s\n' "$blk" | grep -c 'LANE_TOOLCHAIN: \${{ steps.toolchain.outputs.name }}')" -ge 2 ] && wired=1
  (
    cd "$repo" || exit 9
    export RUSTUP_HOME=$rh; unset RUSTUP_TOOLCHAIN
    [ -n "$pin" ] && LANE_TOOLCHAIN=$label GITHUB_ENV=$envf bash -e -c "$pin"
    set -a; . "$envf"; set +a
    if [ -n "$chk" ]; then LANE_TOOLCHAIN=$label bash -e -c "$chk" >"$WORK/chk-$job.out" 2>&1; echo "chk_rc=$?" >"$WORK/chk-$job.rc"
    else echo "chk_rc=absent" >"$WORK/chk-$job.rc"; fi
    echo "$(rustc --version)|$(rustc +"$label" --version)"
  ) >"$WORK/i11-$job.out" 2>&1
  # the check must FAIL when the pin step is absent (else it is a gate that cannot fail)
  local nopin=absent
  if [ -n "$chk" ]; then
    ( cd "$repo" || exit 9; export RUSTUP_HOME=$rh; unset RUSTUP_TOOLCHAIN
      LANE_TOOLCHAIN=$label bash -e -c "$chk" ) >"$WORK/chk-nopin-$job.out" 2>&1 && nopin=0 || nopin=$?
  fi
  got=$(tail -1 "$WORK/i11-$job.out" | cut -d'|' -f1); want=$(tail -1 "$WORK/i11-$job.out" | cut -d'|' -f2)
  chk_rc=$(sed 's/chk_rc=//' "$WORK/chk-$job.rc" 2>/dev/null)
  local pass=0
  [ "$got" = "$want" ] && [ "$wired" = 1 ] && [ "$chk_rc" = 0 ] && [ "$nopin" != absent ] && [ "$nopin" != 0 ] && pass=1
  row "i11_${job}_lane_runs_label" "$pass" "label=$label unqualified: $got | labelled: $want | wired=$wired check-step=$chk_rc check-without-pin=$nopin"
}
i11_lane build stable
i11_lane fmt 1.95.0
i11_lane vectors-roundtrip stable

# T45: beta labelled non-blocking, stable not; CHECK_NAME follows the name.
blk=$(job_block build)
name=$(printf '%s\n' "$blk" | sed -n 's/^    name: //p')
coe=$(printf '%s\n' "$blk" | sed -n 's/^    continue-on-error: //p')
chkname=$(printf '%s\n' "$blk" | sed -n 's/^          CHECK_NAME: //p')
want_name="build (\${{ matrix.toolchain }} on \${{ matrix.os }}\${{ matrix.toolchain == 'beta' && ', non-blocking' || '' }})"
want_coe="\${{ matrix.toolchain == 'beta' }}"
p=0; [ "$name" = "$want_name" ] && [ "$coe" = "$want_coe" ] && [ "$chkname" = "$want_name" ] && p=1
row i11_beta_lane_labelled_non_blocking "$p" "name=[$name] continue-on-error=[$coe] CHECK_NAME=[$chkname]"

# ---------- I12 ----------
ci_block=$(awk '/- name: Create GitHub release if missing/{f=1} f&&/run: \|/{g=1;next} g&&/^$/{exit} g{sub(/^          /,"");print}' "$ci")
musl_line=$(grep -v "^ *#" "$musl" | grep -m1 'gh release create' | sed 's/^ *run: //; s/^ *//')
rel_line=$(grep -v "^ *#" "$rel" | grep -m1 'gh release create' | sed 's/^ *//')
i12_case() { # <code> <view1> <create> <view2>: prints rc and create count
  local code=$1 cnt=$WORK/gh-count created=$WORK/gh-created
  echo 0 >"$cnt"; : >"$created"
  CNT=$cnt CREATED=$created VIEW1=$2 CREATE=$3 VIEW2=$4 \
  GH_REF=refs/tags/mk-cli-v0.0.0-rows TAG=mk-cli-v0.0.0-rows REF_NAME=mk-cli-v0.0.0-rows \
  bash -e -c '
    gh() { case "$1 $2" in
      "release view") n=$(cat "$CNT"); echo $((n+1)) >"$CNT"
                      if [ "$n" = 0 ]; then return "$VIEW1"; else return "$VIEW2"; fi ;;
      "release create") echo c >>"$CREATED"; return "$CREATE" ;;
      *) return 99 ;; esac; }
    '"$code" >/dev/null 2>&1
  echo "$? $(wc -l <"$created")"
}
i12_site() { # <row name> <code>
  local code=$2 e c r s
  if [ -z "$code" ]; then row "$1" 0 "no release-creating command found"; return; fi
  read -r e ec < <(i12_case "$code" 0 17 17)
  read -r c _ < <(i12_case "$code" 1 0 1)
  read -r r _ < <(i12_case "$code" 1 17 0)
  read -r s _ < <(i12_case "$code" 1 17 1)
  local p=0
  [ "$e" = 0 ] && [ "$ec" = 0 ] && [ "$c" = 0 ] && [ "$r" = 0 ] && [ "$s" != 0 ] && p=1
  row "$1" "$p" "exists rc=$e (creates=$ec) created rc=$c raced rc=$r still-missing rc=$s"
}
i12_site i12_ci_release_on_tag_fails_without_release "$ci_block"
i12_site i12_musl_ensure_release_fails_without_release "$musl_line"
i12_site i12_release_yml_ensure_release_fails_without_release "$rel_line"

# ---------- I13 ----------
block=$(awk '/^### 9.3 /{h=1} h&&/^```sh/{f=1;next} f&&/^```/{exit} f{print}' "$doc" |
  sed 's/<tagged-commit-SHA>/HEAD/g; s/<VER>/rows/g')
arch=mk-rows-aarch64-linux-musl.tar.gz
mk_repo() { # fresh scratch repo whose HEAD commit time is 1577836800
  local d; d=$(mktemp -d "$WORK/i13.XXXXXX"); cd "$d" || exit 9
  git init -q .
  GIT_COMMITTER_DATE=2020-01-01T00:00:00Z git -c user.email=rows@example.invalid -c user.name=rows \
    commit -q --allow-empty -m x --date=2020-01-01T00:00:00Z
}
# the cross stand-in: records the env it got, then $1 = rc, $2 = make the binary (1/0)
cross_ok='cross() { printf "%s|%s\n" "${SOURCE_DATE_EPOCH-unset}" "${CARGO_BUILD_RUSTFLAGS-unset}" >cross.env
  mkdir -p target/aarch64-unknown-linux-musl/release
  printf "synthetic artifact" >target/aarch64-unknown-linux-musl/release/mk
  chmod 755 target/aarch64-unknown-linux-musl/release/mk; return 0; }'
cross_fail='cross() { return 23; }'
cross_nobin='cross() { return 0; }'
want_epoch=1577836800
want_flags="--remap-path-prefix=/project=/build --remap-path-prefix=/cargo=/cargo"
member_epoch() {
  local t; t=$(TZ=UTC tar -tvzf "$arch" --full-time 2>/dev/null | awk '{print $4" "$5}')
  date -u -d "$t" +%s 2>/dev/null || echo none
}
(
  mk_repo
  env -u SOURCE_DATE_EPOCH bash -c "$cross_ok"$'\n'"$block" >out.txt 2>&1; rc=$?
  got=$(member_epoch); seen=$(cat cross.env 2>/dev/null)
  p=0; [ "$rc" = 0 ] && [ "$got" = "$want_epoch" ] && [ "$seen" = "$want_epoch|$want_flags" ] && p=1
  echo "$p|rc=$rc member_mtime=$got want=$want_epoch cross_saw=[$seen] stderr1=[$(head -1 out.txt)]"
) >"$WORK/i13a"
row i13_recipe_build_ok_epoch_from_commit "$(cut -d'|' -f1 "$WORK/i13a")" "$(cut -d'|' -f2- "$WORK/i13a")"
(
  mk_repo
  SOURCE_DATE_EPOCH=1 bash -c "$cross_ok"$'\n'"$block" >out.txt 2>&1; rc=$?
  got=$(member_epoch)
  p=0; [ "$rc" = 0 ] && [ "$got" = "$want_epoch" ] && p=1
  echo "$p|caller SOURCE_DATE_EPOCH=1: rc=$rc member_mtime=$got want=$want_epoch"
) >"$WORK/i13b"
row i13_recipe_ignores_stale_caller_epoch "$(cut -d'|' -f1 "$WORK/i13b")" "$(cut -d'|' -f2- "$WORK/i13b")"
(
  mk_repo
  env -u SOURCE_DATE_EPOCH bash -c "$cross_fail"$'\n'"$block" >out.txt 2>&1; rc=$?
  ex=no; [ -e "$arch" ] && ex=yes
  p=0; [ "$rc" != 0 ] && [ "$ex" = no ] && p=1
  echo "$p|cross rc 23: rc=$rc archive_exists=$ex"
) >"$WORK/i13c"
row i13_recipe_build_fails_nonzero_no_archive "$(cut -d'|' -f1 "$WORK/i13c")" "$(cut -d'|' -f2- "$WORK/i13c")"
(
  mk_repo
  env -u SOURCE_DATE_EPOCH bash -c "$cross_nobin"$'\n'"$block" >out.txt 2>&1; rc=$?
  p=0; [ "$rc" != 0 ] && p=1
  echo "$p|cross rc 0, no binary: rc=$rc"
) >"$WORK/i13d"
row i13_recipe_missing_binary_nonzero "$(cut -d'|' -f1 "$WORK/i13d")" "$(cut -d'|' -f2- "$WORK/i13d")"
(
  mk_repo
  out=$(env -u SOURCE_DATE_EPOCH bash -c "$cross_fail"$'\n'"$block"$'\n''echo "after rc=$? opts=$-"' 2>/dev/null | tail -1)
  rc=${out#after rc=}; rc=${rc%% *}; opts=${out##* opts=}
  p=0; case "$out" in "after rc="*) [ "$rc" != 0 ] && case "$opts" in *e*) ;; *) p=1 ;; esac ;; esac
  echo "$p|caller shell after a failed build: [$out]"
) >"$WORK/i13e"
row i13_recipe_reports_failure_to_caller_without_set_e "$(cut -d'|' -f1 "$WORK/i13e")" "$(cut -d'|' -f2- "$WORK/i13e")"

if [ "$mismatch" = 0 ]; then echo "ALL ROWS MATCH $mode"; exit 0; fi
echo "ROWS MISMATCH $mode"; exit 1
