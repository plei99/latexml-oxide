//! Guards for `latexml_contrib/src/memoir_fork/layout.rs` (fork-only memoir bindings): the section-numbering and ToC
//! depth commands, the `\cft<level>font` commands reaching the ToC entries, the lists, and the page-layout arithmetic.
//! A ToC is built by the post stage, so each document is converted to core XML and post-processed to HTML in a second
//! process, as the corpus does.
use std::process::Command;

use super::perfect_kernel_batch46::error_count;

const RAW: &str = "--preload=[rawstyles,rawclasses]latexml.sty";

/// Convert `t.tex` to core XML, then post-process that XML alone to HTML. Returns the ANSI-stripped stderr of the core
/// run, the core XML and the HTML.
fn convert_xml_then_html(tex: &str) -> (String, String, String) {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  let core = Command::new(bin)
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
    .expect("spawn core");
  let post = Command::new(bin)
    .args([
      "--whatsin=xml",
      "t.xml",
      "--dest",
      "p.html",
      "--timeout=110",
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn post");
  let post_stderr = String::from_utf8_lossy(&post.stderr).replace('\u{1b}', "");
  assert_eq!(
    error_count(&post_stderr),
    0,
    "post-processing:\n{post_stderr}"
  );
  let stderr = String::from_utf8_lossy(&core.stderr).replace('\u{1b}', "");
  let xml = std::fs::read_to_string(workdir.path().join("t.xml")).unwrap_or_default();
  let html = std::fs::read_to_string(workdir.path().join("p.html")).unwrap_or_default();
  (stderr, xml, html)
}

/// The `Warning:<category>:<object>` heads in a log, in order.
fn warning_heads(stderr: &str) -> Vec<String> {
  stderr
    .lines()
    .filter(|l| l.starts_with("Warning:"))
    .map(|l| l.split_whitespace().next().unwrap_or_default().to_string())
    .collect()
}

/// memoir's one warning on every class load: `\newif\@iffirstamp` (memoir.cls:5454) does not start with `\if`.
const CLASS_LOAD_WARNING: &str = r"Warning:misdefined:\@iffirstamp";

/// The `<li>` of the ToC entry of `kind` (`chapter`, `section`, ...) that mentions `title`.
fn toc_entry<'a>(html: &'a str, kind: &str, title: &str) -> Option<&'a str> {
  let open = format!("<li class=\"ltx_tocentry ltx_tocentry_{kind}\">");
  html
    .split(&open)
    .skip(1)
    .map(|rest| rest.split("</li>").next().unwrap_or_default())
    .find(|entry| entry.contains(title))
}

fn count_entries(html: &str, kind: &str) -> usize {
  html
    .matches(&format!("<li class=\"ltx_tocentry ltx_tocentry_{kind}\">"))
    .count()
}

const DEPTHS: &str = "\\documentclass{memoir}\n\\setsecnumdepth{subsection}\n\\settocdepth{subsection}\n\
                      \\begin{document}\n\\tableofcontents\n\\chapter{First}\n\\section{Alpha}\n\\subsection{Detail}\n\
                      \\subsubsection{Deeper}\nBody.\n\\end{document}\n";

/// `\setsecnumdepth{subsection}` numbers down to subsections and `\settocdepth{subsection}` lists them: memoir's
/// default ToC stops at sections (memoir.cls:12126), and the `\addtocontents{toc}{\changetocdepth{2}}` memoir writes
/// is a no-op here, so the preamble call did nothing.
#[test]
fn settocdepth_and_setsecnumdepth_reach_the_document() {
  let (stderr, xml, html) = convert_xml_then_html(DEPTHS);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_heads(&stderr), [CLASS_LOAD_WARNING], "{stderr}");
  assert_eq!(count_entries(&html, "chapter"), 1, "{html}");
  assert_eq!(count_entries(&html, "section"), 1, "{html}");
  assert_eq!(
    count_entries(&html, "subsection"),
    1,
    "the subsection is listed:\n{html}"
  );
  assert_eq!(
    count_entries(&html, "subsubsection"),
    0,
    "its child is not:\n{html}"
  );
  assert!(
    xml.contains("<tag>1.1.1</tag>") || xml.contains(">1.1.1</tag>"),
    "subsections are numbered:\n{xml}"
  );
}

