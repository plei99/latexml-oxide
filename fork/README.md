# Fork tooling (plei99/latexml-oxide)

This fork tracks `upstream/perfect_kernel` and adds what is needed to use oxide on one author's
notes and papers. Everything fork-specific lives under `fork/`, so merges from upstream never
touch it. Personal sources and outputs never enter git: they live in `.corpus/`, which is listed
in `.git/info/exclude`.

Every conversion here uses upstream's perfect-kernel protocol: raw loading of `.sty`/`.cls`
via `--preload=[rawstyles,rawclasses]latexml.sty`, a fixed `SOURCE_DATE_EPOCH`, and the release
binary. Build the binary with `cargo build --release`. Rebuild the dumps after a TeX Live
upgrade or an upstream merge with `PROFILE=release tools/make_formats.sh`.

Agents start at [`AGENTS.md`](AGENTS.md): where this checkout differs from upstream's instructions, and how
parallel agents share it. Three tools serve any change:

```sh
fork/oxide doc.tex [outdir]        # convert one local document under the protocol; prints the verdict
fork/binary_info.sh [bin]          # which build a binary is, and STALE when the tree is newer
fork/worktree.sh <branch>          # a writer agent's own checkout, with LaTeXML/, dumps and .corpus/
```

Fork-driven feature work has its own worklist: [`MEMOIR_ROADMAP.md`](MEMOIR_ROADMAP.md) for the memoir class.

## Two indexes

| | `fork/fixtures/` (public) | `fork/corpus/` over `.corpus/` (private) |
|---|---|---|
| Content | Synthetic feature snippets for each package in use, modelled on how the corpus uses them | The author's actual documents (`.corpus/corpus.tsv`: id, path) |
| Answers | What correct output looks like for each feature, and whether oxide produces it | Did a change break any real document? |
| Reference | Each feature's own pdfLaTeX PDF | Each document's PDF, typeset with its own engine (`refs.sh`) |
| Verdict | `fixtures/STATUS.tsv`: ok / partial / wrong, reviewed side by side | `compare.py` against the accepted baseline |

## Fixture loop

`fork/fixtures/<pkg>.tex` holds one preamble and a series of features. Each feature starts with
`%% feature: <slug> | <description>`. `review.sh` does the following:

1. splits each fixture into one mini-document per feature;
2. typesets references for the features that changed;
3. converts every feature and screenshots both sides;
4. writes `.corpus/fixtures/runs/<label>/gallery.html`, with the reference on the left, oxide on
   the right, and the recorded verdict;
5. compares the run against the accepted fixture run.

```sh
fork/fixtures/review.sh                    # label fx-<HEAD>
CORPUS=.corpus/fixtures fork/corpus/accept.sh fx-<HEAD>
```

When a fix lands, flip the feature's row in `STATUS.tsv` and accept the new run. A feature
listed as `ok` that changes picture digests or loses words is a regression.

## Corpus loop

```sh
fork/corpus/refs.sh                        # reference PDFs; rerun after editing documents or upgrading TeX Live
fork/corpus/run.sh base-<sha>              # convert everything; JOBS=6 by default, about 1 h
fork/corpus/accept.sh base-<sha>           # make it the baseline, recorded in .corpus git history
fork/corpus/check.sh                       # after a change: convert again and compare; exits 1 on regression
fork/corpus/check.sh '' '0(0[1-9]|1.)'     # the same over a subset (id regex)
```

`compare.py` treats any of the following as a regression:

- a worse status;
- more errors or a new error class;
- more `ltx_ERROR` marks;
- fewer `<math>` or `<svg>` elements;
- recall against the PDF down by more than 0.2 points;
- lost visible words.

A changed SVG digest on its own is reported as a change. Look at it before accepting.

Per-document facts are written to `runs/<label>/index/{docs,errors,pictures}.tsv`. Their
meanings are documented in `corpus/index.py`.

## Linux gate, locally

All Linux testing for the fork runs on the persistent OrbStack machine `agents` (Ubuntu 26.04, arm64, 13
CPUs; its memory is OrbStack's `machine.agents.memory_mib`, 12 GiB as of writing, see `orb config show`;
the cgroup allows no swap although the VM has some).
`fork/linux/check.sh` runs upstream CI's `test` and `lint` jobs there on **TeX Live 2026, scheme-full**,
installed with `install-tl`, so its package set matches the Mac's. The apt `texlive*` packages are not
installed. The `ci` profile, 2 build jobs, 2 test threads and mold are CI's settings, kept as they are. A
macOS run still differs from this gate in two ways:

- Lints for Linux-only `cfg` code fire only on macOS, or never there.
- The machine is not upstream's runner. CI uses GitHub's `ubuntu-latest` image, not Ubuntu 26.04, so
  libxml2, clang and libkpathsea can be other versions than CI's; and each of CI's four test
  partitions gets its own 16 GB runner, where here one machine's memory is shared by whatever runs on
  it.

```sh
fork/linux/setup.sh                                   # provision the machine; idempotent
fork/linux/check.sh <branch> [formats nextest lint]   # logs: /tmp/linux-check/<sha>/<step>.log
```

