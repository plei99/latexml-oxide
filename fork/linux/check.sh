#!/usr/bin/env bash
# Run the fork's Linux gates (upstream CI's test and lint jobs, on TeX Live 2026 scheme-full) on a
# committed ref, in Docker.
#
#   fork/linux/check.sh [ref] [step...]     steps: formats nextest lint (default: all three)
#
# The ref is exported with `git archive`, so only committed code is tested and the macOS
# checkout (its target/ and its dumps) is never touched. Named volumes keep the cargo
# registry, the Linux target/ and the rustup toolchains between runs, as rust-cache does on CI;
# rust-toolchain.toml floats nightly, so rustup still updates it when a new one ships. Cargo.lock is
# gitignored, so the main checkout's is copied in (a fresh resolve fails while every zip 8.x is
# yanked) and both hosts build the same dependency versions. Logs go to
# /tmp/linux-check/<ref>/<step>.log; the exit code is the number of failed steps.
set -uo pipefail
REPO=$(cd "$(dirname "$0")/../.." && pwd)
ref=${1:-HEAD}; shift || true
steps=("${@:-formats nextest lint}"); steps=(${steps[*]})
sha=$(git -C "$REPO" rev-parse --short "$ref") || exit 2
logs=/tmp/linux-check/$sha; mkdir -p "$logs"
image=latexml-oxide-ci

docker build -q -t "$image" "$REPO/fork/linux" >/dev/null || exit 2
# A fresh source volume per run; target/ and the registry persist.
docker volume rm -f lxo-src >/dev/null
git -C "$REPO" archive "$ref" | docker run --rm -i -v lxo-src:/src "$image" tar -x -C /src
main=$(dirname "$(git -C "$REPO" rev-parse --path-format=absolute --git-common-dir)")
for lock in "$REPO/Cargo.lock" "$main/Cargo.lock"; do
  [ -f "$lock" ] && { docker run --rm -i -v lxo-src:/src "$image" sh -c 'cat > /src/Cargo.lock' <"$lock"; break; }
done
run() {
  docker run --rm -v lxo-src:/src -v lxo-target:/src/target -v lxo-registry:/opt/cargo/registry \
    -v lxo-rustup:/opt/rustup "$image" bash -c "$1"
}
fails=0
for s in "${steps[@]}"; do
  case $s in
    formats) cmd="PROFILE=ci tools/make_formats.sh" ;;
    nextest) cmd="cargo nextest run --cargo-profile ci --profile ci --workspace --no-fail-fast" ;;
    lint)    cmd="LINT_PROFILE=ci tools/lint.sh" ;;
    *) echo "unknown step $s"; exit 2 ;;
  esac
  echo "[$sha] $s ..."
  if run "$cmd" >"$logs/$s.log" 2>&1; then echo "[$sha] $s ok"; else echo "[$sha] $s FAILED ($logs/$s.log)"; fails=$((fails+1))
    # Without dumps every test takes the slow raw-load path and many fail spuriously.
    [ "$s" = formats ] && { echo "[$sha] stopping: no dumps"; exit $fails; }
  fi
done
exit $fails
