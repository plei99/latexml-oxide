# Agent notes for this fork

`CLAUDE.md`, the root `AGENTS.md`, `.claude/` and `.agents/` are upstream's, written for its arXiv work on its
own Linux host. They apply here too, with the differences below. Keep fork matters in `fork/`: an edit to
upstream's instruction files conflicts on every upstream merge. Codex and other harnesses do not load this file
by themselves, so a brief for one of them says to read it first.

## Paths that differ from upstream's docs

- **Perl LaTeXML source:** `LaTeXML/` links to `.upstream/`, a source checkout of brucemiller/latexml master.
  Upstream's `LaTeXML/blib/lib/LaTeXML/…` paths are `LaTeXML/lib/LaTeXML/…` here. Cite Perl lines from this
  tree only. Homebrew's LaTeXML is 0.8.8, older than what oxide mirrors (it lacks `TeX_Box.pool.ltxml`, for
  example), so it supplies Perl modules and nothing else.
- **Running Perl LaTeXML:**
  `PATH=/usr/local/texlive/2026/bin/universal-darwin:$PATH PERL5LIB=/opt/homebrew/opt/latexml/libexec/lib/perl5 perl -I<repo>/LaTeXML/lib <repo>/LaTeXML/bin/latexml --nocomments --dest=x.xml x.tex`
- **TeX Live 2026** (scheme-full) is at `/usr/local/texlive/2026`; `/Library/TeX/texbin` resolves to it as well.
  It is the fork's only target: guards expect what TeX Live 2026's packages produce, and nothing here keeps
  upstream CI's TeX Live 2023 passing. When a guard and an older TeX Live disagree, the guard follows 2026.
- **Absent here:** `~/data` (and so `payload-writer`'s `~/data/pk_wt/` worktrees), `/home/data`, cortex and the
  canvas harness. `/tmp` is on disk on macOS, not in RAM.

## Converting a document

- `fork/oxide doc.tex [outdir]` converts any local document under the corpus protocol and prints the verdict.
  Do not run the binary bare on a real document: without the protocol it reports hundreds of spurious errors.
- `fork/binary_info.sh [bin]` names a build and says STALE when the tree is newer. `cargo test` does not
  rebuild `target/release/latexml_oxide`; `cargo build --release --bin latexml_oxide` before a corpus check.
- Regression checks over the private corpus and the public fixtures: `fork/README.md`.

## Parallel agents

- One writer per worktree: `fork/worktree.sh <branch>`, and the brief names that path. Never put two writers in
  one checkout. Integrate in the main checkout and run the bundle's gates once there.
- Analysis agents are read-only and use the main checkout, where `LaTeXML/` and `.corpus/` are.
- Background commands and delegated tasks report when they finish. Do not poll them with `sleep` or by reading
  their output files.

## Guards: what reviews kept sending back

- Assert warnings as well as errors. Pin an expected warning by name and count.
- Assert what the fix produces (an element or a count only the fix yields), not only the bug's absence.
- Cite `file:line` from the tree actually read (`LaTeXML/` here, TeX files via `kpsewhich`), and recheck the
  citations before committing.
- Doc and CHANGELOG text says what the code does and no more.
- Compare before and after with binaries named by `binary_info.sh`, not a path that a build may replace.

## Pushing and upstream PRs

- `.githooks/pre-push` (`tools/lint.sh`) passes on macOS. Push without `--no-verify`.
- `fork/linux/check.sh <branch>` runs the gates on Linux with TeX Live 2026. Upstream's own CI runs TeX Live
  2023, so an upstream PR is judged there, not here.
- Upstream is `dginev/latexml-oxide`, branch `perfect_kernel`. A PR branch starts from `upstream/perfect_kernel`
  and holds one fix. Match the maintainer's closed PRs: a `fix: …` or `fix #N: …` title, a body of
  **Diagnostic.**, **Approach.** and **Validation.** paragraphs, a CHANGELOG `## Unreleased` entry and
  `Closes #N`. Check a recent closed PR for drift before writing one.
- Pushing and opening PRs need the user's go-ahead each time.

## Rules and what enforces them

The `correct` skill keeps this table: a mistake agents repeat gets the strongest check that works, and a rule
leaves the table once its mistake can no longer happen.

| Rule | Enforced by |
|---|---|
| Perl citations come from the source oxide mirrors | `LaTeXML/` link (structure) |
| A corpus run uses one binary throughout | `run.sh` copies the binary and writes `BINARY` (structure) |
| A binary under test contains the tree | `binary_info.sh` STALE (report only) |
| macOS lint is clean before a push | `.githooks/pre-push` (blocks) |
| Writers never share a checkout | `fork/worktree.sh` (convention: the brief must use it) |
| Local conversions use the corpus protocol | `fork/oxide` (convention) |
| Guards assert the fix's output and pin warnings | `reviewer` agent, `principle-test-behavior-not-implementation` (judgment) |
