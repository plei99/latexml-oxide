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

/// `t.tex` converted to `t.html` in a fresh directory: (stderr, HTML, the directory).
fn convert_html(tex: &str) -> (String, String, tempfile::TempDir) {
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  let output = std::process::Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args(["t.tex", "--dest", "t.html", "--nocomments", "--timeout=110"])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  let html = std::fs::read_to_string(workdir.path().join("t.html")).expect("read t.html");
  (stderr, html, workdir)
}

/// In HTML a diagram in math keeps its line and its square boxes. The raw tabular in its `m:mtext` keeps its
/// whitespace-only text as whitespace, as Perl's `cloneNode` does: as NBSP it was text in table context, which the
/// HTML parser moves out in front of the `<table>`, a line of spaces that pushed the diagram below `\lambda =`. And a
/// cell takes no paragraph indent: `text-indent` inherits, and in the first line of an indented paragraph each cell's
/// box was pushed 2em right inside a widened cell.
#[test]
fn a_diagram_in_math_renders_on_its_line() {
  let (stderr, html, workdir) = convert_html(
    "\\documentclass{amsart}\n\\usepackage{ytableau}\n\\begin{document}\n$\\lambda = \\ydiagram{2,1}$\n\
     \\end{document}\n",
  );
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  let math = &html[html
    .find("<math")
    .unwrap_or_else(|| panic!("the formula:\n{html}"))..];
  let math = &math[..math.find("</math>").expect("its end")];
  let table = &math[math
    .find("<table")
    .unwrap_or_else(|| panic!("the diagram is a table in the formula:\n{math}"))..];
  let table = &table[..table.find("</table>").expect("its end")];
  // Every run between the table's structural tags — before a `<tr>`, `<td>`, `</tr>` or `</table>` — is whitespace.
  let structural = regex::Regex::new(
    r"(?s)<(?:table|tbody|tr|/tr|/td)\b[^>]*>([^<]*)<(?:tbody|tr|td|/tr|/tbody)\b",
  )
  .unwrap();
  let mut runs = 0;
  for run in structural.captures_iter(table) {
    runs += 1;
    assert!(
      run[1].chars().all(|c| c.is_ascii_whitespace()),
      "no NBSP between the table's rows:\n{math}"
    );
  }
  assert!(runs >= 4, "the table's structure is found:\n{math}");
  let css = std::fs::read_to_string(workdir.path().join("LaTeXML.css")).expect("read LaTeXML.css");
  assert!(
    css.contains(".ltx_tabular { text-indent:0; }"),
    "a cell takes no paragraph indent"
  );
}

/// Other text in raw markup in math keeps its edge spaces as NBSP, as `\fbox{ q }` keeps its glue in TeX: plain
/// spaces at the edge of the frame's inline block would collapse, the frame hugging the `q`.
#[test]
fn a_framed_box_in_math_keeps_its_spaces() {
  let (stderr, html, _workdir) = convert_html(
    "\\documentclass{article}\n\\begin{document}\n$a+\\mbox{\\fbox{ q } w}$\n\\end{document}\n",
  );
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  assert!(
    html.contains("\u{a0}q\u{a0}"),
    "the frame keeps its spaces:\n{html}"
  );
}

/// A diagram is followed by nothing: ytableau's overlay kern, as wide as the diagram it zeroes, doubled the space
/// after it, a box keeping its own width in HTML.
#[test]
fn a_diagram_leaves_no_space_after_it() {
  let tex = "\\documentclass{article}\n\\usepackage{ytableau}\n\\begin{document}\n\
             $\\ydiagram{3,2}$X\\begin{ytableau} 1 & 2 \\end{ytableau}Y\n\\end{document}\n";
  let (stderr, xml) = super::convert(tex, true);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  assert!(
    !xml.contains(['\u{2002}', '\u{2003}', '\u{2009}']),
    "no kern's spaces:\n{xml}"
  );
  assert_eq!(
    xml.matches("<tabular").count(),
    2,
    "both diagrams are drawn:\n{xml}"
  );
}
