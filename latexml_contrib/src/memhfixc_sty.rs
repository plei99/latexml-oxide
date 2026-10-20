//! Binding for `memhfixc.sty` — memoir's hyperref fix: `\@sbook` anchors (:76-83), the `\bookautorefname` strings
//! (:85-113) and a `\M@sect` wrapper that makes the section anchors (:116-140). memoir.cls requests it after
//! hyperref.
//!
//! Perl LaTeXML ships no `memhfixc.sty.ltxml` binding and, at the default `notex` setting, does not raw-load the real
//! file; under `--preload=latexml.sty` (no `rawstyles`) memoir's own request was skipped with
//! `Warning:missing_file`. Raw-loads the genuine TL file: with hyperref loaded, `\chapter`, `\section`,
//! `\subsection`, a heading below `secnumdepth`, `\section*` and a captioned float convert with no errors and the
//! same structure as under `rawstyles` (guard `native_notes_bindings::memhfixc_patches_memoir_with_hyperref`).
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("memhfixc", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
