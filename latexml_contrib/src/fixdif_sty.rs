//! Binding for `fixdif.sty` — Zhang Tingxuan's differential operators: `\d` becomes the roman, kerned differential
//! `d` in math (`\mathinner{\mathrm{d}}`, `[rm|normal]`, `[partial|nopartial]`), `\letdif` declares more,
//! and text-mode `\d` stays LaTeX's dot-under accent (fixdif.sty:68-69, 84).
//!
//! Perl LaTeXML ships no `fixdif.sty.ltxml` binding and, at the default `notex` setting, does not raw-load the real
//! file; under `--preload=latexml.sty` (no `rawstyles`) the package was skipped with `Warning:missing_file`.
//! Without the file `\d{x_i}` in math is the text accent (`Error:unexpected:_ Script _ can only appear in math
//! mode`), and `\d\ab(x)` hands physics2's `\ab` to the accent alone, whose dispatcher (phy-ab.sty:74) loops to
//! `Fatal:TooManyErrors`. The package is plain `\def`/`\DeclareRobustCommand` TeX that the engine runs correctly
//! under raw loading, so the faithful binding force-raw-loads the real `.sty` instead of reimplementing it;
//! `noltxml` bypasses the "raw loading is off" gate and the `\usepackage` options reach the file through
//! `InputDefinitions`. Reading a host texmf `.sty` via kpathsea is in scope (CLAUDE.md Guiding Principles).
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("fixdif", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
