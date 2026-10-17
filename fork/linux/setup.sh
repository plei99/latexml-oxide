#!/usr/bin/env bash
# Provision the OrbStack machine `agents` (Ubuntu, arm64 or x86_64) for the fork's Linux gate. All Linux
# testing for this fork runs on that machine; fork/linux/check.sh drives it.
#
#   fork/linux/setup.sh [step...]     steps: apt rust texlive clone (default: all four)
#
# Run it from macOS (it hands itself to the guest through `orb -m agents bash -c`) or inside the guest.
# Every step is idempotent: a step that finds its result already there does nothing, so re-running
# repairs a missing or partial install. The rust step also re-downloads nextest each time, so that is
# how nextest is kept current; the nightly toolchain is refreshed by fork/linux/check.sh on every run.
# What it sets up:
#   apt      build dependencies and the runtime libraries TeX Live's own binaries need
#   rust     rustup with nightly, cargo-nextest and cargo-machete (CI's lint job runs it), in ~/.rustup
#            and ~/.cargo
#   texlive  TeX Live 2026 scheme-full in /usr/local/texlive/2026, from the pinned snapshot below; the
#            marker file .setup-complete says an install finished
#   clone    the guest clone's settings (worktree.useRelativePaths, the remote `mac`); the clone itself,
#            /home/patrick/agent-work/repos/latexml, must already exist
set -euo pipefail

