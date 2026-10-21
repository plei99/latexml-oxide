//! An `\item`'s label is typeset in the font in force at the `\item` (OXIDIZED_DESIGN_DIVERGENCES #604).
//!
//! LaTeX's `\@item` sets `\makelabel{\labelenumi}` in the list's current font (latex.ltx), so the `1.` `2.` of an
//! enumerate inside a plain-style theorem (`\itshape` body) print in italic, as a bullet does. Perl LaTeXML builds every
//! `ltx:tag` after `neutralizeFont` and `\normalfont` (Base_Utility.pool.ltxml:1005, 1041), so its labels are always
//! upright: the item body carries the italic, the label does not. Only the displayed tag follows the context: the
//! `refnum` and `typerefnum` tags, which `\ref` copies into running text, stay plain. A bullet is upright all the same (`\labelitemi` starts `\labelitemfont`,
//! `\normalfont`, article.cls:355-359), a description label keeps the `\descriptionlabel`'s `\normalfont\bfseries`, and a
//! label with an explicit font (`enumitem`'s `font=`) adds to the context's, as it does in LaTeX. The font is the label's
//! only: a footnote in it, whose mark is `\normalfont`, is typeset as one in the text.
use super::perfect_kernel_batch46::{error_count, warning_count};

const PREAMBLE: &str = "\\documentclass{article}\n\\usepackage{amsthm}\n\\usepackage{enumitem}\n\\usepackage{xcolor}\n\\newtheorem{lemma}{Lemma}\n\\begin{document}\n";

