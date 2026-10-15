#!/usr/bin/env bash
# binary_info.sh [bin] — one line naming a latexml_oxide build: its path, sha256 prefix and build time, this
# checkout's HEAD (-dirty for uncommitted changes outside fork/), and STALE when a Rust source, a manifest or a
# resource of the checkout is newer than the binary, so the binary does not contain the working tree.
# Exit 2 when there is no binary.
set -uo pipefail
REPO="$(cd "$(dirname "$0")/.." && pwd)"
bin=${1:-$REPO/target/release/latexml_oxide}
[ -x "$bin" ] || { echo "no binary at $bin" >&2; exit 2; }
sha=$(shasum -a 256 "$bin" | cut -c1-12)
built=$(date -r "$bin" '+%Y-%m-%d %H:%M:%S')
head=$(git -C "$REPO" rev-parse --short=9 HEAD)$(git -C "$REPO" diff --quiet HEAD -- . ':!fork' || echo -dirty)
newer=$(cd "$REPO" && find latexml_* Cargo.toml Cargo.lock -path '*/tests' -prune -o -newer "$bin" -type f \
  \( -name '*.rs' -o -name 'Cargo.*' -o -path '*/resources/*' \) -print -quit 2>/dev/null)
printf '%s\tsha256:%s\tbuilt %s\tcheckout %s%s\n' "$bin" "$sha" "$built" "$head" "${newer:+	STALE: $newer is newer}"
