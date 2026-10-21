//! A paragraph is set with the `\leftskip`/`\rightskip` in force when it ENDS (tex.web §1096 `end_graf`
//! breaks the lines at `\par`), so `\raggedleft`, `\raggedright` and `\centering` shape the paragraphs a
//! group finishes, not a paragraph that is still open when the group closes. Perl stamps every element
//! the group added, the open paragraph included (latex_constructs.pool.ltxml:1232-1241
//! `applyAligningContext`). `\vspace` in horizontal mode is a `\vadjust` (latex.ltx:9522-9532
//! `\@vspace`) and does not end the paragraph either, where `\vskip` does.
//! OXIDIZED_DESIGN_DIVERGENCES #602 and #603.

fn xml_of(body: &str) -> String {
  let (stderr, xml) = super::convert(
    &format!("\\documentclass{{article}}\n\\begin{{document}}\n{body}\n\\end{{document}}\n"),
    false,
  );
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  assert_eq!(
    super::perfect_kernel_batch46::warning_count(&stderr),
    0,
    "{stderr}"
  );
  xml
}

/// The `<para>` elements of the body, each from its opening tag to its closing one.
fn paras(xml: &str) -> Vec<&str> {
  let mut out = Vec::new();
  let mut rest = xml;
  while let Some(start) = rest.find("<para ") {
    let end = rest[start..].find("</para>").expect("closed para") + "</para>".len();
    out.push(&rest[start..start + end]);
    rest = &rest[start + end..];
  }
  out
}

/// A title page set in a group that ends with a line of text and a `\vspace*`, as classes and
/// documents write them: the closing line is the paragraph of the text after the group, which
/// the group's `\raggedleft` no longer shapes.
#[test]
fn raggedleft_group_aligns_only_the_paragraphs_it_finishes() {
  let xml = xml_of(
    "\\begingroup\n\
     \\raggedleft\n\
     \\vspace*{\\baselineskip}\n\
     {\\Huge\\itshape Title \\\\ Term}\\\\[\\baselineskip]\n\
     {\\Large Lectures by Y}\\par\n\
     \\vfill\n\
     {\\Large \\sffamily Some University}\n\
     \\vspace*{\\baselineskip}\n\
     \\endgroup\n\n\
     After.\n",
  );
  let paras = paras(&xml);
  assert_eq!(
    paras,
    [
      "<para class=\"ltx_align_right\" xml:id=\"p1\">\n    <p><text font=\"italic\" fontsize=\"298%\">Title <break/>Term<break cssstyle=\"--ltx-break-space:12.0pt\"/></text>\n<text fontsize=\"144%\">Lectures by Y</text></p>\n  </para>",
      "<para xml:id=\"p2\">\n    <p><text font=\"sansserif\" fontsize=\"144%\">Some University</text></p>\n  </para>",
      "<para xml:id=\"p3\">\n    <p>After.</p>\n  </para>",
    ],
    "{xml}"
  );
}

/// The same with a `\par` last: the group then finishes both paragraphs.
#[test]
fn raggedleft_group_with_a_final_par_aligns_every_paragraph() {
  let xml = xml_of("\\begingroup\\raggedleft Q1 first\\par Q2 second\\par\\endgroup\n\nQ3 third\n");
  assert_eq!(
    paras(&xml),
    [
      "<para class=\"ltx_align_right\" xml:id=\"p1\">\n    <p>Q1 first</p>\n  </para>",
      "<para class=\"ltx_align_right\" xml:id=\"p2\">\n    <p>Q2 second</p>\n  </para>",
      "<para xml:id=\"p3\">\n    <p>Q3 third</p>\n  </para>",
    ],
    "{xml}"
  );
}

/// A declaration with no `\par` before the closing brace leaves the paragraph in the enclosing shape.
#[test]
fn centering_braced_without_par_leaves_the_paragraph_alone() {
  let xml =
    xml_of("{\\centering\\par B1 spans} the rest of B1.\n\n{\\centering\\par B2 alone\\par}\n");
  assert_eq!(
    paras(&xml),
    [
      "<para xml:id=\"p1\">\n    <p>B1 spans the rest of B1.</p>\n  </para>",
      "<para align=\"center\" xml:id=\"p2\">\n    <p>B2 alone</p>\n  </para>",
    ],
    "{xml}"
  );
}

/// Inside a declaration `\\` is `\@centercr`, which ends the paragraph (latex.ltx:15619-15626): a group whose open
/// paragraph holds a `\\` has finished its lines inside the group, and they are aligned.
#[test]
fn newline_in_a_declaration_ends_the_paragraph_inside_the_group() {
  let xml = xml_of("{\\centering A\\\\ B\\\\}\n\nC\n");
  assert_eq!(
    paras(&xml),
    [
      "<para align=\"center\" xml:id=\"p1\">\n    <p>A<break/>B<break/></p>\n  </para>",
      "<para xml:id=\"p2\">\n    <p>C</p>\n  </para>",
    ],
    "{xml}"
  );
}

