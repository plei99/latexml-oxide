//! Guards for `latexml_contrib/src/memoir_fork/chapters.rs` (fork-only memoir bindings).
//!
//! LaTeXML builds a chapter's head itself, so memoir's `\@makechapterhead` (memoir.cls:3092) and the hooks of
//! `\chapterstyle` never ran: every chapter was headed "Chapter 1 Title", appendices "Chapter A". The tag before the
//! title now follows the style (the table in chapters.rs), the unit carries `ltx_chapterstyle_<name>`, and
//! `\appendix`, `\frontmatter`, `\mainmatter` and `\backmatter` are native. Cross-references keep the plain number.

/// Convert `tex` as the corpus does (`[rawstyles,rawclasses]` preload) and return stderr and the XML.
fn convert_memoir(tex: &str) -> (String, String) { convert_memoir_with(tex, &[]) }

/// The same with `preloads` (file name, contents) written beside `m.tex` and preloaded before memoir loads.
fn convert_memoir_with(tex: &str, preloads: &[(&str, &str)]) -> (String, String) {
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("m.tex"), tex).expect("write m.tex");
  let mut preload_args = Vec::new();
  for (name, contents) in preloads {
    std::fs::write(workdir.path().join(name), contents).expect("write preload");
    preload_args.push(format!("--preload={name}"));
  }
  let output = std::process::Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args([
      "m.tex",
      "--dest",
      "m.xml",
      "--timeout=110",
      "--preload=[rawstyles,rawclasses]latexml.sty",
    ])
    .args(&preload_args)
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  let xml = std::fs::read_to_string(workdir.path().join("m.xml")).expect("read m.xml");
  (stderr, xml)
}

/// No errors, and the one warning every memoir document carries: `\newif\@iffirstamp` (memoir.cls:5454), which
/// does not start with `\if` (roadmap M0 removes it).
fn assert_only_the_memoir_warning(stderr: &str) {
  assert_eq!(super::error_count(stderr), 0, "{stderr}");
  let warnings: Vec<&str> = stderr
    .lines()
    .filter(|l| l.starts_with("Warning:"))
    .collect();
  assert_eq!(warnings.len(), 1, "{stderr}");
  assert!(
    warnings[0].starts_with("Warning:misdefined:\\@iffirstamp"),
    "{stderr}"
  );
}

/// The `<title>` of the unit whose title text is `title`, and the unit's opening tag.
fn head<'a>(xml: &'a str, title: &str) -> (&'a str, &'a str) {
  let end = xml
    .find(&format!("{title}</title>"))
    .unwrap_or_else(|| panic!("no title {title}:\n{xml}"));
  let title_start = xml[..end].rfind("<title>").expect("<title>");
  let unit_start = xml[..title_start]
    .rfind("<chapter ")
    .max(xml[..title_start].rfind("<appendix "))
    .expect("the unit of the title");
  let unit_end = unit_start + xml[unit_start..].find('>').expect(">");
  (
    &xml[unit_start..=unit_end],
    &xml[title_start..end + title.len() + "</title>".len()],
  )
}

/// The element kind (`chapter` or `appendix`) of the unit that [`head`] found.
fn unit_kind(unit: &str) -> &str {
  unit
    .trim_start_matches('<')
    .split_whitespace()
    .next()
    .unwrap_or_default()
}

fn assert_head(xml: &str, title: &str, tag: &str, class: &str) {
  let (unit, title_element) = head(xml, title);
  assert_eq!(
    title_element,
    format!("<title><tag close=\" \">{tag}</tag>{title}</title>"),
    "the tag of {title}:\n{xml}"
  );
  assert!(
    unit.contains(&format!("class=\"ltx_chapterstyle_{class}\"")),
    "the class of {title}: {unit}"
  );
}

fn assert_untagged_head(xml: &str, title: &str, class: &str) {
  let (unit, title_element) = head(xml, title);
  assert_eq!(title_element, format!("<title>{title}</title>"), "{xml}");
  assert!(
    unit.contains(&format!("class=\"ltx_chapterstyle_{class}\"")),
    "the class of {title}: {unit}"
  );
}

