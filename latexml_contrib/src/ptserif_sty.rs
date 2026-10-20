//! Binding for `PTSerif.sty` (paratype) — PT Serif as the serif font (NFSS, a `scaled=<factor>` keyval option, PTSerif.sty:8).
//!
//! Perl LaTeXML ships no `PTSerif.sty.ltxml` binding and, at the default `notex` setting, does not raw-load the
//! real file; under `--preload=latexml.sty` (no `rawstyles`) the package was skipped with `Warning:missing_file`.
//! Font choice has no effect on the LaTeXML tree. Raw-loads the genuine TL file like `alegreyasans_sty`.
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("PTSerif", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
