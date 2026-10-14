//! pgfkeys' sentinels stay pgf's macros (pgfkeys.code.tex:363, :367): `\ifx` compares them with macros whose body is the
//! sentinel, so as primitives every `.search also` path got `/.try` and a bare colour only `/tikz/.unknown` applies
//! (`\addplot[blue]`, a `/foo/.search also={/tikz}` scope) was dropped silently.

#[test]
fn bare_colour_in_search_also_path_is_applied() {
  let tex = "\\documentclass{article}\n\\usepackage{tikz}\n\\begin{document}\n\\pgfkeys{/foo/.search also={/tikz}}\n\
             \\begin{tikzpicture}\n\\begin{scope}[/foo/.cd,blue]\\draw (0,0)--(1,1);\\end{scope}\n\\end{tikzpicture}\n\
             \\end{document}\n";
  let (stderr, xml) = super::convert(tex, true);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(r##"stroke="#0000FF""##),
    "the bare colour reaches /tikz/.unknown:\n{xml}"
  );
}

#[test]
fn addplot_bare_colour_is_applied() {
  let tex = "\\documentclass{article}\n\\usepackage{pgfplots}\n\\pgfplotsset{compat=1.18}\n\\begin{document}\n\
             \\begin{tikzpicture}\n\\begin{axis}\\addplot[blue] {x};\\end{axis}\n\\end{tikzpicture}\n\\end{document}\n";
  let (stderr, xml) = super::convert(tex, true);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains(r##"stroke="#0000FF""##), "{xml}");
}
