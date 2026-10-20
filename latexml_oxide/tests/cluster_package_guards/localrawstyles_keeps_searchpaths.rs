//! `localrawstyles` (latexml.sty) must stay `INCLUDE_STYLES => 'searchpaths'` for the whole run.
//!
//! The raw-definitions loader (`input_definitions`'s raw branch; Perl `Package.pm:2372-2392`) saves
//! `INCLUDE_STYLES`, sets it to 1 while it reads a definitions file, and restores what it saved. It saved the
//! boolean, so the first raw definitions file read (`ts1enc.dfu`, autoloaded by LaTeX.pool from latexml.sty's
//! `\AddToHook`) turned the string `searchpaths` into `true`; `require_package`'s searchpaths_only gate
//! (Perl `Package.pm:2691`) never fired again, and TeX Live packages without a binding were raw-loaded under a
//! profile meant to load only the document's own `.sty` files raw.
//!
//! `nowidow` is the unbound TeX Live package (no `.sty.ltxml` in LaTeXML/, no `*_sty.rs` binding here); it defines
//! `\setnoclub` in `nowidow.sty:52`. The local `lxlocalgreeting.sty` stands in for a paper's own package.

const LOCAL_STY: &str = "\\ProvidesPackage{lxlocalgreeting}\n\
    \\newcommand{\\lxgreeting}{greetings-from-the-local-sty}\n\
    \\endinput\n";

const TEX: &str = "\\documentclass{article}\n\
    \\usepackage{nowidow}\n\
    \\usepackage{lxlocalgreeting}\n\
    \\begin{document}\n\
    \\ifdefined\\setnoclub TL:raw-loaded\\else TL:not-loaded\\fi.\n\
    \\lxgreeting\n\
    \\end{document}\n";

#[test]
fn localrawstyles_raw_loads_only_the_documents_own_sty() {
  let (log, xml) = super::convert_files_with(
    TEX,
    &[("lxlocalgreeting.sty", LOCAL_STY)],
    Some("[localrawstyles]latexml.sty"),
  );
  // The unbound TeX Live package is skipped with exactly one missing-file warning, so nothing of it is defined.
  assert_eq!(
    log.matches("Warning:missing_file:nowidow").count(),
    1,
    "nowidow must be reported missing once, not raw-loaded:\n{log}"
  );
  assert_eq!(super::error_count(&log), 0, "no errors expected:\n{log}");
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
