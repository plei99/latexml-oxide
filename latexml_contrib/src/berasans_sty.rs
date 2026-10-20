//! Binding for `berasans.sty` — Bera Sans (Bitstream Vera Sans) as the sans-serif family (NFSS), with a
//! `scaled=<factor>` keyval option (berasans.sty:3-4).
//!
//! Perl LaTeXML ships no `berasans.sty.ltxml` binding and, at the default `notex` setting, does not raw-load the
//! real file; under `--preload=latexml.sty` (no `rawstyles`) the package was skipped with `Warning:missing_file`.
//! Font choice has no effect on the LaTeXML tree; the binding exists so the package is recognised and its options
//! reach the real file instead of a `Warning:missing_file`. Raw-loads the genuine TL file like `alegreyasans_sty`.
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("berasans", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
