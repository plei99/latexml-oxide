//! cleveref's link extent in a document's `\crefformat` / `\Crefformat` (cleveref.sty:1047-1071 `\@@setcref`,
//! :1886-1906): `#1` is the number, and only the part between `#2` and `#3` is the hyperlink; the text outside it is
//! plain text beside the link. The formats themselves (what `\cref` prints for a type, the sub-unit copies, the
//! derived variant) are OXIDIZED_DESIGN_DIVERGENCES #845; the link extent is #965: `#2` and `#3` are empty marks
//! (`\lx@cref@linkstart`, `\lx@cref@linkend`) in the format's tag, which the CrossRef pass cuts the reference at
//! (`crossref.rs` `split_cref_link`), the part between them the `<ref>` and the rest `ltx_ref_tag` text beside it. A
//! wrapper the mark sits in (`\textbf{(#2#1#3)}`) is split around the link and keeps its font on the link; a format
//! that reads neither mark links its number alone (pdflatex links nothing there; a web reference stays one), and an
//! empty format is an empty link. A type without a document format prints as before, all of it the link. Repro:
//! `tools/perfect_kernel/repros/package-bindings/cleveref_crefformat.tex`.
use std::process::Command;

const RAW: &str = "--preload=[rawstyles,rawclasses]latexml.sty";