/// `body` in an article, converted to XML: (stderr, xml), with the conversion required clean.
fn xml(body: &str) -> String {
  let tex = format!("{PREAMBLE}{body}\n\\end{{document}}\n");
  let (stderr, xml) = super::convert(&tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  xml
}

/// Every displayed (role-less) `ltx:tag` of an `ltx:item`, whole, in document order.
fn item_labels(xml: &str) -> Vec<String> {
  let mut labels = Vec::new();
  let mut rest = xml;
  while let Some(at) = [rest.find("<item "), rest.find("<inline-item ")]
    .into_iter()
    .flatten()
    .min()
  {
    rest = &rest[at..];
    let from = rest.find("<tag>").expect("an item with a tag");
    let to = from + rest[from..].find("</tag>").expect("end of the tag") + "</tag>".len();
    labels.push(rest[from..to].to_string());
    rest = &rest[to..];
  }
  labels
}

/// The `refnum` and `typerefnum` tags of the first item, whole.
fn ref_tags(xml: &str) -> Vec<String> {
  let first = &xml[xml.find("<item ").expect("an item")..];
  let first = &first[..first.find("</tags>").expect("end of the tags")];
  first
    .lines()
    .map(str::trim)
    .filter(|l| l.starts_with("<tag role="))
    .map(str::to_string)
    .collect()
}

#[test]
fn enumerate_labels_in_an_italic_theorem_are_italic() {
  let xml = xml(
    "\\begin{lemma}Let $x$. \\begin{enumerate}\\item First. \\item Second.\\end{enumerate}\\end{lemma}",
  );
  assert_eq!(item_labels(&xml), [
    r#"<tag><text font="italic">1.</text></tag>"#,
    r#"<tag><text font="italic">2.</text></tag>"#
  ]);
  // `\ref` copies these into running text: they carry no font.
  assert_eq!(ref_tags(&xml), [
    r#"<tag role="refnum">1</tag>"#,
    "<tag role=\"typerefnum\">item\u{a0}1</tag>"
  ]);
}

#[test]
fn enumerate_labels_in_upright_text_stay_upright() {
  let xml = xml(
    "\\begin{lemma}Let $x$. \\begin{enumerate}\\item First.\\end{enumerate}\\end{lemma}\n\
     \\begin{enumerate}\\item Up. \\item Two.\\end{enumerate}",
  );
  assert_eq!(item_labels(&xml), [
    r#"<tag><text font="italic">1.</text></tag>"#,
    "<tag>1.</tag>",
    "<tag>2.</tag>"
  ]);
}

#[test]
fn itemize_bullets_are_upright_in_an_italic_theorem() {
  // `\labelitemi` is `\labelitemfont \textbullet` and `\labelitemfont` is `\normalfont` (pdflatex's `\showbox\@labels`
  // in a plain lemma: the bullet `\TS1/cmr/m/n/10`, the enumerate label `\OT1/cmr/m/it/10`).
  let xml = xml(
    "\\begin{lemma}\\begin{itemize}\\item Dot.\\end{itemize}\\end{lemma}\n\
     \\begin{itemize}\\item Dot.\\end{itemize}",
  );
  assert_eq!(item_labels(&xml), ["<tag>•</tag>", "<tag>•</tag>"]);
}

#[test]
fn description_labels_stay_bold_upright_in_an_italic_theorem() {
  let xml = xml(
    "\\begin{lemma}\\begin{description}\\item[Key] Desc.\\end{description}\\end{lemma}\n\
     \\begin{description}\\item[Key] Desc.\\end{description}",
  );
  // `\descriptionlabel` is `\normalfont\bfseries #1`: the label of an italic context is bold, not bold italic.
  assert_eq!(item_labels(&xml), [
    r#"<tag><text font="bold">Key</text></tag>"#,
    r#"<tag><text font="bold">Key</text></tag>"#
  ]);
}

#[test]
fn an_explicit_label_font_adds_to_the_context_font() {
  let xml = xml(
    "\\begin{lemma}\\begin{enumerate}[font=\\bfseries]\\item Bf.\\end{enumerate}\n\
     \\begin{enumerate}\\item[(x)] Explicit.\\end{enumerate}\\end{lemma}\n\
     \\begin{enumerate}[font=\\bfseries]\\item Bf.\\end{enumerate}",
  );
  assert_eq!(item_labels(&xml), [
    r#"<tag><text font="bold italic">1.</text></tag>"#,
    r#"<tag><text font="italic">(x)</text></tag>"#,
    r#"<tag><text font="bold">1.</text></tag>"#
  ]);
}

#[test]
fn a_label_takes_the_size_in_force() {
  let xml = xml("{\\small\\begin{enumerate}\\item Small.\\end{enumerate}}");
  assert_eq!(item_labels(&xml), [
    r#"<tag><text fontsize="90%">1.</text></tag>"#
  ]);
}

#[test]
fn a_label_takes_the_colour_in_force() {
  let xml =
    xml("{\\color{red}\\begin{lemma}\\begin{enumerate}\\item Red.\\end{enumerate}\\end{lemma}}");
  assert_eq!(item_labels(&xml), [
    r##"<tag><text color="#FF0000" font="italic">1.</text></tag>"##
  ]);
}

#[test]
fn a_label_inside_a_small_caption_is_not_resized() {
  // The tag is typeset in the caption's `\small`, which is also the context: no `<text fontsize>` of its own (the
  // neutral font was normalsize, and read as 111% of the caption).
  let xml = xml(
    "\\begin{figure}\\caption{\\small cap \\begin{enumerate}\\item In caption.\\end{enumerate}}\\end{figure}",
  );
  assert_eq!(item_labels(&xml), ["<tag>1.</tag>", "<tag>1.</tag>"]);
}

#[test]
fn inline_item_labels_in_an_italic_theorem_are_not_wrapped() {
  // An `enumerate*` item sits in the theorem's italic text, so the label's italic is the context's: no wrapper, where
  // the neutral font made it `<text font="upright">`.
  let tex = "\\documentclass{article}\n\\usepackage{amsthm}\n\\usepackage[inline]{enumitem}\n\\newtheorem{lemma}{Lemma}\n\\begin{document}\n\
    \\begin{lemma}Let. \\begin{enumerate*}\\item One \\item Two\\end{enumerate*}\\end{lemma}\n\\end{document}\n";
  let (stderr, xml) = super::convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_eq!(item_labels(&xml), ["<tag>1.</tag>", "<tag>2.</tag>"]);
}

/// The `ltx:tags` of the first footnote, whole, with the indentation removed.
fn note_tags(xml: &str) -> String {
  let note = &xml[xml.find("<note ").expect("a footnote")..];
  let from = note.find("<tags>").expect("the note's tags");
  let to = from + note[from..].find("</tags>").expect("end of the tags") + "</tags>".len();
  note[from..to]
    .lines()
    .map(str::trim)
    .collect::<Vec<_>>()
    .join("")
}

#[test]
fn a_footnote_in_a_label_is_set_as_a_footnote_in_the_text() {
  // The restored font is for the item's own counter: `\@makefnmark` is `\normalfont` (latex.ltx:17879), so the mark
  // of a footnote in the label is upright, as in the theorem's text, not italic from the label.
  let in_label = xml(
    "\\begin{lemma}\\begin{enumerate}\\item[a\\footnote{inlabel}] Foot.\\end{enumerate}\\end{lemma}",
  );
  let in_text = xml("\\begin{lemma}Text\\footnote{plain}.\\end{lemma}");
  let expected = concat!(
    "<tags><tag><text font=\"upright\">1</text></tag>",
    "<tag role=\"refnum\"><text font=\"upright\">1</text></tag>",
    "<tag role=\"typerefnum\"><text font=\"upright\">footnote 1</text></tag></tags>"
  );
  assert_eq!(note_tags(&in_text), expected);
  assert_eq!(note_tags(&in_label), expected);
}
