//! Binding for `rank-2-roots.sty` — Dynkin/root-system diagrams for A2, B2, C2, G2 (`{rootSystem}`, `\roots`,
//! `\simpleroots`, `\weightLattice`, `\fundamentalweights`) drawn with TikZ.
//!
//! Perl LaTeXML ships no `rank-2-roots.sty.ltxml` binding and, at the default `notex` setting, does not raw-load
//! the real file; under `--preload=latexml.sty` (no `rawstyles`) the package was skipped with
//! `Warning:missing_file`. Without the file the commands are undefined. The package is expl3/xparse/xstring TikZ
//! code the engine already runs under raw loading; raw-loads the genuine TL file. The drawing is an `ltx:picture`.
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("rank-2-roots", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
