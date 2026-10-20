//! Binding for `memhfixc.sty` — memoir's hyperref fix: pdfTeX hyperref's anchor internals patched into memoir's
//! `\@sbook` (memhfixc.sty:76-83), its `\M@sect` (:116-140), `\bookautorefname` (:85-113) and `\cftaddtitleline`
//! (:143). memoir.cls requests it after hyperref.
//!
//! Perl LaTeXML ships no `memhfixc.sty.ltxml` binding and, at the default `notex` setting, does not raw-load the real
//! file; under `--preload=latexml.sty` (no `rawstyles`) memoir's own request was skipped with
//! `Warning:missing_file`. A deliberate difference from `rawstyles`: the binding defines nothing. The file's
//! patches call `\Hy@MakeCurrentHrefAuto`, `\Hy@SectionHShift` and `\Hy@SectionAnchorHref`, hyperref internals that
//! pdfTeX's hyperref defines and LaTeXML's hyperref binding does not (a raw load turns a `\book*` into
//! `Error:undefined:\Hy@MakeCurrentHrefAuto`), and the macros it patches (`\M@sect`) are not the ones the native
//! sectioning runs (`\@startsection`, sect04.rs). LaTeXML's own memoir and hyperref bindings make the anchors and ids.
//! OXIDIZED_DESIGN #593; guard
//! `native_notes_bindings::memhfixc_is_a_no_op_for_memoir_with_hyperref`.
use latexml_package::prelude::*;

LoadDefinitions!({});