MACHINE=agents
# The Mac checkout, which the guest reads through the shared /Users and fetches from as remote `mac`.
MAC_REPO=${MAC_REPO:-/Users/patrick/.t3/projects/latexml}
GUEST_CLONE=/home/patrick/agent-work/repos/latexml
# TeX Live 2026, scheme-full, no docs or sources, pinned to the snapshot whose package revisions
# match the Mac's TL 2026 (16 October 2026: LaTeX 2026-06-01, l3kernel 2026-09-09), so both hosts
# have identical package revisions. When the Mac's TL is updated (the user does that, not an agent),
# move TL_REPO's date to the snapshot whose texlive.tlpdb revisions match the Mac's, move MIRROR's
# date with it, and re-run the texlive step on a machine whose /usr/local/texlive/2026 was removed.
# The step fails unless `tex --version` says TeX Live 2026.
TL_REPO=${TL_REPO:-https://texlive.info/tlnet-archive/2026/10/16/tlnet}
MIRROR=$HOME/agent-work/cache/tlnet-2026-10-16
TL_DIR=/usr/local/texlive/2026

if [ "$(uname -s)" = Darwin ]; then
  # Mac side: run this script in the guest. The checkout may be a worktree, whose git
  # common dir is the main checkout that the guest fetches from.
  repo=$(cd "$(dirname "$0")/../.." && pwd)
  main=$(dirname "$(git -C "$repo" rev-parse --path-format=absolute --git-common-dir)")
  # The script travels as an argument, not on stdin, which install-tl and apt may read.
  exec orb -m "$MACHINE" env MAC_REPO="$main" bash -c "$(cat "$0")" setup.sh "$@"
fi

steps=("$@"); [ ${#steps[@]} -gt 0 ] || steps=(apt rust texlive clone)

step_apt() {
  # No apt `texlive*` packages: a second TeX Live tree (Ubuntu's is older) once ran a dump built from
  # one TL with another's packages. perl, xz-utils and the fontconfig/freetype runtime libraries are
  # what install-tl and TL's own binaries (xetex) need; they come from apt, TeX itself does not.
  # libkpathsea-dev is what the `kpathsea` crate links; libclang-dev and clang are what bindgen build
  # scripts need (preinstalled on GitHub's runner image); python3 is tools/lint.sh's audit and
  # fontmap-drift checks; jing runs the schema guards.
  local pkgs=(libxml2-dev libxml2-utils libxslt1-dev libkpathsea-dev libkpathsea6 mold
    perl xz-utils libfontconfig1 libfreetype6 build-essential pkg-config git curl ca-certificates
    libclang-dev clang python3 jing)
  local missing=() p
  for p in "${pkgs[@]}"; do dpkg -s "$p" >/dev/null 2>&1 || missing+=("$p"); done
  if [ ${#missing[@]} -gt 0 ]; then
    sudo DEBIAN_FRONTEND=noninteractive apt-get update -qq
    sudo DEBIAN_FRONTEND=noninteractive apt-get install -qy --no-install-recommends "${missing[@]}"
  fi
  echo "apt: ok"
}

step_rust() {
  # rust-toolchain.toml floats nightly; check.sh installs the current one before every run.
  if [ ! -x "$HOME/.cargo/bin/rustup" ]; then
    curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain nightly \
      --component clippy,rustfmt,rust-src
  fi
  # Always re-downloaded: nextest has no other update path here.
  case "$(uname -m)" in aarch64) nx=linux-arm ;; x86_64) nx=linux ;; *) echo "unsupported arch" >&2; exit 1 ;; esac
  curl -LsSf "https://get.nexte.st/latest/$nx" | tar zxf - -C "$HOME/.cargo/bin"
  "$HOME/.cargo/bin/cargo-nextest" nextest --version | head -1
  # cargo-machete, as CI's lint job installs it (the latest release). Its release page has a
  # prebuilt binary for aarch64-unknown-linux-gnu and x86_64-unknown-linux-musl; a version that is
  # already installed is kept.
  local tag cur mt
  tag=$(curl -fsSLI -o /dev/null -w '%{url_effective}' https://github.com/bnjbvr/cargo-machete/releases/latest)
  tag=${tag##*/}
  cur=$("$HOME/.cargo/bin/cargo-machete" --version 2>/dev/null | awk '{print $NF}' || true)
  if [ "v$cur" != "$tag" ]; then
    case "$(uname -m)" in aarch64) mt=aarch64-unknown-linux-gnu ;; x86_64) mt=x86_64-unknown-linux-musl ;; *) echo "unsupported arch" >&2; exit 1 ;; esac
    mkdir -p /tmp/machete-dl
    curl -fsSL "https://github.com/bnjbvr/cargo-machete/releases/download/$tag/cargo-machete-$tag-$mt.tar.gz" \
      | tar zxf - -C /tmp/machete-dl --strip-components=1
    install -m 755 /tmp/machete-dl/cargo-machete "$HOME/.cargo/bin/cargo-machete"
    rm -rf /tmp/machete-dl
  fi
  "$HOME/.cargo/bin/cargo-machete" --version
  echo "rust: ok"
}

tl_mirror() {
  # The snapshot host answers one small file per ~1.3 s, and install-tl fetches ~5000 packages one
  # after another (over 2 hours), so the repository (install-tl, the tlpdb and the unversioned,
  # non-doc, non-source archives for this arch) is copied first with parallel curl, resumably, and
  # install-tl then installs from that local copy. An archive whose size differs from the tlpdb's
  # `containersize` (an interrupted or reset download) is deleted and fetched again.
  # GNU comm needs the byte order that C-locale sort gives; the guest's en_US.UTF-8 would not.
  export LC_ALL=C
  local plat others
  case "$(uname -m)" in aarch64) plat=aarch64-linux ;; x86_64) plat=x86_64-linux ;; *) echo "unsupported arch" >&2; exit 1 ;; esac
  others=$(printf '%s\n' aarch64-linux aarch64-linuxmusl amd64-freebsd amd64-netbsd armhf-linux \
    i386-freebsd i386-linux i386-netbsd i386-solaris universal-darwin win32 windows x86_64-cygwin \
    x86_64-darwin x86_64-darwinlegacy x86_64-linux x86_64-linuxmusl x86_64-solaris | grep -vx "$plat" | paste -sd'|')
  mkdir -p "$MIRROR/archive" "$MIRROR/tlpkg"; cd "$MIRROR"
  curl -fsS "$TL_REPO/install-tl-unx.tar.gz" -o install-tl-unx.tar.gz
  curl -fsS "$TL_REPO/tlpkg/texlive.tlpdb" -o tlpkg/texlive.tlpdb
  curl -fsS "$TL_REPO/archive/" \
    | grep -o 'href="[^"?/][^"]*\.tar\.xz"' | sed 's/^href="//;s/"$//' \
    | grep -vE '\.(doc|source)\.tar\.xz$|\.r[0-9]+\.tar\.xz$' \
    | grep -vE "\.($others)\.tar\.xz\$" | sort > need.txt
  # The host resets connections under load ("Connection reset by peer", "Broken pipe" at -P 48), so
  # the download runs in rounds at -P 16: each round drops bad archives, fetches what is missing, and
  # a failing curl does not end the step. Rounds repeat (at most 5) while the missing count drops; the
  # step fails if anything is still missing.
  local round missing prev=-1
  for round in 1 2 3 4 5; do
    tl_drop_bad_archives "$MIRROR"
    missing=$(cd archive && comm -23 ../need.txt <(ls | sort) | wc -l)
    echo "mirror: round $round, $missing archives missing"
    [ "$missing" -eq 0 ] && return 0
    { [ "$prev" -ge 0 ] && [ "$missing" -ge "$prev" ]; } && break
    prev=$missing
    (cd archive && comm -23 ../need.txt <(ls | sort) | sed "s#^#$TL_REPO/archive/#" \
      | xargs -r -P 16 -n 10 curl -fsS --retry 5 --retry-all-errors --remote-name-all) || true
  done
  tl_drop_bad_archives "$MIRROR"
  missing=$(cd archive && comm -23 ../need.txt <(ls | sort) | wc -l)
  [ "$missing" -eq 0 ] || { echo "mirror: $missing archives still missing" >&2; return 1; }
}