/// The refnum and typerefnum tags of the unit titled `title`: what `\ref` and `\cref` show.
fn assert_refs(xml: &str, title: &str, refnum: &str, typerefnum: &str) {
  let (unit, _) = head(xml, title);
  let at = xml.find(unit).expect("unit") + unit.len();
  let tags = &xml[at..xml[at..].find("<title>").map(|i| at + i).expect("title")];
  assert!(
    tags.contains(&format!("<tag role=\"refnum\">{refnum}</tag>")),
    "refnum of {title}: {tags}"
  );
  assert!(
    tags.contains(&format!("<tag role=\"typerefnum\">{typerefnum}</tag>")),
    "typerefnum of {title}: {tags}"
  );
}

/// The `<toctitle>` of the unit titled `title` is the number and the title, both in `\cftchapterfont`'s bold: the
/// contents entry carries `\thechapter` (`\addcontentsline{toc}{chapter}{\chapternumberline{\thechapter}...}`,
/// memoir.cls:3065-3066), which `\@chapapp` and the chapter style do not touch, where the head prints what the style
/// prints ("Three" under `demo2`). A consumer of the XML that wants the contents' number reads it here.
fn assert_toc_number(xml: &str, title: &str, number: &str) {
  let toctitle = format!(
    "<toctitle><tag close=\" \"><text font=\"bold\">{number}</text></tag><text font=\"bold\">{title}</text></toctitle>"
  );
  assert!(
    xml.contains(&toctitle),
    "the contents line of {title}:\n{xml}"
  );
}

/// Every built-in style keeps the number a reference shows, so the references are checked once, here: chapter 3 is
/// headed "Three" under `demo2` but referred to as 3 and "Chapter 3".
const STYLES: &str = "\\documentclass{memoir}\n\\begin{document}\n\
  \\chapter{Alpha}\n\
  \\chapterstyle{ell}\n\\chapter{Beta}\n\
  \\chapterstyle{demo2}\n\\chapter{Gamma}\\label{g}\nSee \\ref{g}.\n\
  \\chapterstyle{brotherton}\n\\chapter{Delta}\n\
  \\chapterstyle{article}\n\\chapter{Epsilon}\n\
  \\begingroup\\chapterstyle{ell}\\chapter{Zeta}\\endgroup\n\
  \\chapter{Eta}\n\
  \\chapter*{Theta}\n\
  \\makechapterstyle{mine}{\\chapterstyle{default}\\renewcommand*{\\chaptitlefont}{\\normalfont\\itshape}}\n\
  \\chapterstyle{mine}\n\\chapter{Iota}\n\
  \\chapterstyle{nosuchstyle}\n\\chapter{Kappa}\n\
  \\chapterstyle{demo2}\n\\appendix\n\\chapter{Lambda}\n\
  \\end{document}\n";

#[test]
fn chapter_heads_follow_the_chapterstyle() {
  let (stderr, xml) = convert_memoir(STYLES);
  assert_only_the_memoir_warning(&stderr);
  // The initial style is `default`: "Chapter 1".
  assert_head(&xml, "Alpha", "Chapter 1", "default");
  // `ell` (memoir.cls:3390): the number alone.
  assert_head(&xml, "Beta", "2", "ell");
  // `demo2` (:3340): the number as a word. References keep the plain number and the typeref name.
  assert_head(&xml, "Gamma", "Three", "demo2");
  assert_refs(&xml, "Gamma", "3", "Chapter 3");
  assert_toc_number(&xml, "Gamma", "3");
  // `brotherton` (:3295): the name and the number as a word.
  assert_head(&xml, "Delta", "Chapter Four", "brotherton");
  assert_toc_number(&xml, "Delta", "4");
  // `article` (:3204): the number alone.
  assert_head(&xml, "Epsilon", "5", "article");
  // A group restores the style that was in force, class included.
  assert_head(&xml, "Zeta", "6", "ell");
  assert_head(&xml, "Eta", "7", "article");
  // An unnumbered chapter has no tag, and still carries the class.
  assert_untagged_head(&xml, "Theta", "article");
  // A style the document defines is not in the table: the default name and number, under its own class.
  assert_head(&xml, "Iota", "Chapter 8", "mine");
  // An undefined style changes nothing, as memoir's `\@nameuse` changes nothing.
  assert_head(&xml, "Kappa", "Chapter 9", "mine");
  // `demo2` spells the chapters out but letters the appendix (`\ifanappendix \thechapter`, :3344).
  assert_head(&xml, "Lambda", "A", "demo2");
  assert!(head(&xml, "Lambda").0.starts_with("<appendix "), "{xml}");
  // The look is a fork resource, served from the binary.
  assert!(
    xml.contains("<resource src=\"ltx-memoir.css\""),
    "ltx-memoir.css is required:\n{xml}"
  );
}