const SHALLOW: &str = "\\documentclass{memoir}\n\\settocdepth{chapter}\n\\setsecnumdepth{chapter}\n\
                       \\begin{document}\n\\tableofcontents\n\\chapter{First}\n\\section{Alpha}\nBody.\n\\end{document}\n";

/// The same commands lower the depths: chapters only in the ToC, and no section number.
#[test]
fn division_names_set_the_levels() {
  let (stderr, xml, html) = convert_xml_then_html(SHALLOW);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(count_entries(&html, "chapter"), 1, "{html}");
  assert_eq!(count_entries(&html, "section"), 0, "{html}");
  assert!(
    !xml.contains(">1.1</tag>"),
    "sections are not numbered:\n{xml}"
  );
}

const UNKNOWN: &str = "\\documentclass{memoir}\n\\setsecnumdepth{bogus}\n\\settocdepth{nonsense}\n\
                       \\begin{document}\n\\chapter{First}\nBody.\n\\end{document}\n";

/// An unknown division name is memoir's error, one per command (memoir.cls:7557-7563, 7649-7655), and changes
/// nothing.
#[test]
fn unknown_division_name_is_memoirs_error() {
  let (stderr, _xml, _html) = convert_xml_then_html(UNKNOWN);
  assert_eq!(error_count(&stderr), 2, "{stderr}");
  assert!(
    stderr.contains("Unknown document division name (bogus)"),
    "{stderr}"
  );
  assert!(
    stderr.contains("Unknown document division name (nonsense)"),
    "{stderr}"
  );
}

const FONT_DOC: &str = "\\documentclass{memoir}\n\\begin{document}\n\\tableofcontents\n\\chapter{First}\n\\section{Alpha}\n\
                        Body.\n\\end{document}\n";

/// memoir's own `\cftchapterfont` is `\bfseries` (memoir.cls:7433) and the deeper levels `\normalfont`: the chapter
/// ToC entry is bold, the section entry carries no font.
#[test]
fn toc_chapter_entries_are_bold_by_default() {
  let (stderr, _xml, html) = convert_xml_then_html(FONT_DOC);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_heads(&stderr), [CLASS_LOAD_WARNING], "{stderr}");
  let chapter = toc_entry(&html, "chapter", "First").expect("a chapter entry");
  assert!(
    chapter.contains("<span class=\"ltx_text ltx_font_bold\">First</span>"),
    "{chapter}"
  );
  let section = toc_entry(&html, "section", "Alpha").expect("a section entry");
  assert!(!section.contains("ltx_font_"), "{section}");
}

const FONT_RENEWED: &str = "\\documentclass{memoir}\n\\renewcommand\\cftchapterfont{\\hfill\\sffamily}\n\
                            \\begin{document}\n\\tableofcontents\n\\chapter{First}\nBody.\n\\end{document}\n";

/// A renewed `\cftchapterfont` is the entry's font, and the `\hfill` the author put in it to place the entry on a
/// line is dropped: neither a stray space nor a fill reaches the title.
#[test]
fn renewed_cft_font_reaches_the_toc_entry() {
  let (stderr, _xml, html) = convert_xml_then_html(FONT_RENEWED);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_heads(&stderr), [CLASS_LOAD_WARNING], "{stderr}");
  let chapter = toc_entry(&html, "chapter", "First").expect("a chapter entry");
  assert!(
    chapter.contains("<span class=\"ltx_tag ltx_tag_ref\">1 </span><span class=\"ltx_text ltx_font_sansserif\">First</span>"),
    "{chapter}"
  );
}

const SECTION_FONTS: &str = "\\documentclass{memoir}\n\\setsecnumdepth{subsection}\n\\settocdepth{subsection}\n\
                             \\renewcommand*{\\cftsectionfont}{\\hfill\\bfseries}\n\
                             \\renewcommand*{\\cftsubsectionfont}{\\itshape}\n\
                             \\begin{document}\n\\tableofcontents\n\\chapter{First}\n\\section{Alpha}\n\
                             \\subsection{Detail}\nBody.\n\\end{document}\n";

/// The same for the numbered levels below the chapter. Their ToC title reads like the heading's, so it reaches the ToC
/// only because `\@@numbered@section` (sect04.rs) keeps a toctitle whose `\format@toctitle@font@<type>` is defined.
#[test]
fn renewed_cft_font_reaches_section_entries() {
  let (stderr, _xml, html) = convert_xml_then_html(SECTION_FONTS);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  let section = toc_entry(&html, "section", "Alpha").expect("a section entry");
  assert!(
    section.contains("<span class=\"ltx_text ltx_font_bold\">Alpha</span>"),
    "{section}"
  );
  let subsection = toc_entry(&html, "subsection", "Detail").expect("a subsection entry");
  assert!(
    subsection.contains("<span class=\"ltx_text ltx_font_italic\">Detail</span>"),
    "{subsection}"
  );
}

