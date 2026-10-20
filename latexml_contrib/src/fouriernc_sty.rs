//! Binding for `fouriernc.sty` — Fourier math with New Century Schoolbook text; loads fourier.sty.
//!
//! Perl LaTeXML ships no `fouriernc.sty.ltxml` binding and, at the default `notex` setting, does not raw-load the
//! real file; under `--preload=latexml.sty` (no `rawstyles`) the package was skipped with `Warning:missing_file`.
//! Font choice has no effect on the LaTeXML tree; the package `\RequirePackage{fourier}` reaches this crate's
//! fourier binding. Raw-loads the genuine TL file.
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("fouriernc", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
