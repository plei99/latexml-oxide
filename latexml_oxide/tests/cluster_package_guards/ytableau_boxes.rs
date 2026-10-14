//! A rule inside a box in an alignment cell is that box's rule, not a border of the alignment (KNOWN_PERL_ERRORS #661):
//! an `\hrule`'s border restarted the row's columns, so the next `&` overwrote the cell. ytableau's boxes
//! (`\fcolorbox@YT`, kerned `\hrule`s and `\vrule`s in a `\vbox`) kept only a row's first and last cells. The ytableau
//! binding makes each box its cell's four borders around a `\boxdim@YT`-square inline block.

#[test]
fn a_boxed_rule_in_a_cell_keeps_the_row() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\
             \\halign{#&#&#\\cr A\\vbox{\\hrule width 10pt height 1pt}&B\\vbox{\\hrule width 10pt height 1pt}&C\\cr}\n\
             \\end{document}\n";
  let (stderr, xml) = super::convert(tex, true);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  let row = &xml[xml.find("<tr>").expect("a row")..xml.find("</tr>").expect("its end")];
  assert_eq!(row.matches("<td").count(), 3, "every cell is kept:\n{xml}");
  for cell in ["A", "B", "C"] {
    assert!(row.contains(cell), "{cell} is kept:\n{xml}");
  }
  assert_eq!(
    row.matches("<rule").count(),
    2,
    "the boxed rules are drawn, not borders:\n{xml}"
  );
}

#[test]
fn young_tableau_cells_are_bordered_boxes() {
  let tex = "\\documentclass{article}\n\\usepackage{ytableau}\n\\begin{document}\n\
             \\begin{ytableau} 1 & 2 & 4 \\\\ 3 & 5 \\end{ytableau}\n\n$\\lambda = \\ydiagram{2,1}$\n\\end{document}\n";
  let (stderr, xml) = super::convert(tex, true);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  for entry in [">1<", ">2<", ">4<", ">3<", ">5<"] {
    assert!(xml.contains(entry), "entry {entry} is kept:\n{xml}");
  }
  // 5 tableau boxes + 3 diagram boxes, each bordered on all four sides and sized as a box.
  assert_eq!(xml.matches(r#"border="b l r t""#).count(), 8, "{xml}");
  assert_eq!(
    xml.matches(r#"height="15.0pt" width="15.0pt">"#).count(),
    8,
    "{xml}"
  );
}

/// In HTML a diagram in math keeps its line and its square boxes. The raw tabular in its `m:mtext` is copied as
/// Perl's `cloneNode` copies it, its indentation whitespace kept as whitespace: as NBSP it was text in table context,
/// which the HTML parser moves out in front of the `<table>`, a line of spaces that pushed the diagram below
/// `\lambda =`. And a cell takes no paragraph indent: `text-indent` inherits, and in the first line of an indented
/// paragraph each cell's box was pushed 2em right inside a widened cell.
#[test]
fn a_diagram_in_math_renders_on_its_line() {
  let workdir = tempfile::tempdir().expect("create tempdir");
  let tex = "\\documentclass{amsart}\n\\usepackage{ytableau}\n\\begin{document}\n$\\lambda = \\ydiagram{2,1}$\n\
             \\end{document}\n";
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  let output = std::process::Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args(["t.tex", "--dest", "t.html", "--nocomments", "--timeout=110"])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  let html = std::fs::read_to_string(workdir.path().join("t.html")).expect("read t.html");
  let math =
    &html[html.find("<math").expect("the formula")..html.find("</math>").expect("its end")];
  let table = &math[math
    .find("<table")
    .expect("the diagram is a table in the formula:\n{html}")..];
  let rows = &table[..table.find("<td").expect("a cell")];
  assert!(
    !rows.contains('\u{a0}'),
    "no NBSP between the table's rows:\n{math}"
  );
  let css = std::fs::read_to_string(workdir.path().join("LaTeXML.css")).expect("read LaTeXML.css");
  assert!(
    css.contains(".ltx_tabular { text-indent:0; }"),
    "a cell takes no paragraph indent"
  );
}