/// The environments and boxes whose paragraph ends inside their group (LaTeX puts a `\par` before
/// their `\egroup`) keep the alignment of their text: no regression from the open-paragraph rule.
#[test]
fn boxes_and_environments_still_align_their_last_paragraph() {
  let (stderr, xml) = super::convert(
    "\\documentclass{article}\n\\usepackage{amsthm,array}\n\\newtheorem{thm}{Theorem}\n\\begin{document}\n\
     \\begin{minipage}{5cm}\\centering E minipage text\\end{minipage}\n\n\
     F \\parbox{4cm}{\\raggedleft parbox text} then.\n\n\
     \\begin{quote}\\raggedleft I quote text\\end{quote}\n\n\
     \\begin{thm}\\raggedleft H theorem text\\end{thm}\n\n\
     \\begin{itemize}\\raggedright\\item G item text\\end{itemize}\n\n\
     \\begin{tabular}{>{\\centering\\arraybackslash}p{3cm}}T cell text\\end{tabular}\n\n\
     \\begin{center}D center env\\end{center}\n\\end{document}\n",
    false,
  );
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  assert_eq!(
    super::perfect_kernel_batch46::warning_count(&stderr),
    0,
    "{stderr}"
  );
  for element in [
    "<p align=\"center\">E minipage text</p>",
    "<p class=\"ltx_align_right\">parbox text</p>",
    "<quote>\n      <p class=\"ltx_align_right\">I quote text</p>\n    </quote>",
    "<para class=\"ltx_align_right\" xml:id=\"Thmthm1.p1\">\n      <p><text font=\"italic\">H theorem text</text></p>\n    </para>",
    "<item class=\"ltx_align_left\" xml:id=\"S0.I1.i1\">",
    "<td align=\"left\" vattach=\"top\"><inline-block vattach=\"top\" width=\"85.4pt\">\n              <p align=\"center\">T cell text</p>\n            </inline-block></td>",
  ] {
    assert_eq!(xml.matches(element).count(), 1, "{element}\n{xml}");
  }
  assert_eq!(
    xml
      .matches("<para xml:id=\"p6\">\n    <p align=\"center\">D center env</p>\n  </para>")
      .count(),
    1,
    "{xml}"
  );
}

/// `\vspace` between two words of one paragraph is a `\vadjust`: one paragraph, as in the PDF.
#[test]
fn vspace_in_a_paragraph_does_not_split_it() {
  let xml = xml_of("baz \\vspace{5pt} qux \\vspace*{1ex} quux.\n\n\\vspace{5pt}\n\nnext\n");
  assert_eq!(
    paras(&xml),
    [
      "<para xml:id=\"p1\">\n    <p>baz qux quux.</p>\n  </para>",
      "<para xml:id=\"p2\">\n    <p>next</p>\n  </para>",
    ],
    "{xml}"
  );
}

/// `\vspace` in an alignment row or a formula inside a paragraph: TeX sets its `\vadjust` material after the row or the
/// line, and the paragraph around the display goes on. Two paragraphs here would split before the equation.
#[test]
fn vspace_in_a_row_or_a_formula_leaves_the_paragraph_whole() {
  let (stderr, xml) = super::convert(
    "\\documentclass{article}\n\\usepackage{amsmath}\n\\begin{document}\nWe have\n\
     \\begin{align} a&=b \\\\ \\vspace{2mm} c&=d \\end{align}\nand therefore\n\
     \\begin{equation} e=f \\end{equation}\nwhich ends the argument.\n\n\
     Lead \\begin{tabular}{ll}a & \\vspace{5mm} b\\\\ c & d\\end{tabular}\nmiddle $a\\vspace{1mm}b$ text\n\
     \\[ x \\]\nafter display.\n\nNext.\n\\end{document}\n",
    false,
  );
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  assert_eq!(
    super::perfect_kernel_batch46::warning_count(&stderr),
    0,
    "{stderr}"
  );
  let ids: Vec<&str> = paras(&xml)
    .iter()
    .map(|p| {
      p.split("xml:id=\"")
        .nth(1)
        .and_then(|r| r.split('"').next())
        .unwrap()
    })
    .collect();
  assert_eq!(ids, ["p1", "p2", "p3"], "{xml}");
  assert_eq!(xml.matches("<td align=\"left\">b</td>").count(), 1, "{xml}");
  assert_eq!(xml.matches("a\\vskip 2.84526ptb").count(), 1, "{xml}");
}

/// Float rows separated by a rule line (arXiv 2302.11635, IEEEtran `figure*`): `\hrulefill\vspace*` after two
/// minipages of one paragraph ends that paragraph, and each captioned minipage stays a panel of its own with its
/// caption, in the one figure.
#[test]
fn vspace_after_a_rule_line_closes_the_paragraph_of_the_minipages() {
  let panel = |text: &str| {
    format!(
      "\\begin{{minipage}}{{0.48\\textwidth}}\\centering {text}\\caption{{{text}}}\\end{{minipage}}"
    )
  };
  let (stderr, xml) = super::convert(
    &format!(
      "\\documentclass{{article}}\n\\begin{{document}}\n\\begin{{figure*}}\n{}\\hfill\n{}\n\\hrulefill\n\\vspace*{{4pt}}\n{}\\hfill\n{}\n\\vspace*{{4pt}}\n{}\n\\end{{figure*}}\n\\end{{document}}\n",
      panel("One"),
      panel("Two"),
      panel("Three"),
      panel("Four"),
      panel("Five")
    ),
    false,
  );
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  assert_eq!(
    super::perfect_kernel_batch46::warning_count(&stderr),
    0,
    "{stderr}"
  );
  assert_eq!(xml.matches("<figure ").count(), 6, "{xml}");
  assert_eq!(xml.matches("<figure xml:id=\"fig1\">").count(), 1, "{xml}");
  for (n, text) in ["One", "Two", "Three", "Four", "Five"].iter().enumerate() {
    let n = n + 1;
    let panel = format!(
      "<figure class=\"ltx_figure_panel ltx_minipage\" inlist=\"lof\" vattach=\"middle\" width=\"165.6pt\" xml:id=\"S0.F{n}\">"
    );
    let caption = format!(
      "<caption class=\"ltx_centering\"><tag close=\": \">Figure {n}</tag>{text}</caption>"
    );
    assert_eq!(xml.matches(&panel).count(), 1, "{panel}\n{xml}");
    assert_eq!(xml.matches(&caption).count(), 1, "{caption}\n{xml}");
  }
}
