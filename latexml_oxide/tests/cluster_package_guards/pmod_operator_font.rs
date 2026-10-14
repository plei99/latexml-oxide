//! `\pmod` sets `mod` in `\operator@font`, as latex.ltx:15560 does (KNOWN_PERL_ERRORS #657): plain.tex's `{\rm mod}`
//! met memoir's error-raising `\rm` (memoir.cls:6903, without `oldfontcommands`), one "Font command \rm is not
//! supported" per `\pmod`, and the `mod` came out as italic letters.

#[test]
fn pmod_under_memoir_is_an_upright_operator_without_errors() {
  let tex =
    "\\documentclass{memoir}\n\\begin{document}\n$a \\equiv b \\pmod{2}$\n\\end{document}\n";
  let (stderr, xml) = super::convert(tex, true);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(r#"<XMTok role="BIGOP" scriptpos="post">mod</XMTok>"#),
    "`mod` must be one upright operator:\n{xml}"
  );
  assert!(
    xml.contains(r#"text="a equivalent-to annotated@(b, pmod@(2))""#),
    "{xml}"
  );
}
