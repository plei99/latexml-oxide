# Linux mirror of upstream CI's `test` and `lint` jobs (.github/workflows/CI.yml): ubuntu-latest
# (24.04, TeX Live 2023 from apt), the same apt set, nightly Rust, nextest and mold.
# Built and driven by fork/linux/check.sh.
FROM ubuntu:24.04
ENV DEBIAN_FRONTEND=noninteractive TZ=UTC
RUN apt-get update -qq && apt-get install -qy --no-install-recommends \
      libxml2-dev libxml2-utils libxslt1-dev libkpathsea-dev libkpathsea6 \
      mold \
      texlive texlive-latex-extra texlive-science \
      texlive-bibtex-extra texlive-publishers \
      texlive-lang-german texlive-lang-french texlive-lang-greek \
      texlive-lang-cyrillic \
      build-essential pkg-config git curl ca-certificates \
    && rm -rf /var/lib/apt/lists/*
# Preinstalled on GitHub's runner image; bindgen build scripts need libclang.
RUN apt-get update -qq && apt-get install -qy --no-install-recommends libclang-dev clang \
    && rm -rf /var/lib/apt/lists/*
ENV RUSTUP_HOME=/opt/rustup CARGO_HOME=/opt/cargo PATH=/opt/cargo/bin:$PATH
RUN curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain nightly \
      --component clippy,rustfmt,rust-src
RUN curl -LsSf https://get.nexte.st/latest/linux-arm | tar zxf - -C /opt/cargo/bin
# CI's job env
ENV CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 RUSTFLAGS="-C link-arg=-fuse-ld=mold" RUST_BACKTRACE=1
WORKDIR /src
