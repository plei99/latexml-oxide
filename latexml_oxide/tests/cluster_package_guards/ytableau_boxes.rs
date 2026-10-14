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
  assert_eq!(xml.matches(r#"height="15.0pt" width="15.0pt">"#).count(), 8, "{xml}");
}
