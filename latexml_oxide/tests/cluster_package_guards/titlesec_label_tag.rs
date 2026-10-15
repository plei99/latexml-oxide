//! titlesec's `\titleformat` label is the title's number, so it is an `ltx:tag` as in a title without
//! titlesec (`\lx@@compose@title` = `\lx@tag[][ ]{#1}#2`), not bare text of the title (Perl's
//! titlesec.sty.ltxml:55 inserts `$label` untagged; OXIDIZED_DESIGN_DIVERGENCES #583). The tag holds
//! the formatted label alone; the separator (`\hspace`: a run of spaces)
//! stays outside it, an empty label makes no tag, and the format's font stays on the title, so the
//! HTML heading keeps the `style="font-size:…"` that overrides its class's size, and the tag is an
//! inline run (`.ltx_tag_titlesec` in LaTeXML.css), not the block ltx-book.css makes of a chapter's.

use super::perfect_kernel_batch46::{error_count, warning_count};

/// Convert `tex` (a file `t.tex`) to `t.<ext>` with zero errors: the stderr, the output, and the
/// output directory (kept alive by the caller).
fn convert_to(tex: &str, ext: &str) -> (String, String, tempfile::TempDir) {
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  let dest = format!("t.{ext}");
  let output = std::process::Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args([
      "t.tex",
      "--dest",
      &dest,
      "--timeout=110",
      "--preload=[rawstyles,rawclasses]latexml.sty",
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  let out = std::fs::read_to_string(workdir.path().join(&dest)).expect("read output");
  (stderr, out, workdir)
}

/// Convert `tex` to XML, with zero errors and zero warnings.
fn xml_of(tex: &str) -> String {
  let (stderr, xml, _dir) = convert_to(tex, "xml");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  xml
}

/// `text` with each run of spaces (any width: `\hspace{1em}` is an em space, and a hair space more in a
/// bold or larger font) as one ASCII space, so what a test pins is whether a separator is there.
fn squeeze(text: &str) -> String {
  let mut out = String::with_capacity(text.len());
  let mut in_space = false;
  for c in text.chars() {
    if c != '\n' && c.is_whitespace() {
      if !in_space {
        out.push(' ');
      }
      in_space = true;
    } else {
      out.push(c);
      in_space = false;
    }
  }
  out
}

/// The `<title>…</title>` elements of an XML output, in order, whole: the start tag through the end tag.
fn titles(xml: &str) -> Vec<String> {
  let mut out = Vec::new();
  let mut rest = xml;
  while let Some(start) = rest.find("<title") {
    let tail = &rest[start..];
    let stop = tail.find("</title>").expect("title end tag") + "</title>".len();
    out.push(squeeze(&tail[..stop]));
    rest = &tail[stop..];
  }
  out
}

/// The `<h1>`…`<h6>` headings of an HTML output, in order, whole (not the page's `<title>`, `<head>`).
fn headings(html: &str) -> Vec<String> {
  let mut out = Vec::new();
  let mut rest = html;
  while let Some(start) = rest.find("<h") {
    let tail = &rest[start..];
    let Some(level) = tail.as_bytes().get(2).filter(|d| (b'1'..=b'6').contains(d)) else {
      rest = &tail[2..];
      continue;
    };
    let end = format!("</h{}>", *level as char);
    let stop = tail.find(&end).unwrap_or_else(|| panic!("no {end}")) + end.len();
    out.push(squeeze(&tail[..stop]));
    rest = &tail[stop..];
  }
  out
}

const ARTICLE: &str = "\\documentclass{article}\n\\usepackage{titlesec}\n\
  \\titleformat{\\section}{\\normalfont}{\\thesection}{1em}{}\n\
  \\begin{document}\n\\section{Example}\\label{sec:example}\n\
  See Section~\\ref{sec:example}.\n\\end{document}\n";

#[test]
fn section_number_is_a_tag() {
  let xml = xml_of(ARTICLE);
  assert_eq!(
    titles(&xml),
    ["<title><tag class=\"ltx_tag_titlesec\">1</tag> Example</title>"],
    "{xml}"
  );
  // The metadata and the reference are the number's, unchanged.
  assert!(xml.contains("<tag role=\"refnum\">1</tag>"), "{xml}");
  assert!(
    xml.contains("<ref labelref=\"LABEL:sec:example\"/>"),
    "{xml}"
  );
}

#[test]
fn section_number_is_a_tag_in_html() {
  let (_, html, dir) = convert_to(ARTICLE, "html");
  assert_eq!(
    headings(&html),
    [
      "<h2 class=\"ltx_title ltx_title_section\"><span class=\"ltx_tag ltx_tag_titlesec \
      ltx_tag_section\">1</span> Example</h2>"
    ],
    "{html}"
  );
  // The reference still reads the number.
  assert!(
    html.contains("<span class=\"ltx_text ltx_ref_tag\">1</span>"),
    "{html}"
  );
  // The stylesheet makes the tag an inline run, whatever the class's stylesheet says of a tag.
  let css = std::fs::read_to_string(dir.path().join("LaTeXML.css")).expect("read LaTeXML.css");
  assert!(
    css.contains(
      ".ltx_title .ltx_tag.ltx_tag_titlesec { display:inline; font-size:inherit; margin:0; }"
    ),
    "the tag's rule is in LaTeXML.css"
  );
}

#[test]
fn subsection_and_formatted_label() {
  let tex = "\\documentclass{article}\n\\usepackage{titlesec}\n\
    \\titleformat{\\subsection}{\\normalfont\\bfseries}{\\itshape\\thesubsection.}{1em}{}\n\
    \\begin{document}\n\\section{Top}\n\\subsection{Sub}\n\\end{document}\n";
  let xml = xml_of(tex);
  // Only the subsection has a format. The format's bold is the title's, whole; what the label sets
  // itself (italic, with its trailing period) stays in its tag.
  assert_eq!(
    titles(&xml),
    [
      "<title><tag close=\" \">1</tag>Top</title>",
      "<title font=\"bold\"><tag class=\"ltx_tag_titlesec\"><text font=\"italic\">1.1.</text></tag> \
        Sub</title>"
    ],
    "{xml}"
  );
}

#[test]
fn a_size_switch_in_the_format_stays_on_the_title() {
  // The HTML heading carries `style="font-size:144%"`, which overrides the class's 140%; sizes in
  // runs inside the title would multiply with the class's (about 200%).
  let tex = "\\documentclass{article}\n\\usepackage{titlesec}\n\
    \\titleformat{\\section}{\\Large\\bfseries}{\\thesection}{1em}{}\n\
    \\begin{document}\n\\section{Sized}\n\\end{document}\n";
  let xml = xml_of(tex);
  assert_eq!(
    titles(&xml),
    [
      "<title font=\"bold\" fontsize=\"144%\"><tag class=\"ltx_tag_titlesec\">1</tag> Sized</title>"
    ],
    "{xml}"
  );
  let (_, html, _dir) = convert_to(tex, "html");
  assert_eq!(
    headings(&html),
    [
      "<h2 class=\"ltx_title ltx_font_bold ltx_title_section\" style=\"font-size:144%;\">\
      <span class=\"ltx_tag ltx_tag_titlesec ltx_tag_section\">1</span> Sized</h2>"
    ],
    "{html}"
  );
}

#[test]
fn text_the_format_prints_keeps_the_font_it_was_typeset_in() {
  // Text the format prints before a font switch is typeset in the title's earlier font, as in TeX:
  // the title takes the format's final font only when it holds nothing else.
  let tex = "\\documentclass{article}\n\\usepackage{titlesec}\n\
    \\titleformat{\\section}{XX\\bfseries}{\\thesection}{1em}{}\n\
    \\titleformat{\\subsection}{AA\\bfseries XX}{\\thesubsection}{1em}{}\n\
    \\titleformat{\\subsubsection}{\\bfseries XX\\itshape}{\\thesubsubsection}{1em}{}\n\
    \\begin{document}\n\\section{Alpha}\n\\subsection{Beta}\n\\subsubsection{Gamma}\n\\end{document}\n";
  let xml = xml_of(tex);
  assert_eq!(
    titles(&xml),
    [
      "<title>XX<tag class=\"ltx_tag_titlesec\"><text font=\"bold\">1</text></tag>\
       <text font=\"bold\"> Alpha</text></title>",
      "<title>AA<text font=\"bold\">XX</text><tag class=\"ltx_tag_titlesec\">\
       <text font=\"bold\">1.1</text></tag><text font=\"bold\"> Beta</text></title>",
      "<title><text font=\"bold\">XX</text><tag class=\"ltx_tag_titlesec\">\
       <text font=\"bold italic\">1.1.1</text></tag><text font=\"bold italic\"> Gamma</text></title>"
    ],
    "{xml}"
  );
  // The whole title in the format's font, though: the font is the title's.
  let tex = "\\documentclass{article}\n\\usepackage{titlesec}\n\
    \\titleformat{\\section}{\\bfseries XX}{\\thesection}{1em}{}\n\
    \\begin{document}\n\\section{Alpha}\n\\end{document}\n";
  let xml = xml_of(tex);
  assert_eq!(
    titles(&xml),
    ["<title font=\"bold\">XX<tag class=\"ltx_tag_titlesec\">1</tag> Alpha</title>"],
    "{xml}"
  );
}

/// A book with `\titleformat{\chapter}[<shape>]{\Huge\bfseries}{\thechapter}{<sep>}{}`, converted to HTML.
fn book_chapter(shape: &str, sep: &str) -> String {
  let tex = format!(
    "\\documentclass{{book}}\n\\usepackage{{titlesec}}\n\
     \\titleformat{{\\chapter}}[{shape}]{{\\Huge\\bfseries}}{{\\thechapter}}{{{sep}}}{{}}\n\
     \\begin{{document}}\n\\chapter{{Introduction}}\nText.\n\\end{{document}}\n"
  );
  let (_, html, dir) = convert_to(&tex, "html");
  // ltx-book.css makes a chapter title's tag a block (the number above the title); the rule that
  // undoes it for titlesec's tag is in the stylesheet that comes first and wins on specificity.
  assert!(dir.path().join("ltx-book.css").exists(), "{html}");
  html
}

#[test]
fn a_book_chapter_number_stays_on_the_title_line() {
  let tag = "<span class=\"ltx_tag ltx_tag_titlesec ltx_tag_chapter\">1</span>";
  let head = "<h2 class=\"ltx_title ltx_font_bold ltx_title_chapter\" style=\"font-size:298%;\">";
  // hang: the number and the title on one line.
  let html = book_chapter("hang", "20pt");
  assert_eq!(
    headings(&html),
    [format!("{head}{tag} Introduction</h2>")],
    "{html}"
  );
  // block: likewise.
  let html = book_chapter("block", "20pt");
  assert_eq!(
    headings(&html),
    [format!("{head}{tag} Introduction</h2>")],
    "{html}"
  );
  // display: the number, then titlesec's own line break (one, not the stylesheet's block as well).
  let html = book_chapter("display", "20pt");
  assert_eq!(
    headings(&html),
    [format!(
      "{head}{tag}\n<br class=\"ltx_break\">Introduction</h2>"
    )],
    "{html}"
  );
  // runin: the heading runs into the text, the number inside it.
  let html = book_chapter("runin", "20pt");
  assert_eq!(
    headings(&html),
    [format!(
      "<h2 class=\"ltx_title ltx_runin ltx_font_bold ltx_title_chapter\" style=\"font-size:298%;\">\
       {tag} Introduction</h2>"
    )],
    "{html}"
  );
}

#[test]
fn memoir_display_chapter() {
  let tex = "\\documentclass{memoir}\n\\usepackage{titlesec}\n\
    \\titleformat{\\chapter}[display]{\\normalfont\\bfseries}{\\chaptertitlename\\ \\thechapter}{20pt}{\\Huge}\n\
    \\begin{document}\n\\chapter{Intro}\\label{ch:intro}\n\\chapter*{Starred}\n\
    See Chapter~\\ref{ch:intro}.\n\\end{document}\n";
  let (stderr, xml, _dir) = convert_to(tex, "xml");
  // memoir's own: titlesec is not recommended with it, and its `\@iffirstamp` is a conditional that
  // does not start with `\if` (memoir-wide, the same without titlesec; not a titlesec leak); nothing else.
  assert_eq!(warning_count(&stderr), 2, "{stderr}");
  assert!(
    stderr.contains("The titlesec package is not recommended with memoir"),
    "{stderr}"
  );
  assert!(stderr.contains("misdefined:\\@iffirstamp"), "{stderr}");
  // The label (name and number) is the tag, on a line of its own above the title; a starred
  // heading has no number, so no tag.
  assert_eq!(
    titles(&xml),
    [
      "<title font=\"bold\"><tag class=\"ltx_tag_titlesec\">Chapter 1</tag><break/>\
       <text fontsize=\"207%\">Intro</text></title>",
      "<title>Starred</title>"
    ],
    "{xml}"
  );
  assert!(xml.contains("<ref labelref=\"LABEL:ch:intro\"/>"), "{xml}");
}

#[test]
fn empty_label_and_starred_section_have_no_tag() {
  let tex = "\\documentclass{article}\n\\usepackage{titlesec}\n\
    \\titleformat{\\section}{\\normalfont}{}{0pt}{}\n\
    \\titleformat{\\subsection}{\\normalfont}{ }{0pt}{}\n\
    \\begin{document}\n\\section{Plain}\n\\subsection{Spaced}\n\\section*{Star}\n\\end{document}\n";
  let xml = xml_of(tex);
  assert_eq!(
    titles(&xml),
    [
      "<title>Plain</title>",
      "<title>Spaced</title>",
      "<title>Star</title>"
    ],
    "{xml}"
  );
}

#[test]
fn label_shapes() {
  let tex = "\\documentclass{article}\n\\usepackage{titlesec}\n\
    \\titleformat{\\section}[runin]{\\normalfont}{\\thesection.}{.5em}{}\n\
    \\titleformat{\\subsection}[block]{\\normalfont}{\\thesubsection}{.5em}{}\n\
    \\titleformat{\\subsubsection}[frame]{\\normalfont}{\\thesubsubsection}{1em}{}\n\
    \\begin{document}\n\\section{Run}\n\\subsection{Blk}\n\\subsubsection{Frm}\n\\end{document}\n";
  let xml = xml_of(tex);
  assert_eq!(
    titles(&xml),
    [
      "<title class=\"ltx_runin\"><tag class=\"ltx_tag_titlesec\">1.</tag> Run</title>",
      "<title><tag class=\"ltx_tag_titlesec\">1.1</tag> Blk</title>",
      "<title class=\"ltx_framed ltx_framed_rectangle\"><tag class=\"ltx_tag_titlesec\">1.1.1</tag>\
       <break/>Frm</title>"
    ],
    "{xml}"
  );
}

#[test]
fn titleformat_star_label_keeps_its_quad_in_the_tag() {
  // `\titleformat*` on a section never given a `\titleformat` takes titlesec's default label,
  // `\@seccntformat{section}` = `\csname thesection\endcsname\quad`: the quad is the label's own, so
  // it stays in the tag (OXIDIZED_DESIGN_DIVERGENCES #583).
  let tex = "\\documentclass{article}\n\\usepackage{titlesec}\n\
    \\titleformat*{\\subsection}{\\bfseries}\n\
    \\begin{document}\n\\section{Top}\n\\subsection{Sub}\n\\end{document}\n";
  let xml = xml_of(tex);
  assert_eq!(
    titles(&xml),
    [
      "<title><tag close=\" \">1</tag>Top</title>",
      "<title font=\"bold\"><tag class=\"ltx_tag_titlesec\">1.1 </tag>Sub</title>"
    ],
    "{xml}"
  );
}
