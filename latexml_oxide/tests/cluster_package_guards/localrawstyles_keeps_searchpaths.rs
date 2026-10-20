//! `localrawstyles` (latexml.sty) must stay `INCLUDE_STYLES => 'searchpaths'` for the whole run.
//!
//! The raw-definitions loader (`input_definitions`'s raw branch; Perl `Package.pm:2372-2392`) saves
//! `INCLUDE_STYLES`, sets it to 1 while it reads a definitions file, and restores what it saved. It saved the
//! boolean, so the first raw definitions file read (`ts1enc.dfu`, autoloaded by LaTeX.pool from latexml.sty's
//! `\AddToHook`) turned the string `searchpaths` into `true`; `require_package`'s searchpaths_only gate
//! (Perl `Package.pm:2691`) never fired again, and TeX Live packages without a binding were raw-loaded under a
//! profile meant to load only the document's own `.sty` files raw.
//!
//! A miss under `localrawstyles` must also not be remembered as an attempted load: a local `.sty` that later
//! `\RequirePackage`s the same package runs with INCLUDE_STYLES = 1 (Perl `Package.pm:2376`) and raw-loads it.
//!
//! `nowidow` is the unbound TeX Live package (no `.sty.ltxml` in LaTeXML/, no `*_sty.rs` binding here); it defines
//! `\setnoclub` in `nowidow.sty:52`. The local `lxlocalgreeting.sty` stands in for a paper's own package.

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: Option<&str> = Some("[localrawstyles]latexml.sty");

const LOCAL_STY: &str = "\\ProvidesPackage{lxlocalgreeting}\n\
    \\newcommand{\\lxgreeting}{greetings-from-the-local-sty}\n\
    \\endinput\n";

/// A local package that itself requires the unbound TeX Live package: read raw, so what it loads is raw too.
const LOCAL_REQUIRING_STY: &str = "\\ProvidesPackage{lxlocalrequires}\n\
    \\RequirePackage{nowidow}\n\
    \\newcommand{\\lxrequires}{greetings-from-the-requiring-sty}\n\
    \\endinput\n";

/// The document's own package is loaded BEFORE the unbound TeX Live package, so the first raw read (and with it any
/// restore of INCLUDE_STYLES) happens independently of the `ts1enc.dfu` autoload.
const TEX: &str = "\\documentclass{article}\n\
    \\usepackage{lxlocalgreeting}\n\
    \\usepackage{nowidow}\n\
    \\begin{document}\n\
    \\ifdefined\\setnoclub TL:raw-loaded\\else TL:not-loaded\\fi.\n\
    \\lxgreeting\n\
    \\end{document}\n";

#[test]
fn localrawstyles_raw_loads_only_the_documents_own_sty() {
  let (log, xml) = super::convert_files_with(TEX, &[("lxlocalgreeting.sty", LOCAL_STY)], PRELOAD);
  // The unbound TeX Live package is skipped with exactly one missing-file warning, so nothing of it is defined.
  assert_eq!(
    log.matches("Warning:missing_file:nowidow").count(),
    1,
    "nowidow must be reported missing once, not raw-loaded:\n{log}"
  );
  assert!(
    log.contains("not under the document's search paths (localrawstyles)"),
    "the warning names the search-path restriction, not a disk search:\n{log}"
  );
  assert_eq!(
    (error_count(&log), warning_count(&log)),
    (0, 1),
    "the missing-file warning is the only diagnostic:\n{log}"
  );
  assert!(
    xml.contains("TL:not-loaded"),
    "\\setnoclub must stay undefined (nowidow.sty not read):\n{xml}"
  );
  assert!(
    !xml.contains("TL:raw-loaded"),
    "nowidow.sty was raw-loaded under localrawstyles:\n{xml}"
  );
  // The document's own package is read raw: its macro expands.
  assert!(
    xml.contains("greetings-from-the-local-sty"),
    "the local .sty next to the document must be loaded:\n{xml}"
  );
}

/// `\usepackage{nowidow}` misses (one warning); the local package then requires it from inside its raw read, where
/// INCLUDE_STYLES is 1 (Perl `Package.pm:2376`), and it loads from the TeX tree. The miss must not have been
/// remembered as an attempted load (`already_handled` would skip the second request).
#[test]
fn a_local_sty_raw_loads_what_the_document_could_not() {
  let tex = "\\documentclass{article}\n\
    \\usepackage{nowidow}\n\
    \\usepackage{lxlocalrequires}\n\
    \\begin{document}\n\
    \\ifdefined\\setnoclub TL:raw-loaded\\else TL:not-loaded\\fi.\n\
    \\lxrequires\n\
    \\end{document}\n";
  let (log, xml) = super::convert_files_with(
    tex,
    &[("lxlocalrequires.sty", LOCAL_REQUIRING_STY)],
    PRELOAD,
  );
  assert_eq!(
    log.matches("Warning:missing_file:nowidow").count(),
    1,
    "only the document's own request misses; the local .sty's request loads:\n{log}"
  );
  assert_eq!(
    (error_count(&log), warning_count(&log)),
    (0, 1),
    "the missing-file warning is the only diagnostic:\n{log}"
  );
  assert!(
    xml.contains("TL:raw-loaded"),
    "\\setnoclub must be defined by the local .sty's \\RequirePackage{{nowidow}}:\n{xml}"
  );
  assert!(
    xml.contains("greetings-from-the-requiring-sty"),
    "the local .sty itself loads:\n{xml}"
  );
}