const APPENDIX: &str = "\\documentclass{memoir}\n\\begin{document}\n\
  \\frontmatter\n\\chapter{Preface}\n\\chapter*{Foreword}\n\
  \\begin{figure}\\caption{front}\\end{figure}\n\
  \\mainmatter\n\\chapter{First}\n\\begin{figure}\\caption{main}\\end{figure}\n\\section{Sect}\n\
  \\appendix\n\\chapter{Extra}\\label{x}\nSee Appendix~\\ref{x}.\n\\section{Sub}\n\
  \\begin{figure}\\caption{app}\\end{figure}\n\
  \\chapter{More}\n\
  \\backmatter\n\\chapter{Afterword}\n\\begin{figure}\\caption{back}\\end{figure}\n\
  \\end{document}\n";

#[test]
fn appendix_and_matter() {
  let (stderr, xml) = convert_memoir(APPENDIX);
  assert_only_the_memoir_warning(&stderr);
  // memoir.cls:2723 `\frontmatter`: `secnumdepth` -10, so a numbered `\chapter` is not numbered.
  assert_untagged_head(&xml, "Preface", "default");
  assert_untagged_head(&xml, "Foreword", "default");
  // `\mainmatter` (:2738) numbers again, from 1.
  assert_head(&xml, "First", "Chapter 1", "default");
  // `\appendix` (:4297) makes appendix units, named by `\appendixname` and lettered; the chapter section counter
  // restarts, and the references keep the letter.
  assert!(head(&xml, "Extra").0.starts_with("<appendix "), "{xml}");
  assert_head(&xml, "Extra", "Appendix A", "default");
  assert_refs(&xml, "Extra", "A", "Appendix A");
  assert_head(&xml, "More", "Appendix B", "default");
  assert!(
    xml.contains("<tag close=\" \">A.1</tag>Sub"),
    "the appendix's section is A.1:\n{xml}"
  );
  // The floats count per chapter in the main matter only (`\@memfront@floats` :2726, `\@memmain@floats` :2740,
  // `\@memback@floats` :2760), a plain count in front and back matter.
  for (caption, number) in [
    ("front", "1"),
    ("main", "1.1"),
    ("app", "A.1"),
    ("back", "1"),
  ] {
    assert!(
      xml.contains(&format!(
        "<caption><tag close=\": \">Figure\u{a0}{number}</tag>{caption}</caption>"
      )),
      "the {caption} figure is Figure {number}:\n{xml}"
    );
  }
  // `\backmatter` ends the appendices (:2766): the Afterword is a chapter again, which memoir writes to the ToC as
  // `chapter` (:3071). The next test checks the ToC.
  assert_untagged_head(&xml, "Afterword", "default");
  assert_eq!(
    unit_kind(head(&xml, "Afterword").0),
    "chapter",
    "the back matter chapter is not an appendix:\n{xml}"
  );
  assert_eq!(unit_kind(head(&xml, "More").0), "appendix", "{xml}");
}

/// A chapter in the back matter after `\appendix` is a chapter in the document and a `chapter` entry in the ToC
/// (memoir.cls:3071, an unnumbered chapter; the appendix kind is the numbered chapter under `\ifanappendix`, :3061),
/// while the lettered appendices before it keep `ltx:appendix` and their ToC kind.
#[test]
fn backmatter_after_appendix_is_a_chapter() {
  let (stderr, xml, html) = super::memoir_fork_layout::convert_xml_then_html(
    "\\documentclass{memoir}\n\\begin{document}\n\\tableofcontents\n\\mainmatter\n\\chapter{First}\n\
     \\appendix\n\\chapter{Extra}\n\\backmatter\n\\chapter{Afterword}\n\\end{document}\n",
  );
  assert_only_the_memoir_warning(&stderr);
  assert_eq!(unit_kind(head(&xml, "Extra").0), "appendix", "{xml}");
  assert_eq!(unit_kind(head(&xml, "Afterword").0), "chapter", "{xml}");
  let kinds: Vec<&str> = html
    .match_indices("<li class=\"ltx_tocentry ltx_tocentry_")
    .map(|(at, open)| {
      let rest = &html[at + open.len()..];
      &rest[..rest.find('"').expect("closing quote")]
    })
    .collect();
  assert_eq!(kinds, ["chapter", "appendix", "chapter"], "{html}");
  assert!(
    html.contains("ltx_tocentry_chapter\"><a href=\"#chapterx1\""),
    "the Afterword entry is a chapter entry:\n{html}"
  );
}

