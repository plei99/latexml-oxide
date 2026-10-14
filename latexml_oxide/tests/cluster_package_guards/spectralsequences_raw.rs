//! spectralsequences loads raw. Its Rust-only stub (an empty `{sseqdata}`/`{sseqpage}`, no-op `\printpage`/`\class`, a
//! global `\d{}{}`) dropped every diagram silently, never loaded the `backgrounds` library its .sty loads (line 37:
//! `/tikz/on background layer` unknown), and turned the kernel's `\d` underdot accent into a two-argument gobbler (a
//! label `\d["g_0", ->]2(2,0)` left `g_0` in text: "Script _ can only appear in math mode"). The raw package needs
//! `\ifpgfmathunitsdeclared` set globally, as pgf sets it: its `\pgfmathparse` runs in a group (sseqparsers.code.tex:41-49).

#[test]
fn pgfmath_units_flag_survives_a_group() {
  let tex = "\\documentclass{article}\n\\usepackage{tikz}\n\\begin{document}\n\
             \\pgfmathparse{3}\\begingroup\\pgfmathparse{2em}\\endgroup[\\ifpgfmathunitsdeclared U\\else NU\\fi]\n\
             \\end{document}\n";
  let (stderr, xml) = super::convert(tex, true);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("[U]"),
    "the flag a parse in a group sets is global (pgfmathparser.code.tex:443-526):\n{xml}"
  );
}

#[test]
fn spectral_sequence_diagram_is_drawn_and_d_stays_an_accent() {
  let tex = "\\documentclass{article}\n\\usepackage{spectralsequences}\n\\begin{document}\nBefore \\d{o} after.\n\
             \\begin{sseqdata}[name=a, homological Serre grading, y axis gap=2em]\n\\class[\"H_0(F)\"](0,0)\n\
             \\class[\"H_1(F)\"](0,1)\n\\class[\"H_0(F)\"](2,0)\n\\d[\"g_0\", ->]2(2,0)\n\\end{sseqdata}\n\
             \\printpage[name=a, page=2]\n\\end{document}\n";
  let (stderr, xml) = super::convert(tex, true);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<picture"),
    "the diagram must be drawn:\n{xml}"
  );
  assert!(
    xml.contains("Before ọ after."),
    "`\\d` outside the diagram is the underdot accent:\n{xml}"
  );
}