const LISTS: &str = "\\documentclass{memoir}\n\\begin{document}\n\\tableofcontents*\n\\listoffigures\n\\listoftables*\n\
                     \\chapter{Floats}\n\\begin{figure}\\centering F\\caption{The figure}\\end{figure}\n\
                     \\begin{table}\\centering\\begin{tabular}{c}x\\end{tabular}\\caption{The table}\\end{table}\n\
                     \\end{document}\n";

/// The three lists take memoir's optional star and are LaTeXML's lists, with nothing of the star left in the text.
#[test]
fn lists_take_a_star_and_are_latexml_tocs() {
  let (stderr, xml, html) = convert_xml_then_html(LISTS);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_heads(&stderr), [CLASS_LOAD_WARNING], "{stderr}");
  for lists in ["toc", "lof", "lot"] {
    assert_eq!(
      xml.matches(&format!("<TOC lists=\"{lists}\"")).count(),
      1,
      "{lists}:\n{xml}"
    );
  }
  assert!(
    !xml.contains("*\n") && !xml.contains(">*<"),
    "no star in the text:\n{xml}"
  );
  assert!(toc_entry(&html, "figure", "The figure").is_some(), "{html}");
  assert!(toc_entry(&html, "table", "The table").is_some(), "{html}");
}

/// Layouts and the lengths pdfLaTeX computes for them at `\begin{document}` (memoir.cls, pdfLaTeX of TeX Live 2026, the
/// default `letterpaper,10pt,twoside` class): the arithmetic of `\setfillsize` for each way of leaving one of the
/// three parts open (`*`) or given as a ratio, the `\binding` adjustment, a length expression as an argument, and the
/// four height algorithms of `\checkthelayout`.
const LAYOUTS: &[(&str, &str, &[&str])] = &[
  // the notes' own call
  (
    r"\setulmarginsandblock{3.5cm}{3.5cm}{*}\setlrmarginsandblock{3cm}{3.5cm}{*}\checkandfixthelayout",
    "textwidth=429.0pt; textheight=598.0pt; spinemargin=85.35826pt; foremargin=99.58464pt; uppermargin=99.58464pt; \
     lowermargin=97.38535pt; oddsidemargin=13.0pt; evensidemargin=27.0pt; topmargin=-2.75284pt; columnwidth=429.0pt;",
    &[],
  ),
  // a ratio for the open margin, `lines`
  (
    r"\setlrmarginsandblock{3cm}{*}{1.5}\setulmarginsandblock{*}{2in}{0.75}\checkandfixthelayout[lines]",
    "textwidth=400.0pt; textheight=538.0pt; spinemargin=85.35826pt; foremargin=128.03738pt; uppermargin=108.40498pt; \
     lowermargin=148.565pt; oddsidemargin=13.0pt; evensidemargin=55.0pt; topmargin=6.0675pt; columnwidth=400.0pt;",
    &[],
  ),
  // the left margin as a ratio of the right one, `nearest`
  (
    r"\setlrmarginsandblock{*}{4cm}{0.6}\setulmarginsandblock{2cm}{*}{*}\checkandfixthelayout[nearest]",
    "textwidth=432.0pt; textheight=682.0pt; spinemargin=68.2873pt; foremargin=113.81102pt; uppermargin=56.9055pt; \
     lowermargin=56.06448pt; oddsidemargin=-3.0pt; evensidemargin=41.0pt; topmargin=-45.43198pt; columnwidth=432.0pt;",
    &[],
  ),
  // the block given and both margins open: (T - C)/(1 + r), then both open and no ratio: half each; `fixed`
  (
    r"\setlrmargins{*}{*}{1.618}\setulmargins{*}{*}{*}\checkandfixthelayout[fixed]",
    "textwidth=341.0pt; textheight=550.0pt; spinemargin=104.4159pt; foremargin=168.8791pt; uppermargin=122.48499pt; \
     lowermargin=122.485pt; oddsidemargin=32.0pt; evensidemargin=96.0pt; topmargin=20.1475pt; columnwidth=341.0pt;",
    &[],
  ),
  // a binding, taken off the paper width while the margins are set
  (
    r"\setbinding{1cm}\setlrmarginsandblock{2cm}{3cm}{*}\setulmarginsandblock{1.25in}{2.5in}{*}\checkandfixthelayout[classic]",
    "textwidth=443.0pt; textheight=526.0pt; spinemargin=85.35825pt; foremargin=85.35826pt; uppermargin=90.3375pt; \
     lowermargin=178.63249pt; oddsidemargin=13.0pt; evensidemargin=13.0pt; topmargin=-11.99998pt; columnwidth=443.0pt;",
    &[],
  ),
  // lengths written as expressions
  (
    r"\setlength{\unitlength}{7.3pt}\setlrmarginsandblock{2\unitlength}{*}{1.3333}\setulmarginsandblock{3\unitlength}{3.3\unitlength}{*}\checkandfixthelayout",
    "textwidth=580.0pt; textheight=754.0pt; spinemargin=14.6pt; foremargin=19.46614pt; uppermargin=21.90001pt; \
     lowermargin=19.06998pt; oddsidemargin=-57.0pt; evensidemargin=-52.0pt; topmargin=-80.43747pt; columnwidth=580.0pt;",
    &[
      "\\headheight (12.0pt) and/or \\headsep (18.06749pt) are too large for \\uppermargin (21.90001pt) by 8.16748pt",
      "\\footskip (25.29494pt) is too large for \\lowermargin (19.06998pt) by 6.22496pt",
    ],
  ),
];

