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

/// A class is a circle of `minimum size=0.35em` with `inner sep=0pt` (spectralsequences.sty:332 `classes={draw,circle,
/// inner sep=0pt,minimum size=0.35em}`). The package stores those keys by redefining `\pgfkeys@case@one` and
/// `\pgfkeys@case@two@extern` around its option lists (sseqkeys.code.tex:260-261, 307-325); the native pgfkeys loop,
/// which never calls them, ran the keys at once and dropped them, so the dot kept TikZ's default `inner sep=.3333em`
/// (radius 4.71pt = 6.52 in picture units, three times the 1.75pt = 2.42 TeX draws: the notes' spectral-sequence dots).
#[test]
fn class_dot_has_its_declared_size() {
  let tex = "\\documentclass{article}\n\\usepackage{spectralsequences}\n\\begin{document}\n\
             \\begin{sseqdata}[classes={fill,draw=none}, name=a, homological Serre grading]\n\\class(0,0)\n\\end{sseqdata}\n\
             \\printpage[name=a, page=2]\n\\end{document}\n";
  let (stderr, xml) = super::convert(tex, true);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(
      r#"<svg:path d="M 2.42 0 C 2.42 1.34 1.34 2.42 0 2.42 C -1.34 2.42 -2.42 1.34 -2.42 0 C -2.42 -1.34 -1.34 -2.42 0 -2.42 C 1.34 -2.42 2.42 -1.34 2.42 0 Z M 0 0" style="stroke:none"/>"#
    ),
    "the class is a filled circle of radius 1.75pt:\n{xml}"
  );
  assert_eq!(stderr.matches("Warning:").count(), 0, "{stderr}");
}

/// A package that swaps `\pgfkeys@case@one` / `\pgfkeys@case@two@extern` for the length of a key list (spectralsequences)
/// is obeyed: the native loop stands aside while either differs from the raw file's definition (pgfkeys.code.tex:422-437).
#[test]
fn redefined_case_one_step_is_called_by_the_key_loop() {
  let tex = "\\documentclass{article}\n\\usepackage{tikz}\n\\makeatletter\n\\newcount\\casecalls\n\
             \\let\\origcaseone\\pgfkeys@case@one\n\\def\\pgfkeys@case@one{\\global\\advance\\casecalls\\@ne\\origcaseone}\n\
             \\makeatother\n\\begin{document}\n\\pgfkeys{/foo/bar/.code=\\relax}\\pgfkeys{/foo/bar}\n\
             calls=\\ifnum\\casecalls>0 some\\else none\\fi.\n\\end{document}\n";
  let (stderr, xml) = super::convert(tex, true);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("calls=some."),
    "the redefined step ran:\n{xml}"
  );
  assert_eq!(stderr.matches("Warning:").count(), 0, "{stderr}");
}