/// Convert `t.tex` to HTML in one process. Returns (ANSI-stripped stderr, HTML).
fn convert_html(tex: &str) -> (String, String) {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  let output = Command::new(bin)
    .args([
      "t.tex",
      "--dest",
      "t.html",
      "--nocomments",
      "--timeout=110",
      RAW,
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  let html = std::fs::read_to_string(workdir.path().join("t.html")).unwrap_or_default();
  // the link-part marker is the CrossRef pass's own, never output
  assert!(!html.contains("ltx_cref_link"), "{html}");
  (stderr, html)
}

/// No error, no fatal and no warning in the log.
fn clean(stderr: &str) -> bool {
  !stderr.contains("Error:") && !stderr.contains("Fatal:") && !stderr.contains("Warning:")
}

/// Convert `t.tex` to XML. Returns (ANSI-stripped stderr, XML).
fn convert_xml(tex: &str) -> (String, String) {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  let output = Command::new(bin)
    .args([
      "t.tex",
      "--dest",
      "t.xml",
      "--nocomments",
      "--timeout=110",
      RAW,
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  let xml = std::fs::read_to_string(workdir.path().join("t.xml")).unwrap_or_default();
  (stderr, xml)
}

/// The body paragraph holding the references, between `<p class="ltx_p">` and its end.
fn paragraph(html: &str) -> &str {
  let at = html.find(r#"<p class="ltx_p">"#).expect("a paragraph");
  let rest = &html[at..];
  &rest[..rest.find("</p>").expect("paragraph end")]
}

/// `<span class="ltx_text ltx_ref_tag">text</span>`.
fn tag(text: &str) -> String { format!(r#"<span class="ltx_text ltx_ref_tag">{text}</span>"#) }

/// The `;`-separated references of a paragraph, each as the text between the separators.
fn refs(p: &str) -> Vec<&str> {
  let body = p.trim_start_matches(r#"<p class="ltx_p">"#);
  body.split("; ").collect()
}

/// `item` is `[text before]<a href=HREF …>INNER</a>[text after]`.
fn assert_ref(item: &str, before: &str, href: &str, inner: &str, after: &str) {
  let (head, rest) = item
    .split_once("<a ")
    .unwrap_or_else(|| panic!("no link in {item}"));
  assert_eq!(head, before, "text before the link in {item}");
  assert!(
    rest.starts_with(&format!(r#"href="{href}""#)),
    "link target in {item}"
  );
  let (_, rest) = rest
    .split_once(r#"class="ltx_ref">"#)
    .unwrap_or_else(|| panic!("class in {item}"));
  let (content, tail) = rest
    .split_once("</a>")
    .unwrap_or_else(|| panic!("no end in {item}"));
  assert_eq!(content, inner, "link content in {item}");
  assert_eq!(
    tail.trim_end_matches('.'),
    after,
    "text after the link in {item}"
  );
}

const PREAMBLE: &str = r"\documentclass{article}
\usepackage{hyperref}
\usepackage{cleveref}
";

/// The report's document (repro `cleveref_crefformat.tex`): `\Crefformat` for subsections,
/// with the section counter set.
#[test]
fn crefformat_cjk_wraps_the_number() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/package-bindings/cleveref_crefformat.tex");
  let (stderr, html) = convert_html(tex);
  assert!(clean(&stderr), "{stderr}");
  let p = paragraph(&html);
  let items = refs(p);
  // `\ref`: the number alone, as ever.
  assert_ref(items[0], "", "#S3.SS2", &tag("3.2"), "");
  // `\Cref`: the format's text beside the link, which covers the number.
  assert_ref(
    items[1],
    &tag("第\u{a0}"),
    "#S3.SS2",
    &tag("3.2"),
    &tag("\u{a0}节"),
  );
}

/// `\crefformat` and `\Crefformat` are told apart; a literal inside `#2`..`#3` is in the
/// link; a format without `#2`/`#3` links its number alone; unlisted types print as ever.
#[test]
fn crefformat_variants_and_link_extent() {
  let tex = format!(
    "{PREAMBLE}{}",
    r"\crefformat{subsection}{小节~#2#1#3}
\Crefformat{subsection}{第~#2#1#3~节}
\crefformat{figure}{[#2图~#1#3]}
\Crefformat{figure}{[#2图~#1#3]}
\crefformat{table}{表~#1}
\begin{document}
\section{A}\label{sec}
\subsection{B}\label{sub}
\begin{figure}\caption{F}\label{fig}\end{figure}
\begin{table}\caption{T}\label{tab}\end{table}
\cref{sub}; \Cref{sub}; \cref{fig}; \Cref{fig}; \cref{tab}; \Cref{tab}; \cref{sec}; \Cref{sec}
\end{document}
"
  );
  let (stderr, html) = convert_html(&tex);
  assert!(clean(&stderr), "{stderr}");
  let items = refs(paragraph(&html));
  assert_eq!(items.len(), 8, "{items:?}");
  assert_ref(items[0], &tag("小节\u{a0}"), "#S1.SS1", &tag("1.1"), "");
  assert_ref(
    items[1],
    &tag("第\u{a0}"),
    "#S1.SS1",
    &tag("1.1"),
    &tag("\u{a0}节"),
  );
  // `[#2图~#1#3]`: the link takes the literal between the markers, not the brackets.
  assert_ref(items[2], &tag("["), "#S1.F1", &tag("图\u{a0}1"), &tag("]"));
  // (set for `\Cref` too: cleveref derives a missing variant with `\MakeUppercase`, which since the 2022 kernel reads a
  // leading `[` as its optional language argument, so a bracket-led `\crefformat` has no usable derived `\Cref`.)
  assert_ref(items[3], &tag("["), "#S1.F1", &tag("图\u{a0}1"), &tag("]"));
  // No markers: the number alone is linked.
  assert_ref(items[4], &tag("表\u{a0}"), "#S1.T1", &tag("1"), "");
  assert_ref(items[5], &tag("表\u{a0}"), "#S1.T1", &tag("1"), "");
  // No user format: type name and number, all of it the link, exactly as before.
  assert_ref(
    items[6],
    "",
    "#S1",
    &format!("{}\u{a0}{}", tag("section"), tag("1")),
    "",
  );
  assert_ref(
    items[7],
    "",
    "#S1",
    &format!("{}\u{a0}{}", tag("Section"), tag("1")),
    "",
  );
}

/// Without any format the output is the type name and number in one link; a `\Crefname`
/// alone still renames it; lists, `\cpageref` and `\cref*` keep their shapes.
#[test]
fn names_without_formats_are_unchanged() {
  let tex = format!(
    "{PREAMBLE}{}",
    r"\Crefname{subsection}{小节}{小节}
\begin{document}
\section{A}\label{sec}
\subsection{B}\label{sub}
\cref{sec}; \Cref{sec}; \Cref{sub}; \cref{sec,sub}; \cpageref{sec}; \ref{sub}
\end{document}
"
  );
  let (stderr, html) = convert_html(&tex);
  assert!(clean(&stderr), "{stderr}");
  let items = refs(paragraph(&html));
  assert_eq!(items.len(), 6, "{items:?}");
  assert_ref(
    items[0],
    "",
    "#S1",
    &format!("{}\u{a0}{}", tag("section"), tag("1")),
    "",
  );
  assert_ref(
    items[1],
    "",
    "#S1",
    &format!("{}\u{a0}{}", tag("Section"), tag("1")),
    "",
  );
  assert_ref(
    items[2],
    "",
    "#S1.SS1",
    &format!("{}\u{a0}{}", tag("小节"), tag("1.1")),
    "",
  );
  // A list: the first label with the plural name, the rest by number.
  assert!(items[3].starts_with(r##"<a href="#S1""##), "{}", items[3]);
  assert!(
    items[3].contains(&format!("{}\u{a0}{}", tag("sections"), tag("1"))),
    "{}",
    items[3]
  );
  assert!(items[3].contains(" and\u{a0}"), "{}", items[3]);
  assert_ref(
    items[4],
    "",
    "#S1",
    &format!("{}\u{a0}{}", tag("section"), tag("1")),
    "",
  );
  assert_ref(items[5], "", "#S1.SS1", &tag("1.1"), "");
}

/// The CrossRef pass's `show` alternatives (`/`) must not touch its other `|`-separated
/// attribute lists: a TOC's `select="ltx:part | ltx:chapter | …"` once lost every entry
/// to a blanket `split('|')` rewrite made for them.
#[test]
fn a_plain_table_of_contents_keeps_its_entries() {
  let (stderr, html) = convert_html(
    r"\documentclass{article}
\begin{document}
\tableofcontents
\section{First}\section{Second}
\end{document}
",
  );
  assert!(clean(&stderr), "{stderr}");
  assert!(
    html.contains(r#"<nav class="ltx_TOC ltx_list_toc ltx_toc_toc">"#),
    "{html}"
  );
  assert_eq!(
    html.matches("ltx_tocentry ltx_tocentry_section").count(),
    2,
    "{html}"
  );
}

/// The compact text of a paragraph: ref tags in braces, no `title`/`class` noise.
fn compact(p: &str) -> String {
  let mut out = p.trim_start_matches(r#"<p class="ltx_p">"#).to_string();
  while let Some(at) = out.find(r#" title=""#) {
    let end = out[at + 8..].find('"').expect("closing quote") + at + 9;
    out.replace_range(at..end, "");
  }
  out
    .replace(r#" class="ltx_ref""#, "")
    .replace(r#"<span class="ltx_text ltx_ref_tag">"#, "{")
    .replace("</span>", "}")
}

/// The compact paragraph of `body` under a preamble loading hyperref and cleveref, with the
/// author's `setup`; the log must be clean.
fn compact_paragraph(setup: &str, body: &str) -> String {
  let tex = format!("{PREAMBLE}{setup}\\begin{{document}}\n{body}\n\\end{{document}}\n");
  let (stderr, html) = convert_html(&tex);
  assert!(clean(&stderr), "{stderr}");
  compact(paragraph(&html))
}

/// A format is read as a unit before it is case-changed: a leading `$`, `#1` alone and a
/// leading `#1` with link markers behind it all reached `\MakeUppercase` as a bare token
/// once (errors, an equation's body lost, a counter's name printed for its number).
#[test]
fn crefformat_case_change_reads_the_raw_format() {
  let p = compact_paragraph(
    r"\usepackage{amsmath}
\crefformat{section}{$\S$#2#1#3}
\crefformat{equation}{#1}
\crefformat{subsection}{#1~(#2here#3)}
",
    r"\section{A}\label{sec}\subsection{B}\label{sub}
\begin{equation}x\label{eq}\end{equation}
\cref{sec}; \cref{eq}; \Cref{eq}; \cref{sub}; \Cref{sub}",
  );
  let items: Vec<&str> = p.split("; ").collect();
  assert_eq!(items.len(), 5, "{p}");
  // `$\S$#2#1#3`: the math, then the linked number. (Its derived `\Cref`, `\MakeUppercase$\S$…`, is an error in
  // pdflatex too, "Extra }, or forgotten $": the 2022 kernel's `\MakeUppercase` typesets inside a group, which the
  // `$` it was given leaves in math.)
  assert!(items[0].starts_with("{<math"), "{p}");
  assert!(
    items[0].ends_with(r##"</math>}<a href="#S1">{1}</a>"##),
    "{p}"
  );
  for item in &items[1..3] {
    assert_eq!(*item, r##"<a href="#S1.E1">{1}</a>"##, "{p}");
  }
  for item in &items[3..] {
    // `\Cref` changes the case of the number itself, not of its counter's name.
    assert_eq!(
      item.trim_end_matches('.'),
      "{1.1\u{a0}(}<a href=\"#S1.SS1\">{here}</a>{)}",
      "{p}"
    );
  }
}

/// A format with no letters or digits stands alone: it does not pull the parent's reference
/// in beside it (`(∗) ‣ section 1`), neither for a starred equation nor for a `\fnsymbol`
/// footnote, and nothing leaks into the text after it.
#[test]
fn crefformat_without_letters_stands_alone() {
  let p = compact_paragraph(
    r"\usepackage{amsmath}
\crefformat{equation}{(#2#1#3)}
",
    r"\section{A}
\begin{equation}x\tag{$\ast$}\label{eq}\end{equation}
See \cref{eq} here and the end.",
  );
  assert!(!p.contains('\u{2023}'), "{p}");
  assert!(p.contains(r##"{(}<a href="#S1.Ex1">{<math"##), "{p}");
  assert!(p.ends_with(r"{)} here and the end."), "{p}");
  let p = compact_paragraph(
    r"\renewcommand\thefootnote{\fnsymbol{footnote}}
\crefformat{footnote}{#2#1#3}
",
    r"\section{A}
Text\footnote{F\label{fn}} more. See \cref{fn} here and the end.",
  );
  assert!(!p.contains('\u{2023}'), "{p}");
  assert!(
    p.ends_with(r##"See <a href="#footnote1">{*}</a> here and the end."##),
    "{p}"
  );
}

/// The font a format puts on its link part (`#2\textbf{Thm~#1}#3`) is the link's font.
#[test]
fn crefformat_link_keeps_its_font() {
  let p = compact_paragraph(
    r"\newtheorem{theorem}{Theorem}
\crefformat{theorem}{#2\textbf{Thm~#1}#3}
",
    r"\section{A}
See \cref{thm}.

\begin{theorem}\label{thm}T\end{theorem}",
  );
  // (the link is the bold part, number included: `#1` is typeset in the font the format sets)
  assert!(
    p.contains("<a href=\"#Thmtheorem1\">{<span class=\"ltx_text ltx_font_bold\">Thm\u{a0}1}}</a>"),
    "{p}"
  );
}

/// cleveref copies a unit's formats to its sub-units at `\begin{document}`
/// (`\@crefcopyformats`: section to subsection to subsubsection, figure to subfigure,
/// equation to subequation, enumi to enumii): `\crefformat{section}{\S#2#1#3}` reaches
/// `\cref{subsec}`. A sub-unit the author named with `\crefname` is left to its names, and
/// its own format wins.
#[test]
fn crefformat_reaches_the_sub_units() {
  let p = compact_paragraph(
    r"\crefformat{section}{SEC~#2#1#3}
\crefformat{figure}{FIG~#2#1#3}
\crefname{subsubsection}{ssub}{ssubs}
",
    r"\section{A}\label{sec}\subsection{B}\label{sub}\subsubsection{C}\label{subsub}
\cref{sec}; \cref{sub}; \cref{subsub}",
  );
  let items: Vec<&str> = p.split("; ").collect();
  assert_eq!(items[0], "{SEC\u{a0}}<a href=\"#S1\">{1}</a>", "{p}");
  assert_eq!(items[1], "{SEC\u{a0}}<a href=\"#S1.SS1\">{1.1}</a>", "{p}");
  // named by the author: its names, not the parent's format
  assert_eq!(
    items[2], "<a href=\"#S1.SS1.SSS1\">{ssub}\u{a0}{1.1.1}</a>",
    "{p}"
  );
}

/// cleveref copies formats to sub-units once, at `\begin{document}`: a `\crefformat` set in
/// the body shapes its own type and no sub-unit.
#[test]
fn crefformat_in_the_body_is_not_copied() {
  let tex = format!(
    "{PREAMBLE}{}",
    r"\begin{document}
\crefformat{section}{SEC~#2#1#3}
\section{A}\label{sec}\subsection{B}\label{sub}
\cref{sec}; \Cref{sub}
\end{document}
"
  );
  let (stderr, html) = convert_html(&tex);
  assert!(clean(&stderr), "{stderr}");
  let p = compact(paragraph(&html));
  let items: Vec<&str> = p.split("; ").collect();
  assert_eq!(items[0], "{SEC\u{a0}}<a href=\"#S1\">{1}</a>", "{p}");
  assert_eq!(
    items[1], "<a href=\"#S1.SS1\">{Section}\u{a0}{1.1}</a>",
    "{p}"
  );
}

/// The link part may sit inside the format's own markup (`\textbf{(#2#1#3)}`,
/// `\emph{Fig.~#2#1#3}`) or around a reference (`#2…\ref{x}#3`): the wrapper is split around
/// the link and the link keeps its font; the whole format is never the link, and the marker
/// never reaches the output. A title that holds such a reference carries no marker either.
#[test]
fn crefformat_link_inside_markup() {
  let p = compact_paragraph(
    r"\usepackage{amsmath}
\crefformat{equation}{\textbf{(#2#1#3)}}
\crefformat{figure}{\emph{Fig.~#2#1#3}}
\crefformat{section}{#2Sec.~#1 of \ref{last}#3}
",
    r"\section{A}\label{sec}
\begin{equation}x\label{eq}\end{equation}
\begin{figure}\caption{F}\label{fig}\end{figure}
\section{Z}\label{last}
X1 \cref{eq}; X2 \cref{fig}; X3 \cref{sec}.",
  );
  let nb = '\u{a0}';
  let bold = r#"<span class="ltx_text ltx_font_bold">"#;
  let italic = r#"<em class="ltx_emph ltx_font_italic">"#;
  assert!(
    p.contains(&format!(
      r##"X1 {{{bold}(}}}}<a href="#S1.E1">{{{bold}1}}}}</a>{{{bold})}}}}"##
    )),
    "{p}"
  );
  assert!(
    p.contains(&format!(
      r##"X2 {{{italic}Fig.{nb}</em>}}<a href="#S1.F1">{{{italic}1</em>}}</a>"##
    )),
    "{p}"
  );
  assert!(
    p.contains(&format!(r##"X3 <a href="#S1">{{Sec.{nb}1 of {{2}}}}</a>"##)),
    "{p}"
  );
  // A section title holding a `\cref` with a format: the TOC entry and `\nameref` text.
  let (stderr, html) = convert_html(&format!(
    "{PREAMBLE}{}",
    r"\crefformat{figure}{FIG~#2#1#3 end}
\begin{document}
\tableofcontents
\begin{figure}\caption{F}\label{fig}\end{figure}
\section{About \cref{fig} here}\label{sec}
\nameref{sec}
\end{document}
"
  ));
  assert!(clean(&stderr), "{stderr}");
  assert!(html.matches("FIG").count() >= 2, "{html}");
}

/// An empty format is an empty reference (pdflatex prints nothing for it), not the type's
/// name and number.
#[test]
fn crefformat_empty_prints_nothing() {
  let p = compact_paragraph(
    r"\crefformat{section}{}
",
    r"\section{A}\label{sec}
[\cref{sec}]",
  );
  assert_eq!(p, r##"[<a href="#S1">{}</a>]"##);
}

/// Without a document format the XML has no `crefformat` tags and no link marks: the references show the type's
/// name and number.
#[test]
fn no_format_leaves_the_xml_alone() {
  let (stderr, xml) = convert_xml(&format!(
    "{PREAMBLE}{}",
    r"\begin{document}
\section{A}\label{sec}
\cref{sec}; \Cref{sec}
\end{document}
"
  ));
  assert!(clean(&stderr), "{stderr}");
  assert!(!xml.contains("crefformat"), "{xml}");
  assert!(!xml.contains("ltx_cref_link"), "{xml}");
  assert!(xml.contains(r#"show="creftype~creflabel""#), "{xml}");
  assert!(xml.contains(r#"show="creftypecap~creflabel""#), "{xml}");
}

/// A `\cref` list under a multi-reference format with a link extent: each reference's text before its link sits
/// between the previous link and its own, and the conjunction cleveref's pair format would add (`\crefpairconjunction`)
/// is dropped, as it is for a format that is all link (`settle_cref_conjunctions`): "eqs. (1 and 2)", not
/// "eqs. (1 and  and 2)".
#[test]
fn crefmultiformat_list_has_no_second_conjunction() {
  let p = compact_paragraph(
    r"\usepackage{amsmath}
\crefmultiformat{equation}{eqs.~(#2#1#3}{ and~#2#1#3)}{, #2#1#3}{, and~#2#1#3)}
",
    r"\begin{equation}a\label{e1}\end{equation}
\begin{equation}b\label{e2}\end{equation}
\begin{equation}c\label{e3}\end{equation}
X \cref{e1,e2}; \cref{e1,e2,e3}.",
  );
  let nb = '\u{a0}';
  assert!(
    p.contains(&format!(
      r##"X {{eqs.{nb}(}}<a href="#S0.E1">{{1}}</a>{{ and{nb}}}<a href="#S0.E2">{{2}}</a>{{)}}; "##
    )),
    "{p}"
  );
  assert!(
    p.contains(&format!(
      r##"{{eqs.{nb}(}}<a href="#S0.E1">{{1}}</a>{{, }}<a href="#S0.E2">{{2}}</a>{{, and{nb}}}<a href="#S0.E3">{{3}}</a>{{)}}."##
    )),
    "{p}"
  );
}
