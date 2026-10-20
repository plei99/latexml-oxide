//! Binding for `ebgaramond.sty` — the EB Garamond font package (NFSS: fontaxes, mweights, xkeyval options such as `scaled`, ebgaramond.sty:25).
//!
//! Perl LaTeXML ships no `ebgaramond.sty.ltxml` binding and, at the default `notex` setting, does not raw-load the
//! real file; under `--preload=latexml.sty` (no `rawstyles`) the package was skipped with `Warning:missing_file`.
//! Font choice has no effect on the LaTeXML tree. Raw-loads the genuine TL file so its options are consumed and its
//! `\RequirePackage{fontaxes,mweights}` read raw instead of `Warning:missing_file`.
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("ebgaramond", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