`setup.sh` installs the apt build dependencies, rustup with nightly, nextest and cargo-machete (in
`~/.rustup` and `~/.cargo`), and TeX Live 2026 in `/usr/local/texlive/2026`, then sets `worktree.useRelativePaths` in the
guest clone and adds the remote `mac`. Run it again to repair the machine; `setup.sh texlive` runs one
step (`apt rust texlive clone`). The rust step downloads nextest again each time, which is how nextest
is updated, and downloads cargo-machete's latest release unless that version is installed. `install-tl` pulls TL from `TL_REPO`, which defaults to the 16 October 2026 `tlnet`
snapshot, whose package revisions match the Mac's TL 2026: when the Mac's TL is updated, move the date
to the snapshot whose `texlive.tlpdb` revisions match it. The snapshot is copied first to
`~/agent-work/cache/tlnet-2026-10-16` with parallel curl, because its host answers one file per 1.3 s
and resets connections under load: the copy runs in rounds, and an archive whose size differs from the
tlpdb's is deleted and fetched again. The install writes `/usr/local/texlive/2026/.setup-complete` once,
when it and the check succeed (a later run leaves the marker and its time alone, because the dumps record
holds that time); a TL directory without the marker is a partial install and is removed and
installed again. The script fails unless `tex --version` says TeX Live 2026. The `kpathsea` crate links
Ubuntu's libkpathsea in-process, so `check.sh` sets `TEXMFCNF` to TL 2026's `texmf.cnf`. The Mac build
has no libkpathsea and runs `kpsewhich` as a subprocess.

`check.sh` resolves the branch to a commit, fetches it into the guest clone
(`/home/patrick/agent-work/repos/latexml`, remote `mac` = the Mac checkout) and checks it out, detached,
in the worktree `/home/patrick/agent-work/worktrees/latexml/linux-gate`, with `git checkout -f` and `git
clean -fd`: anything done by hand in that worktree is overwritten by the next run. Only committed code is
tested and the Mac checkout is never touched. The guest clone's own working tree is never touched either.
Each run first installs the current nightly (CI installs a fresh one each run), copies in `Cargo.lock`
(the invoking checkout's, else the Mac main checkout's) and runs `cargo clean -p` on every workspace
member, because a new test fixture is globbed in at compile time and a kept build would not see it. When
`rustc -vV` differs from the last run's, `target/ci` is emptied altogether, so it does not grow with each
nightly. The other dependencies in the worktree's `target/` stay cached, as `rust-cache` keeps them on CI.

The steps run in a non-login shell with CI's job environment plus `TZ=UTC` and `LC_ALL=C.UTF-8` (the lint
step also sets `LINT_STRICT=1`, as CI does, so a missing lint tool fails it), and without
`CARGO_TARGET_DIR`, `RUSTC_WRAPPER` and `SOURCE_DATE_EPOCH`. The lint step here also runs the
fontmap-drift check, which CI's lint job skips because it has no TeX Live. The `nextest` step follows CI's
test job: it builds once (`--no-run`), then runs the four partitions `hash:1/4` to `hash:4/4` one after
another, each with `--no-fail-fast`, all in `nextest.log`. The step fails if any partition fails, and the
log's last line names which. The partitions run one after another, as CI runs them on four runners. With the machine at 8 GiB, two
`114_streaming_cluster_regressions` shards, about 4.1 GB each, were killed by the cgroup OOM killer in one
2-thread nextest pass. Today the two big shards land in different partitions (the first in 1/4, the second
in 4/4) only because of where the hash puts them, not by any rule, so a change to the test names can bring
them together again. An OOM-killed test still fails its partition, so it cannot pass falsely.

The dumps in `resources/dumps` also stay. After a successful `formats` step the file
`resources/dumps/.gate-record` holds the commit, the sha256 of each dump file and the TeX Live identity
(its version line, the tlpdb's sha256 and the install time). `nextest` without a `formats` step earlier in
the same run is refused (exit 121) unless the record matches the present state, so a different commit,
regenerated dumps or a TL reinstall each require `formats` again. If the `formats` step fails, the script
stops, because without dumps the tests fail spuriously.

One run at a time: a second `check.sh` is refused while the first holds the worktree, and a crashed run
does not block the next. An interrupted `check.sh` (INT, TERM or HUP) stops its guest command; a
`check.sh` killed with SIGKILL does not, and the guest command then keeps the next run refused until it
finishes. A run begins by removing the old logs of that commit, so a step log that is missing afterwards
means the guest never started. The guest writes each step's log (and `prepare.log`) straight to the Mac
path, because output streamed through `orb` loses its tail when the command exits right after a burst.
`check.sh <branch> prepare` only puts the worktree on the commit, for working in it by hand with `orb -m
agents -w /home/patrick/agent-work/worktrees/latexml/linux-gate bash -l`.

The exit code is the number of failed steps; 100 means a usage or setup error (an unknown step, a stopped
or unprovisioned machine, a failed prepare or log removal, a busy gate), and `check.sh` then names `setup.sh` when the
machine lacks the toolchain or a complete TeX Live 2026 install. The guest script refuses a step with its own exit codes, 121 (no dumps for this state), 122 (worktree not
at the commit), 123 (worktree missing), 124 (the guest lock cannot be opened or taken) and 125 (log not
writable), which lie outside cargo's and nextest's. A refusal also writes a `lxgate-refused:` line to the
step log, and `check.sh` prints it.