/// A source registered in `INTERPRETABLE_SOURCES` (physics2.sty, its `phy-*` modules, fontawesome7.sty:
/// `latex_constructs_rust_only.rs` §10) is vouched for as safe to interpret, so it raw-loads from the TeX tree under
/// `localrawstyles` as it does by default (`require_package`'s notex default exempts it; the searchpaths_only gate
/// must too). Without the exemption `\usepackage{physics2}` was skipped as "not under the document's search paths"
/// and `\usephysicsmodule`, `\ab`, `\pdiagmat` were undefined, with `Error:unexpected:\middle` after them.
#[test]
fn localrawstyles_still_raw_loads_an_interpretable_source() {
  let tex = "\\documentclass{article}\n\
    \\usepackage{physics2}\n\
    \\usephysicsmodule{ab,diagmat}\n\
    \\begin{document}\n\
    $\\ab(x)$ and $\\pdiagmat{a,b}$\n\
    \\end{document}\n";
  let (log, xml) = super::convert_files_with(tex, &[], PRELOAD);
  assert_eq!(
    (error_count(&log), warning_count(&log)),
    (0, 0),
    "physics2 loads without a diagnostic:\n{log}"
  );
  assert!(
    !log.contains("missing_file"),
    "physics2 is not reported missing:\n{log}"
  );
  assert!(
    xml.contains(r#"<XMTok role="OPEN" stretchy="true">(</XMTok>"#)
      && xml.contains(r#"<XMTok role="CLOSE" stretchy="true">)</XMTok>"#),
    "\\ab(x) makes stretchy fences around x:\n{xml}"
  );
  // `\pdiagmat{a,b}` is the 2x2 diagonal matrix in parentheses: the pmatrix source, its array semantics, and two
  // rows of two cells.
  assert!(
    xml.contains(r#"tex="\begin{pmatrix}a&amp;0\\&#10;0&amp;b\\&#10;\end{pmatrix}""#)
      && xml.contains(r#"text="matrix@(Array[[a, 0], [0, b]])""#),
    "\\pdiagmat builds the parenthesised diagonal matrix:\n{xml}"
  );
  assert_eq!(
    (
      xml.matches("<XMRow>").count(),
      xml.matches("<XMCell ").count()
    ),
    (2, 4),
    "the matrix has two rows of two cells:\n{xml}"
  );
}

/// A binding that requests an unbound TeX Live package "only where raw loading is on" must not make the request under
/// `localrawstyles`: INCLUDE_STYLES is then the string `searchpaths`, which is truthy, but the request can only miss
/// (the package is in the TeX tree, outside the document's search paths) and adds a `missing_file` warning for a
/// package nothing asked for. `mdframed_sty.rs` requests zref-abspage (mdframed.sty:72-77), `amsrefs_sty.rs`
/// rkeyval and textcmds; `raw_tex_styles_enabled` is the gate.
#[test]
fn bindings_do_not_request_unbound_packages_under_localrawstyles() {
  let tex = "\\documentclass{article}\n\
    \\usepackage{mdframed}\n\
    \\usepackage{amsrefs}\n\
    \\usepackage{backref}\n\
    \\begin{document}\n\
    \\begin{mdframed}framed-text\\end{mdframed}\n\
    \\end{document}\n";
  let (log, xml) = super::convert_files_with(tex, &[], PRELOAD);
  for missing in ["zref-abspage", "rkeyval", "textcmds", "rerunfilecheck"] {
    assert!(
      !log.contains(&format!("missing_file:{missing}")),
      "{missing} must not be requested under localrawstyles:\n{log}"
    );
  }
  for stub in [
    "Warning:missing_file:mdframed.sty mdframed.sty is only minimally stubbed",
    "Warning:missing_file:backref.sty backref.sty is minimally stubbed",
  ] {
    assert_eq!(log.matches(stub).count(), 1, "{stub} once:\n{log}");
  }
  assert_eq!(
    (error_count(&log), warning_count(&log)),
    (0, 2),
    "the two bindings' own stub warnings are the only diagnostics:\n{log}"
  );
  assert!(
    xml.contains("framed-text"),
    "the framed text is present:\n{xml}"
  );
}

/// The gate is not applied to a paper's own `arxiv.sty` (`arxiv_sty.rs`, `primearxiv_sty.rs`): that file is in the
/// document's directory, so it is found under `localrawstyles` and stays authoritative over the bare fallbacks.
#[test]
fn a_local_arxiv_sty_still_loads_under_localrawstyles() {
  let tex = "\\documentclass{article}\n\
    \\usepackage{arxiv}\n\
    \\begin{document}\n\
    \\lxarxiv\n\
    \\end{document}\n";
  let sty =
    "\\ProvidesPackage{arxiv}\n\\newcommand{\\lxarxiv}{from-the-local-arxiv-sty}\n\\endinput\n";
  let (log, xml) = super::convert_files_with(tex, &[("arxiv.sty", sty)], PRELOAD);
  assert_eq!(
    (error_count(&log), warning_count(&log)),
    (0, 0),
    "no diagnostic at all:\n{log}"
  );
  assert!(
    xml.contains("from-the-local-arxiv-sty"),
    "the paper's own arxiv.sty is read:\n{xml}"
  );
}
