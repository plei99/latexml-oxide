#!/usr/bin/env bash
# Run the fork's Linux gates (upstream CI's test and lint jobs, on TeX Live 2026 scheme-full) on a
# committed ref, on the OrbStack machine `agents` (provisioned by fork/linux/setup.sh).
#
#   fork/linux/check.sh [ref] [step...]     steps: formats nextest lint (default: all three);
#                                             `prepare` only puts the gate worktree on the ref
#
# The ref is fetched into the guest clone (remote `mac` = the Mac checkout) and checked out, detached,
# in a dedicated persistent worktree, so only committed code is tested and the macOS checkout (its
# target/ and its dumps) is never touched. `git clean -fdq` (not -x) keeps the worktree's ignored
# target/ and resources/dumps between runs. CI's rust-cache keeps dependencies only, so the workspace
# members are cleaned (`cargo clean -p`) on every run: a new test fixture is globbed in at compile time
# with no rerun-if-changed, and a kept build would not see it. Dependencies stay cached. CI also
# installs a fresh nightly every run, so `rustup toolchain install nightly` runs on every run here.
# Dumps are kept, and resources/dumps/.gate-record is written after a successful `formats`: the commit,
# the sha256 of every dump file and the TeX Live identity (version line, tlpdb sha256, install time).
# `nextest` refuses (exit 121) when `formats` did not run earlier in the same run and the record
# differs from the current state, so a different ref, hand-regenerated dumps or a TL reinstall each
# invalidate it. `nextest` follows CI's test job: it builds once (`--no-run`), then runs the four
# partitions hash:1/4 to hash:4/4 one after another, each with --no-fail-fast, all in nextest.log; the
# step fails if any partition fails and the log names which. CI gives each partition its own 16 GB
# runner. The machine's memory is OrbStack's machine.agents.memory_mib (`orb config show`) and its
# cgroup allows no swap, so the partitions run one after another: with that setting at 8 GiB, two
# ~4.1 GB streaming-sweep shards in one 2-thread nextest pass were OOM-killed.
# Cargo.lock is gitignored, so one is copied in (the invoking checkout's, else the main checkout's; a
# fresh resolve fails while every zip 8.x is yanked) and both hosts build the same dependency versions.
# A different `rustc -vV` than the last run empties target/ci altogether, so it does not grow per nightly.
#
# Logs go to /tmp/linux-check/<sha>/<step>.log on the Mac; a run first removes that directory's old
# logs, so a missing log means the guest never started. Exit code: the number of failed steps; 100 for a
# usage or setup error (an unknown step, a machine that is stopped or not provisioned, a failed prepare or
# log removal, a busy gate). The guest script's own refusals use exit codes 121 to 125 (121 no dumps for
# this state, 122 worktree not at the ref, 123 gate worktree missing, 124 guest lock cannot be opened or
# taken, 125 log not writable), outside cargo's and nextest's codes; a refusal also puts a
# `lxgate-refused:` line in the log, and it is that line, with the code, that marks one.
# One run at a time: a second run is refused while the first holds the gate worktree.
set -uo pipefail
REPO=$(cd "$(dirname "$0")/../.." && pwd)
MACHINE=agents
GUEST_CLONE=/home/patrick/agent-work/repos/latexml
GATE=/home/patrick/agent-work/worktrees/latexml/linux-gate
GUEST_LOCK=/home/patrick/agent-work/linux-gate.lock
LOCK=/tmp/linux-check/gate.lock
USAGE=100

ref=${1:-HEAD}; shift || true
# An empty or blank step list is the default (bash 3.2 on the Mac treats an empty array as unbound).
arg="$*"; [ -n "${arg// /}" ] || arg="formats nextest lint"
steps=($arg)
for s in "${steps[@]}"; do
  case $s in prepare|formats|nextest|lint) ;; *) echo "unknown step $s"; exit $USAGE ;; esac
