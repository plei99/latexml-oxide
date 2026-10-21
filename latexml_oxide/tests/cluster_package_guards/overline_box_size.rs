//! `\overline{x}` and `\underline{x}` are as wide as `x`. Perl's `defmath_cons` (Package.pm:1869-1873) sized every
//! DefMath with arguments by its operator glyph alone, here the 5pt macron, so `\hbox{$\overline{C}$}` was 5pt wide
//! (TeX: 7.86pt) and TikZ-cd drew the arrows of a node `\overline{C\setminus C_i}` through its label
//! (OXIDIZED_DESIGN_DIVERGENCES #610). TeX's `\overline` adds 5θ above the box and `\underline` 5θ below it
//! (tex.web §734-735, θ = 0.4pt at 10pt): `\hbox{$\overline{C}$}` is 7.86249pt x 8.8332pt + 0pt there, and
//! `\hbox{$\underline{C}$}` 7.86249pt x 6.83331pt + 1.9999pt.

#[test]
fn overline_and_underline_boxes_take_the_width_of_their_argument() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\
             \\setbox0\\hbox{$C$}C: \\the\\wd0, \\the\\ht0, \\the\\dp0.\n\
             \\setbox0\\hbox{$\\overline{C}$}over: \\the\\wd0, \\the\\ht0, \\the\\dp0.\n\
             \\setbox0\\hbox{$\\underline{C}$}under: \\the\\wd0, \\the\\ht0, \\the\\dp0.\n\
             \\end{document}\n";
  let (stderr, xml) = super::convert(tex, true);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  assert_eq!(stderr.matches("Warning:").count(), 0, "{stderr}");
  for line in [
    "C: 7.86249pt, 6.83331pt, 0.0pt.",
    "over: 7.86249pt, 8.83328pt, 0.0pt.",
    "under: 7.86249pt, 6.83331pt, 1.99997pt.",
  ] {
    assert!(xml.contains(line), "`{line}` expected:\n{xml}");
  }
}

#[test]
fn tikzcd_node_of_an_overlined_formula_is_as_wide_as_the_formula() {
  let tex = "\\documentclass{article}\n\\usepackage{tikz-cd}\n\\begin{document}\n\
             \\begin{tikzcd}\nA \\arrow{r} & \\overline{C\\setminus C_i}\n\\end{tikzcd}\n\\end{document}\n";
  let (stderr, xml) = super::convert(tex, true);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  assert_eq!(stderr.matches("Warning:").count(), 0, "{stderr}");
  assert!(
    xml.contains(
      r#"style="--ltx-fo-width:2.68em;--ltx-fo-height:0.88em;--ltx-fo-depth:0.15em;font-size:10pt;""#
    ),
    "the node's box holds the overlined formula (TeX: 26.8pt wide, 8.8pt high):\n{xml}"
  );
}
