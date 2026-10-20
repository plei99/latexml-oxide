//! Binding for `spectralsequences.sty` — Hood Chatham's spectral sequence diagrams (`{sseqdata}`, `{sseqpage}`,
//! `\class`, `\d`, `\printpage`, `\sseqset`) drawn with TikZ.
//!
//! Perl LaTeXML ships no `spectralsequences.sty.ltxml` binding and, at the default `notex` setting, does not
//! raw-load the real file; under `--preload=latexml.sty` (no `rawstyles`) the package was skipped with
//! `Warning:missing_file`. Without the file the environments are undefined and a `\class["H_0(F)"](0,0)` label
//! leaves `_` in text mode. The raw package loads tikz, etoolbox, xparse and its `backgrounds` library through its
//! own `\RequirePackage`s, which read raw while the wrapped file is read (`INCLUDE_STYLES`, content.rs); its
//! `\pgfmathparse` needs `\ifpgfmathunitsdeclared` set globally, as pgfmath does. Raw-loads the genuine TL file;
//! the diagram is an `ltx:picture`.
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("spectralsequences", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