done
full=$(git -C "$REPO" rev-parse --verify "$ref^{commit}") || exit $USAGE
sha=$(git -C "$REPO" rev-parse --short "$full")
main=$(dirname "$(git -C "$REPO" rev-parse --path-format=absolute --git-common-dir)")
# The guest writes the logs itself, to this path on the Mac's filesystem (the guest sees the Mac's
# /private/tmp at the same path, and /tmp is a symlink to it, so the path is the physical one).
# Streaming a step's output through orb loses its tail when the command exits right after a burst of
# output (a 16,883,280-byte `orb -m agents cat file | wc -c` gave 16.1 to 16.3 MB; with a guest-side
# redirect it was complete), and a nextest log ends on its summary.
mkdir -p "/tmp/linux-check/$sha"; logs=$(cd -P "/tmp/linux-check/$sha" && pwd -P)

orb -m "$MACHINE" true >/dev/null 2>&1 || { echo "OrbStack machine $MACHINE is not running (orb start $MACHINE; fork/linux/setup.sh provisions it)"; exit $USAGE; }

# The run lock is a file holding the owner's pid, made whole under a temporary name and hard-linked
# into place, which is atomic and fails if the lock exists. A crashed run (dead pid) does not block the
# next one; two runs that start together right after a crash can both take over the stale lock, and then
# the guest lock below refuses the second.
echo "$$ $sha" >"$LOCK.$$"
take_lock() { ln "$LOCK.$$" "$LOCK" 2>/dev/null; }
if ! take_lock; then
  holder=$(cut -d' ' -f1 "$LOCK" 2>/dev/null || true)
  if [ -n "$holder" ] && kill -0 "$holder" 2>/dev/null; then
    rm -f "$LOCK.$$"
    echo "another check.sh run (pid $holder, $(cut -d' ' -f2 "$LOCK" 2>/dev/null)) holds the gate worktree"; exit $USAGE
  fi
  rm -f "$LOCK"
  take_lock || { rm -f "$LOCK.$$"; echo "cannot take $LOCK"; exit $USAGE; }
fi
rm -f "$LOCK.$$"
# Only the holder removes the lock.
trap '[ "$(cut -d" " -f1 "$LOCK" 2>/dev/null)" = "$$" ] && rm -f "$LOCK"' EXIT
# Removed by the guest, which writes the logs: a Mac-side rm made the guest's next open of the same
# name fail with "No such file or directory" now and then.
# The Mac must then see no log; its view of the share can lag, so it gets a few seconds.
if ! orb -m "$MACHINE" bash -c 'rm -f "$1"/*.log' rm-logs "$logs" >/dev/null 2>&1; then
  echo "[$sha] cannot remove the old logs in $logs"; exit $USAGE