# Delete the archives in $1/archive whose size differs from the `containersize` of the tlpdb entry
# of the same name: archive/<name>.tar.xz is the entry `name <name>`, and an architecture's binaries,
# <name>.<arch>.tar.xz, have their own entry `name <name>.<arch>`.
tl_drop_bad_archives() {
  local dir=$1 f want have
  awk '$1 == "name" { n = $2 } $1 == "containersize" { print n ".tar.xz", $2 }' "$dir/tlpkg/texlive.tlpdb" \
    | while read -r f want; do
        [ -f "$dir/archive/$f" ] || continue
        have=$(wc -c <"$dir/archive/$f")
        if [ "$have" != "$want" ]; then echo "mirror: $f is $have bytes, tlpdb says $want: removed" >&2; rm -f "$dir/archive/$f"; fi
      done
}

step_texlive() {
  # An install-tl that stopped halfway leaves bin/*/tex behind, so completeness is the marker, which is
  # written only after install-tl and the 2026 check have succeeded.
  local fresh=0
  if [ ! -e "$TL_DIR/.setup-complete" ]; then
    fresh=1
    if [ -e "$TL_DIR" ]; then echo "texlive: $TL_DIR has no .setup-complete marker (a partial install): removing it" >&2; rm -rf "$TL_DIR"; fi
    (tl_mirror)
    local dl=/tmp/tl-dl profile=/tmp/tl.profile
    rm -rf "$dl"; mkdir "$dl"
    tar -xzf "$MIRROR/install-tl-unx.tar.gz" -C "$dl" --strip-components=1
    sudo mkdir -p /usr/local/texlive; sudo chown "$USER" /usr/local/texlive
    printf '%s\n' \
      'selected_scheme scheme-full' \
      "TEXDIR $TL_DIR" \
      'TEXMFLOCAL /usr/local/texlive/texmf-local' \
      "TEXMFSYSVAR $TL_DIR/texmf-var" \
      "TEXMFSYSCONFIG $TL_DIR/texmf-config" \
      'tlpdbopt_install_docfiles 0' \
      'tlpdbopt_install_srcfiles 0' \
      'tlpdbopt_autobackup 0' \
      'instopt_letter 0' \
      'instopt_adjustpath 0' \
      >"$profile"
    perl "$dl/install-tl" -profile "$profile" -repository "$MIRROR"
    rm -rf "$dl" "$profile"
  fi
  # TL's bin dir is per-arch (aarch64-linux on Apple silicon), so `tlmgr path add` symlinks the
  # binaries into /usr/local/bin instead of PATH naming the arch.
  sudo "$(ls "$TL_DIR"/bin/*/tlmgr | head -1)" path add
  # The kpathsea crate links Ubuntu's libkpathsea in-process and anchors it on the `kpsewhich` found
  # on PATH: the /usr/local/bin symlink into TL 2026, which kpathsea follows to the real binary.
  # Ubuntu compiles in a texmf.cnf search path (/etc/texmf/web2c, /usr/share/texlive/...) that does not
  # include TL's texmf-dist/web2c, so with no TEXMFCNF the library reports "texmf.cnf not found" and
  # resolves nothing. fork/linux/check.sh therefore sets
  # TEXMFCNF=/usr/local/texlive/2026/texmf-dist/web2c; $TEXMFROOT and the other tree variables then
  # derive from the same binary anchor, so every path resolves under /usr/local/texlive/2026.
  local v
  v=$(tex --version | head -1)
  echo "$v"
  case $v in *"TeX Live 2026"*) ;; *) echo "texlive: tex --version does not say TeX Live 2026" >&2; exit 1 ;; esac
  # Only after an install: check.sh's dumps record holds the marker's time.
  [ "$fresh" = 0 ] || touch "$TL_DIR/.setup-complete"
  echo "texlive: ok"
}

step_clone() {
  [ -d "$GUEST_CLONE/.git" ] || { echo "clone: $GUEST_CLONE does not exist" >&2; exit 1; }
  # Only configuration and a remote: the clone's own working tree is never touched.
  git -C "$GUEST_CLONE" config worktree.useRelativePaths true
  if ! git -C "$GUEST_CLONE" remote get-url mac >/dev/null 2>&1; then
    git -C "$GUEST_CLONE" remote add mac "$MAC_REPO"
  fi
  echo "clone: remote mac = $(git -C "$GUEST_CLONE" remote get-url mac)"
}

for s in "${steps[@]}"; do
  case $s in
    apt|rust|texlive|clone) "step_$s" ;;
    *) echo "unknown step $s (steps: apt rust texlive clone)" >&2; exit 2 ;;
  esac
done
