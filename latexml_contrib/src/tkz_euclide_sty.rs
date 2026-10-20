//! Binding for `tkz-euclide.sty` — Alain Matthes's Euclidean-geometry drawing macros over TikZ (`\tkzInit`,
//! `\tkzDefPoints`, `\tkzDrawSegment`, …).
//!
//! Perl LaTeXML ships no `tkz-euclide.sty.ltxml` binding and, at the default `notex` setting, does not raw-load the
//! real file; under `--preload=latexml.sty` (no `rawstyles`) the package was skipped with `Warning:missing_file`.
//! Without the file every `\tkz…` command is undefined. The package loads tikz, xfp and xpatch (and luacode only
//! under LuaTeX, tkz-euclide.sty:160) through its own `\RequirePackage`s. Raw-loads the genuine TL file; the figure
//! is an `ltx:picture`.
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("tkz-euclide", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
