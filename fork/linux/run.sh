#!/usr/bin/env bash
# Run a command in this checkout on the OrbStack machine `agents`, with the environment its builds and tests
# need there.
#
#   fork/linux/run.sh <command> [arg...]
#   fork/linux/run.sh cargo nextest run -p latexml --test cluster_package_guards -E 'test(=perfect_kernel_batch64::aastex7_email_show)'
#
# Run it from macOS on a guest worktree's macOS view (/Users/patrick/OrbStack/agents/home/...) or inside the
# guest; the command runs in the checkout's Linux path, from its top. A checkout outside the guest's
# /home/patrick (the Mac's own disk, also mounted in the guest) is refused: its target/ is a macOS build, which a
# guest build would overwrite. The environment, on top of the guest's login shell:
#   TEXMFCNF          TeX Live 2026's texmf.cnf. The `kpathsea` crate links Ubuntu's libkpathsea, which
#                     finds no texmf.cnf without it and falls back to a kpsewhich subprocess.
#   PATH              TeX Live 2026's binaries first.
#   RUSTFLAGS         mold as the linker, as fork/linux/check.sh links, unless set.
#   CARGO_BUILD_JOBS  6 unless set: 13 parallel rustc jobs of the test profile (full debuginfo) outgrew the
#                     machine's memory and rustc was killed (SIGKILL) compiling latexml_package.
# From macOS, these two are taken from the caller's environment; orb passes no other variable.
# Commits, the gate and the dumps record are fork/linux/check.sh's; this only runs a command.
set -euo pipefail
MACHINE=agents
GUEST_HOME=/home/patrick
MAC_VIEW=/Users/patrick/OrbStack/$MACHINE   # the guest's / as macOS sees it

[ $# -gt 0 ] && [ "$1" != -h ] && [ "$1" != --help ] || {
  sed -n '2,19p' "$0" | sed 's/^# \{0,1\}//' >&2
  exit 2
}
top=$(git -C "$(dirname "$0")" rev-parse --show-toplevel)

ENV='export TEXMFCNF=/usr/local/texlive/2026/texmf-dist/web2c PATH=/usr/local/texlive/2026/bin/$(uname -m)-linux:$PATH
export RUSTFLAGS="${RUSTFLAGS:--C link-arg=-fuse-ld=mold}" CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-6}"
cd "$1" || exit 2; shift; exec "$@"'

refuse() { echo "fork/linux/run.sh: $top is on the Mac's disk, not a guest worktree; make one with fork/worktree.sh" >&2; exit 2; }
if [ "$(uname -s)" = Darwin ]; then
  case $top in
    "$MAC_VIEW$GUEST_HOME"/*) guest_top=${top#"$MAC_VIEW"} ;;
    *) refuse ;;
  esac
  orb -m "$MACHINE" true >/dev/null 2>&1 || { echo "fork/linux/run.sh: machine $MACHINE is not running (orb start $MACHINE)" >&2; exit 2; }
  pass=()
  [ -z "${CARGO_BUILD_JOBS:-}" ] || pass+=("CARGO_BUILD_JOBS=$CARGO_BUILD_JOBS")
  [ -z "${RUSTFLAGS:-}" ] || pass+=("RUSTFLAGS=$RUSTFLAGS")
  exec orb -m "$MACHINE" env ${pass[@]+"${pass[@]}"} bash -lc "$ENV" run.sh "$guest_top" "$@"
fi
case $top in "$GUEST_HOME"/*) ;; *) refuse ;; esac
exec bash -lc "$ENV" run.sh "$top" "$@"
