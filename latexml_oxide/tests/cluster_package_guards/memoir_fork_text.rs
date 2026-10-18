//! Guards for `latexml_contrib/src/memoir_fork/text.rs` (fork-only memoir bindings): memoir's emulated `parskip`
//! (memoir.cls:132-152, :12211-12253) is a no-op `\usepackage`, and `\sourceatright` (memoir.cls:4548-4551) sets its
//! text flush right at the end of the paragraph.

/// Converts `tex` as `m.tex` with the raw class/style protocol; returns the XML and the stderr (ANSI stripped).
fn convert(tex: &str) -> (String, String) {
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("m.tex"), tex).expect("write m.tex");
  let output = std::process::Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args([
      "m.tex",
      "--dest",
      "m.xml",
      "--timeout=110",
      "--preload=[rawstyles,rawclasses]latexml.sty",
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  let xml = std::fs::read_to_string(workdir.path().join("m.xml")).expect("read m.xml");
  (xml, stderr)
}

/// A document loading parskip as pdfLaTeX accepts it under memoir: without options (`\usepackage[skip=4pt]{parskip}`
/// is an `Option clash for package parskip` error there, memoir having loaded it, memoir.cls:12241).
fn doc(class: &str, body: &str) -> String {
  format!(
    "\\documentclass{{{class}}}\n\\usepackage{{parskip}}\n\\begin{{document}}\n{body}\n\\end{{document}}\n"
  )
}

const PARAGRAPHS: &str = "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.";

/// memoir marks parskip loaded, so `\usepackage{parskip}` changes nothing: the paragraphs keep their
/// first-line indent (no `noindent`), memoir's `\parskip` stays, and parskip's own `\patchcmd`s are not tried. memoir's
/// raw load prints its two `\immediateaddtocontents` patch infos and one `\@iffirstamp` warning, pinned here.
#[test]
fn memoir_parskip_is_emulated_noop() {
  let (xml, stderr) = convert(&doc("memoir", PARAGRAPHS));
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<para ").count(), 3, "three paragraphs:\n{xml}");
  assert_eq!(
    xml.matches("noindent").count(),
    0,
    "parskip is emulated by memoir, paragraphs keep their indent:\n{xml}"
  );
  assert_eq!(
    stderr.matches("Warning:").count(),
    1,
    "only memoir's own warning:\n{stderr}"
  );
  assert_eq!(
    stderr.matches("Warning:misdefined:\\@iffirstamp").count(),
    1,
    "{stderr}"
  );
  assert_eq!(
    stderr.matches("Info:unexpected:patchcmd").count(),
    2,
    "memoir's two \\immediateaddtocontents patches, none of parskip's:\n{stderr}"
  );
  assert!(
    !stderr.contains("Couldn't patch"),
    "parskip's \\patchcmd\\@startsection is not run:\n{stderr}"
  );
}

/// An option on the emulated package is an error in pdfLaTeX (`Option clash`); LaTeXML returns before it looks at the
/// options and tolerates it, with the same indented paragraphs. Pinned as the tolerance, not as parity.
#[test]
fn memoir_parskip_options_are_tolerated() {
  let (xml, stderr) = convert(&format!(
    "\\documentclass{{memoir}}\n\\usepackage[skip=4pt]{{parskip}}\n\\begin{{document}}\n{PARAGRAPHS}\n\\end{{document}}\n"
  ));
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("noindent").count(), 0, "{xml}");
  assert_eq!(stderr.matches("Warning:").count(), 1, "{stderr}");
}

/// The contrast: under `book` the real parskip applies, every paragraph is `noindent` and its `\patchcmd` runs.
#[test]
fn book_parskip_still_applies() {
  let (xml, stderr) = convert(&doc("book", PARAGRAPHS));
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  assert_eq!(
    xml.matches("noindent").count(),
    3,
    "parskip zeroes \\parindent under book:\n{xml}"
  );
  assert_eq!(
    stderr.matches("Info:unexpected:patchcmd").count(),
    1,
    "parskip's \\@startsection patch fails under book:\n{stderr}"
  );
}

/// `\sourceatright[len]{text}` ends the paragraph with `text` as a right float, `len` as its left margin. The float
/// stays in its paragraph through `ltx-memoir.css` (a paragraph holding one is `display:flow-root`), so that file must
/// be required by memoir itself, whether or not the document has a chapter.
#[test]
fn memoir_sourceatright_is_a_right_float() {
  let (xml, stderr) = convert(&doc(
    "memoir",
    "\\begin{quotation}\n\\itshape Some quoted words.\n\\sourceatright{A. Author}\n\\end{quotation}\n\
     After.\n\n\\begin{quotation}\nShort.\\sourceatright[1em]{Somebody}\n\\end{quotation}",
  ));
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  assert_eq!(stderr.matches("Warning:").count(), 1, "{stderr}");
  assert_eq!(
    xml.matches("<text class=\"ltx_align_floatright\"").count(),
    2,
    "one float per \\sourceatright:\n{xml}"
  );
  // `[2em]` (the default) is 2 x 10.2pt in the italic quotation, `[1em]` is 1 x 10pt in the upright one.
  assert!(
    xml.contains("cssstyle=\"margin-left:20.4pt\">A. Author</text></text></p>"),
    "the default gap is the float's margin, inside the quotation's paragraph:\n{xml}"
  );
  assert!(
    xml.contains("<p>Short.<text class=\"ltx_align_floatright\" cssstyle=\"margin-left:10.0pt\">Somebody</text></p>"),
    "the optional gap, at the end of the paragraph:\n{xml}"
  );
  assert!(
    xml.contains("<resource src=\"ltx-memoir.css\""),
    "ltx-memoir.css is required whenever memoir is loaded:\n{xml}"
  );
  // The float ends its paragraph (memoir's `\par`): the text after the quotation is a paragraph of its own.
  assert!(
    xml.contains("<p>After.</p>"),
    "the text after the quotation:\n{xml}"
  );
}

/// The stylesheet holds the float in its paragraph: the rule is in the embedded `ltx-memoir.css` and written to the
/// destination by the post-processor with the page.
#[test]
fn memoir_css_contains_the_sourceatright_float() {
  let css = include_str!("../../../latexml_post/resources/CSS/ltx-memoir.css");
  assert!(
    css.contains(".ltx_p:has(.ltx_align_floatright) { display: flow-root; }"),
    "{css}"
  );
}
