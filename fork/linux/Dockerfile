# The fork's Linux gate: ubuntu:24.04, TeX Live 2026 scheme-full from install-tl (the Mac runs
# 2026 scheme-full too, so version- and package-sensitive guards agree between hosts), nightly
# Rust, nextest and mold, with upstream CI's `ci` profile and job settings.
# Built and driven by fork/linux/check.sh.
ARG TL_REPO=https://texlive.info/tlnet-archive/2026/03/02/tlnet
FROM ubuntu:24.04 AS tlrepo
ARG TL_REPO
RUN apt-get update -qq && apt-get install -qy --no-install-recommends curl ca-certificates \
    && rm -rf /var/lib/apt/lists/*
RUN set -eux; \
    case "$(uname -m)" in aarch64) plat=aarch64-linux ;; x86_64) plat=x86_64-linux ;; *) exit 1 ;; esac; \
    others=$(printf '%s\n' aarch64-linux aarch64-linuxmusl amd64-freebsd amd64-netbsd armhf-linux \
      i386-freebsd i386-linux i386-netbsd i386-solaris universal-darwin win32 windows x86_64-darwin \
      x86_64-darwinlegacy x86_64-linux x86_64-linuxmusl x86_64-solaris | grep -vx "$plat" | paste -sd'|'); \
    mkdir -p /tlrepo/archive /tlrepo/tlpkg; cd /tlrepo; \
    curl -fsS "$TL_REPO/install-tl-unx.tar.gz" -o install-tl-unx.tar.gz; \
    curl -fsS "$TL_REPO/tlpkg/texlive.tlpdb" -o tlpkg/texlive.tlpdb; \
    curl -fsS "$TL_REPO/archive/" \
      | grep -o 'href="[^"?/][^"]*\.tar\.xz"' | sed 's/^href="//;s/"$//' \
      | grep -vE '\.(doc|source)\.tar\.xz$|\.r[0-9]+\.tar\.xz$' \
      | grep -vE "\.($others)\.tar\.xz\$" > /tmp/need.txt; \
    cd archive; \
    sed "s#^#$TL_REPO/archive/#" /tmp/need.txt \
      | xargs -P 48 -n 10 curl -fsS --retry 5 --retry-all-errors --remote-name-all; \
    test "$(ls | wc -l)" = "$(wc -l < /tmp/need.txt)"

FROM ubuntu:24.04
ENV DEBIAN_FRONTEND=noninteractive TZ=UTC
# No apt `texlive*` packages: a second TeX Live tree (Ubuntu's is 2023) once ran a dump built from
# one TL with another's packages. perl, xz-utils and the fontconfig/freetype runtime libraries are
# what install-tl and TL's own binaries (xetex) need; they come from apt, TeX itself does not.
RUN apt-get update -qq && apt-get install -qy --no-install-recommends \
      libxml2-dev libxml2-utils libxslt1-dev libkpathsea-dev libkpathsea6 \
      mold \
      perl xz-utils libfontconfig1 libfreetype6 \
      build-essential pkg-config git curl ca-certificates \
    && rm -rf /var/lib/apt/lists/*
# TeX Live 2026, scheme-full, no docs or sources, pinned to the snapshot the Mac's TL 2026 was
# installed from (2 March 2026, never updated), so both hosts have identical package revisions. When
# the Mac's TL is updated (the user does that, not an agent), move TL_REPO's date to match; pass
# another repository with --build-arg TL_REPO=... . The current tlnet (LaTeX 2026-06-01) fails
# `--init=latex.ltx` with 137 errors, the first at the final `\@input` of
# latex2e-first-aid-for-external-files.ltx, so moving the date past that needs engine work.
# The build fails unless the result says TeX Live 2026.
# The snapshot host answers one small file per ~1.3 s, and install-tl fetches ~5000 packages one
# after another (over 2 hours), so the stage `tlrepo` first copies the repository (install-tl, the
# tlpdb and the unversioned, non-doc, non-source archives for this arch) with parallel curl, and
# install-tl then installs from that local copy, which is mounted and so never enters the image.
# TL's bin dir is per-arch (aarch64-linux on Apple silicon), so `tlmgr path add` symlinks the
# binaries into /usr/local/bin instead of PATH naming the arch.
RUN --mount=type=bind,from=tlrepo,source=/tlrepo,target=/tlrepo \
    set -eux; \
    mkdir /tmp/tl-dl; \
    tar -xzf /tlrepo/install-tl-unx.tar.gz -C /tmp/tl-dl --strip-components=1; \
    printf '%s\n' \
      'selected_scheme scheme-full' \
      'TEXDIR /usr/local/texlive/2026' \
      'TEXMFLOCAL /usr/local/texlive/texmf-local' \
      'TEXMFSYSVAR /usr/local/texlive/2026/texmf-var' \
      'TEXMFSYSCONFIG /usr/local/texlive/2026/texmf-config' \
      'tlpdbopt_install_docfiles 0' \
      'tlpdbopt_install_srcfiles 0' \
      'tlpdbopt_autobackup 0' \
      'instopt_letter 0' \
      'instopt_adjustpath 0' \
      > /tmp/tl.profile; \
    perl /tmp/tl-dl/install-tl -profile /tmp/tl.profile -repository /tlrepo; \
    /usr/local/texlive/2026/bin/*/tlmgr path add; \
    tex --version | head -1 | tee /dev/stderr | grep -q 'TeX Live 2026'; \
    rm -rf /tmp/tl-dl /tmp/tl.profile /usr/local/texlive/2026/install-tl*
# The kpathsea crate links Ubuntu's libkpathsea (apt, 6.3.5) in-process and anchors it on the
# `kpsewhich` found on PATH: the /usr/local/bin symlink into TL 2026, which kpathsea follows to the
# real binary. Debian compiles in a texmf.cnf search path (/etc/texmf/web2c, /usr/share/texlive/...,
# $SELFAUTOPARENT/share/texmf/web2c) that does not include TL's texmf-dist/web2c, so with no
# TEXMFCNF the library reports "texmf.cnf not found" and resolves nothing. TEXMFCNF names TL 2026's
# directly; $TEXMFROOT and the other tree variables then derive from the same binary anchor, so
# every path resolves under /usr/local/texlive/2026.
ENV TEXMFCNF=/usr/local/texlive/2026/texmf-dist/web2c
# Preinstalled on GitHub's runner image; bindgen build scripts need libclang. python3: tools/lint.sh's
# audit and fontmap-drift checks; jing: the schema guards (apt's texlive pulled python3 in before).
RUN apt-get update -qq && apt-get install -qy --no-install-recommends libclang-dev clang python3 jing \
    && rm -rf /var/lib/apt/lists/*
ENV RUSTUP_HOME=/opt/rustup CARGO_HOME=/opt/cargo PATH=/opt/cargo/bin:$PATH
RUN curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain nightly \
      --component clippy,rustfmt,rust-src
RUN curl -LsSf https://get.nexte.st/latest/linux-arm | tar zxf - -C /opt/cargo/bin
# CI's job env
ENV CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 RUSTFLAGS="-C link-arg=-fuse-ld=mold" RUST_BACKTRACE=1
WORKDIR /src