fi
for i in 1 2 3 4 5; do
  ls "$logs"/*.log >/dev/null 2>&1 || break
  [ "$i" = 5 ] && { echo "[$sha] the old logs in $logs are still there after the guest removed them: not starting"; exit $USAGE; }
  sleep 1
done

# A killed orb client leaves its guest command running (and holding the guest lock until it ends), so an
# interrupt also stops the guest command's process tree. The guest commands are the `bash -c` processes
# whose $0 is lxgate-run.
stop_guest() {
  trap '' INT TERM HUP
  # The tree is listed first and signalled top-down, so that a loop in the guest script (the nextest
  # partitions) is gone before the command it is waiting for, and cannot start the next one.
  orb -m "$MACHINE" bash -c 'q=$(pgrep -f "^bash -c .*[l]xgate-run"); all=$q
    while [ -n "$q" ]; do n=; for p in $q; do n="$n $(pgrep -P "$p")"; done; n=$(echo $n); all="$all $n"; q=$n; done
    [ -z "$all" ] || kill -TERM $all 2>/dev/null' >/dev/null 2>&1
  exit 130
}
trap stop_guest INT TERM HUP

# Every guest command starts the same way: send its output to the log (exit 125 if it cannot), take
# the guest lock (exit 124 if held; the lock is a file descriptor, so the command's own exit codes
# cannot be mistaken for it) and enter the directory (exit 123 if it is missing; `orb -w` would fall back
# to / without a word). A refusal writes a `lxgate-refused:` line to the log. Codes 121 to 125 are the
# gate's own, clear of cargo's and nextest's (96, 100, 101). Arguments: lock, directory, log, then the
# script's own.
PRELUDE='refuse() { echo "lxgate-refused: $2" >&2; exit "$1"; }
exec >"$3" 2>&1 || exit 125
exec 9>"$1" || refuse 124 "cannot open the guest lock $1"
flock -n 9 || refuse 124 "gate busy: $1 is held (a killed run command may still be going)"
cd "$2" || refuse 123 "gate directory $2 is missing"
shift 3
'
# $1: directory, $2: log, $3: bash script, the rest: its arguments. Only the exit code is read from orb.
guest() {
  local dir=$1 log=$2 script=$3; shift 3
  orb -m "$MACHINE" bash -c "$PRELUDE$script" lxgate-run "$GUEST_LOCK" "$dir" "$log" "$@" >/dev/null
}

# CI's job env, on a clean base: a non-login shell (a login one exports the guest's LC_ALL=en_US.UTF-8
# and sources ~/.profile), the UTC and C.UTF-8 of the former Docker image, and none of the variables
# that would send cargo's output or compiler elsewhere. PATH is the guest's, with /usr/local/bin (TL's
# symlinks) and, through ~/.cargo/env, ~/.cargo/bin.
ENV_SETUP='export TZ=UTC LC_ALL=C.UTF-8
unset CARGO_TARGET_DIR RUSTC_WRAPPER SOURCE_DATE_EPOCH
. "$HOME/.cargo/env"
export TEXMFCNF=/usr/local/texlive/2026/texmf-dist/web2c CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 \
  CARGO_NET_GIT_FETCH_WITH_CLI=true RUSTFLAGS="-C link-arg=-fuse-ld=mold" RUST_BACKTRACE=1
# What the dumps depend on besides the commit: the dump files and the TeX Live install.
gate_state() {
  sha256sum resources/dumps/*.dump.txt 2>/dev/null
  tex --version | head -1
  sha256sum /usr/local/texlive/2026/tlpkg/texlive.tlpdb
  stat -c %Y /usr/local/texlive/2026/.setup-complete
}
'

# Fetch the commit, put the gate worktree on it, copy Cargo.lock, refresh the toolchain, clean the
# workspace members. Everything the gate needs from the guest (toolchain, TeX Live 2026, the clone and
# its remote) is checked first, and setup.sh is named when one is missing. Arguments: the guest clone,
# the gate worktree, the commit, the Cargo.lock to copy.
PREPARE="$ENV_SETUP"'
need() { echo "guest is not provisioned ($1): run fork/linux/setup.sh" >&2; exit 100; }
command -v cargo >/dev/null && command -v cargo-nextest >/dev/null || need "no cargo or cargo-nextest"
command -v mold >/dev/null || need "no mold"
tex --version 2>/dev/null | head -1 | grep -q "TeX Live 2026" || need "no TeX Live 2026"
[ -e /usr/local/texlive/2026/.setup-complete ] || need "the TeX Live 2026 install is incomplete (no .setup-complete)"
set -euo pipefail
clone=$1; gate=$2; sha=$3; lock=$4
[ -d "$clone/.git" ] || need "no guest clone $clone"
git -C "$clone" remote get-url mac >/dev/null 2>&1 || need "no remote mac in $clone"
git -C "$clone" config worktree.useRelativePaths true
# A bare sha first (the commit may be a detached HEAD, not a branch tip); if the Mac refuses it, take
# every branch and HEAD, which reaches any commit a branch or HEAD names.
if ! git -C "$clone" cat-file -e "$sha^{commit}" 2>/dev/null; then
  git -C "$clone" fetch -q mac "$sha" 2>/dev/null \
    || git -C "$clone" fetch -q mac "+refs/heads/*:refs/remotes/mac/*" HEAD
  git -C "$clone" cat-file -e "$sha^{commit}" || { echo "commit $sha is not reachable from any branch or HEAD of the Mac checkout" >&2; exit 100; }
fi
if [ ! -e "$gate/.git" ]; then
  mkdir -p "$(dirname "$gate")"
  git -C "$clone" worktree add -f -q --relative-paths --detach "$gate" "$sha"
fi
cd "$gate"
git checkout -fq --detach "$sha"
git clean -fdq
[ -z "$lock" ] || cp "$lock" Cargo.lock
rustup toolchain install nightly --profile minimal -c clippy,rustfmt,rust-src --no-self-update
mkdir -p target
if [ "$(cat target/.gate-rustc 2>/dev/null)" != "$(rustc -vV)" ]; then
  cargo clean --profile ci
  rustc -vV > target/.gate-rustc
fi
members=$(cargo metadata --no-deps --format-version 1 | python3 -c "import json,sys; print(*(\"-p \" + p[\"name\"] for p in json.load(sys.stdin)[\"packages\"]))")
cargo clean --profile ci $members
'

lockfile=
for l in "$REPO/Cargo.lock" "$main/Cargo.lock"; do
  [ -f "$l" ] && { lockfile=$l; break; }
done
echo "[$sha] preparing the gate worktree on $MACHINE ..."
# In the background and waited for, so that a signal runs the trap at once.
guest / "$logs/prepare.log" "$PREPARE" "$GUEST_CLONE" "$GATE" "$full" "$lockfile" & wait $!
rc=$?
if [ $rc -eq 125 ]; then echo "[$sha] log not writable: $logs/prepare.log"; else cat "$logs/prepare.log"; fi
[ $rc -eq 0 ] || { echo "[$sha] prepare FAILED, exit $rc ($logs/prepare.log)"; exit $USAGE; }

# A step: the worktree must be at the ref; nextest needs dumps built for it. Arguments: the commit,
# the command, and 1 when no `formats` ran earlier in this run.
STEP="$ENV_SETUP"'
[ "$(git rev-parse HEAD)" = "$1" ] || refuse 122 "gate worktree is at $(git rev-parse HEAD), not $1"
if [ "$3" = 1 ] && [ "$({ git rev-parse HEAD; gate_state; } 2>/dev/null)" != "$(cat resources/dumps/.gate-record 2>/dev/null)" ]; then
  refuse 121 "no dumps for this state (the commit, the dump files or TeX Live differ from resources/dumps/.gate-record): run the formats step first"
fi
eval "$2"
'
fails=0; formats_done=0
for s in "${steps[@]}"; do
  case $s in
    prepare) continue ;;
    formats) cmd='rm -f resources/dumps/.gate-record; PROFILE=ci tools/make_formats.sh && { git rev-parse HEAD; gate_state; } > resources/dumps/.gate-record' ;;
    nextest) cmd='nx="cargo nextest run --cargo-profile ci --profile ci --workspace"
      $nx --no-run; rc=$?
      if [ $rc -ne 0 ]; then echo "lxgate-result: build failed (exit $rc)"; exit $rc; fi
      failed=
      for i in 1 2 3 4; do
        echo "=== partition $i/4"
        $nx --no-fail-fast --partition hash:$i/4 || failed="$failed $i"
      done
      if [ -n "$failed" ]; then echo "lxgate-result: failed partitions:$failed"; exit 1; fi
      echo "lxgate-result: all 4 partitions passed"' ;;
    lint)    cmd="LINT_PROFILE=ci LINT_STRICT=1 tools/lint.sh" ;;
  esac
  need_dumps=0; [ "$s" = nextest ] && [ "$formats_done" = 0 ] && need_dumps=1
  echo "[$sha] $s ..."
  # In the background and waited for, so that a signal runs the trap at once instead of after the step.
  guest "$GATE" "$logs/$s.log" "$STEP" "$full" "$cmd" "$need_dumps" & wait $!
  rc=$?
  if [ $rc -eq 0 ]; then echo "[$sha] $s ok"; [ "$s" = formats ] && formats_done=1
  else echo "[$sha] $s FAILED, exit $rc ($logs/$s.log)"; fails=$((fails+1))
    # A refusal by the guest script, then a nextest partition verdict, are worth a line here.
    if [ "$rc" -eq 125 ]; then echo "[$sha]   log not writable: $logs/$s.log"
    elif [ "$rc" -ge 121 ] && [ "$rc" -le 124 ]; then grep -m1 '^lxgate-refused:' "$logs/$s.log" | sed "s/^/[$sha]   /"
    elif [ "$s" = nextest ]; then grep '^lxgate-result:' "$logs/$s.log" | tail -n 1 | sed "s/^/[$sha]   /"; fi
    # Without dumps every test takes the slow raw-load path and many fail spuriously.
    [ "$s" = formats ] && { echo "[$sha] stopping: no dumps"; exit $fails; }
  fi
done
exit $fails