#[test]
fn ell_appendix_is_the_letter() {
  let (stderr, xml) = convert_memoir(
    "\\documentclass{memoir}\n\\begin{document}\n\\chapterstyle{ell}\n\\chapter{Main}\n\\appendix\n\
     \\chapter{Extra}\n\\end{document}\n",
  );
  assert_only_the_memoir_warning(&stderr);
  assert_head(&xml, "Main", "1", "ell");
  assert!(head(&xml, "Extra").0.starts_with("<appendix "), "{xml}");
  assert_head(&xml, "Extra", "A", "ell");
  assert_refs(&xml, "Extra", "A", "Appendix A");
}

#[test]
fn article_option_heads_have_no_chapter_name() {
  // memoir.cls:580-582, :12167: the `article` option runs `\chapterstyle{article}` while the class loads.
  let (stderr, xml) = convert_memoir(
    "\\documentclass[article]{memoir}\n\\begin{document}\n\\chapter{Introduction}\n\\chapter{Method}\n\
     \\appendix\n\\chapter{Proofs}\n\\end{document}\n",
  );
  assert_only_the_memoir_warning(&stderr);
  assert_head(&xml, "Introduction", "1", "article");
  assert_head(&xml, "Method", "2", "article");
  assert_head(&xml, "Proofs", "A", "article");
}

/// What the tag can carry of a style's punctuation, from memoir.cls: `verville` prints `\thechapter.\ ` (:3555), "1.
/// Title" (and "A. Title" for an appendix), `dash` a rule, the number and a rule (:3331-3332), read as "- 1 -"
/// with en dashes, and `culver` renews `\thechapter` as `\Roman{chapter}` (:3316), which the number follows. pdfLaTeX
/// prints "1. Title" and "XIII" for the first and third.
#[test]
fn style_punctuation_and_numerals_are_in_the_tag() {
  let (stderr, xml) = convert_memoir(
    "\\documentclass{memoir}\n\\begin{document}\n\
     \\chapterstyle{dash}\n\\chapter{Dsh}\n\
     \\chapterstyle{verville}\n\\chapter{Vrv}\n\
     \\chapterstyle{culver}\n\\chapter{Clv}\n\
     \\appendix\n\\chapterstyle{verville}\n\\chapter{AppV}\n\
     \\chapterstyle{dash}\n\\chapter{AppD}\n\\end{document}\n",
  );
  assert_only_the_memoir_warning(&stderr);
  assert_head(&xml, "Dsh", "\u{2013} 1 \u{2013}", "dash");
  assert_head(&xml, "Vrv", "2.", "verville");
  assert_head(&xml, "Clv", "III", "culver");
  assert_head(&xml, "AppV", "A.", "verville");
  assert_head(&xml, "AppD", "\u{2013} B \u{2013}", "dash");
  // The references keep the plain number.
  assert_refs(&xml, "Vrv", "2", "Chapter 2");
}

/// A hook another binding defined before memoir loads (`\lx@startsection@afterhook`, as svmult's `\motto`) is called
/// after the style's class is put on the unit, and not replaced by it.
#[test]
fn another_bindings_afterhook_is_kept() {
  let (stderr, xml) = convert_memoir_with(
    "\\documentclass{memoir}\n\\begin{document}\n\\chapterstyle{ell}\n\\chapter{First}\nBody.\n\\end{document}\n",
    &[(
      "pre.sty",
      "\\ProvidesPackage{pre}\n\\makeatletter\n\\def\\lx@startsection@afterhook#1#2{\\par\\textit{Hook:#1:#2}\\par}\n\
       \\makeatother\n",
    )],
  );
  assert_only_the_memoir_warning(&stderr);
  assert_head(&xml, "First", "1", "ell");
  assert!(
    xml.contains("<p><text font=\"italic\">Hook:chapter:toc</text></p>"),
    "the earlier hook ran, inside the chapter:\n{xml}"
  );
}
