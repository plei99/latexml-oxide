//! Under the `xetex` profile, XeTeX's line-breaking parameters exist (xetex.web): `\XeTeXlinebreaklocale` reads a file
//! name, `\XeTeXlinebreakskip` is a glue and `\XeTeXlinebreakpenalty` an integer parameter. Chinese documents set them
//! after `\usepackage{xeCJK}` and got "undefined" errors.

#[test]
fn xetex_linebreak_parameters_are_defined() {
  let tex = "\\documentclass{article}\n\\XeTeXlinebreaklocale \"zh\"\n\\XeTeXlinebreakskip = 0pt plus 1pt\n\
             \\XeTeXlinebreakpenalty=50\n\\begin{document}\nText \\the\\XeTeXlinebreakpenalty.\n\\end{document}\n";
  let (stderr, xml) = super::convert_with(tex, Some("[xetex,rawstyles,rawclasses]latexml.sty"));
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>Text 50.</p>"), "{xml}");
}