/// `\setlrmarginsandblock`, `\setulmarginsandblock` and `\checkandfixthelayout` leave the lengths pdfLaTeX computes
/// (the last layout is too tight for its header and footer, and draws the two overflow errors pdfLaTeX draws, with
/// their numbers).
#[test]
fn page_layout_lengths_equal_pdflatexs() {
  for (setup, expected, errors) in LAYOUTS {
    let tex = format!(
      "\\documentclass{{memoir}}\n{setup}\n\\begin{{document}}\n\\chapter{{A}}\n\
       textwidth=\\the\\textwidth; textheight=\\the\\textheight; spinemargin=\\the\\spinemargin; \
       foremargin=\\the\\foremargin; uppermargin=\\the\\uppermargin; lowermargin=\\the\\lowermargin; \
       oddsidemargin=\\the\\oddsidemargin; evensidemargin=\\the\\evensidemargin; topmargin=\\the\\topmargin; \
       columnwidth=\\the\\columnwidth;\n\\end{{document}}\n"
    );
    let (stderr, xml, _html) = convert_xml_then_html(&tex);
    assert_eq!(error_count(&stderr), errors.len(), "{setup}\n{stderr}");
    for message in *errors {
      assert!(
        stderr.contains(message),
        "{setup}\nwanted the error: {message}\n{stderr}"
      );
    }
    assert_eq!(
      warning_heads(&stderr),
      [CLASS_LOAD_WARNING],
      "{setup}\n{stderr}"
    );
    assert!(xml.contains(expected), "{setup}\nwanted: {expected}\n{xml}");
  }
}

/// The combination memoir cannot resolve (all three open), an unknown height algorithm and a layout too wide for its
/// paper (no text width left, for each check) are memoir's own messages, as pdfLaTeX prints them.
#[test]
fn page_layout_misuse_is_memoirs_report() {
  let tex = "\\documentclass{memoir}\n\\setlrmarginsandblock{*}{*}{*}\n\\checkandfixthelayout[bogus]\n\
             \\setlrmarginsandblock{12cm}{12cm}{*}\n\\checkandfixthelayout\n\\begin{document}\nBody.\n\\end{document}\n";
  let (stderr, _xml, _html) = convert_xml_then_html(tex);
  assert_eq!(error_count(&stderr), 2, "{stderr}");
  assert!(
    stderr.contains("The combination of argument values is ambiguous."),
    "{stderr}"
  );
  assert!(
    stderr.contains("Optional argument is not one of:"),
    "{stderr}"
  );
  assert_eq!(
    warning_heads(&stderr),
    [
      CLASS_LOAD_WARNING,
      "Warning:latex:(memoir)",
      "Warning:latex:(memoir)"
    ],
    "one non-positive-width warning per check, as pdfLaTeX (memoir.cls:1103):\n{stderr}"
  );
  assert!(
    stderr.contains("\\textwidth is zero or negative"),
    "{stderr}"
  );
}
