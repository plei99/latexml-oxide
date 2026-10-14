//! A math `'` absorbs a following superscript by meaning, as latex.ltx's `\pr@m@s` does (`\ifx^\@let@token`): `\sp`
//! (`\let\sp=^`) included. derivative.sty's `\pdv[order=2]{}{x'}` writes `x'\sp{2}`, which was a double superscript
//! (KNOWN_PERL_ERRORS #658).

#[test]
fn prime_then_sp_is_one_superscript() {
  let tex = "\\documentclass{article}\n\\usepackage[upright=true]{derivative}\n\\begin{document}\n\
             $x'\\sp{2}$\n\n$\\pdv[order=2]{}{x'}$\n\\end{document}\n";
  let (stderr, xml) = super::convert(tex, true);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains(r#"tex="x^{\prime 2}""#), "{xml}");
  assert!(
    xml.contains(r#"tex="\frac{\partial^{2}}{\partial x^{\prime 2}}""#),
    "{xml}"
  );
}
