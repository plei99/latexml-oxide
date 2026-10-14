//! memoir's lists are LaTeXML's: `\tableofcontents` (and its starred form, `\listoffigures`, `\listoftables`) is an
//! `ltx:TOC` built in post-processing, not memoir's `\@starttoc`, which read a stale `\jobname.toc` beside the source
//! and printed each `\contentsline`'s hyperref destination as text. And memoir is styled as a book (`ltx-book.css`).

const TEX: &str = "\\documentclass{memoir}\n\\usepackage{hyperref}\n\\begin{document}\n\\tableofcontents\n\
                   \\tableofcontents*\n\\listoffigures\n\\chapter{One}\n\\section{Sec}\nText.\n\\end{document}\n";

/// A `.toc` as pdflatex with hyperref leaves it.
const TOC: &str = "\\contentsline {chapter}{\\chapternumberline {1}One}{1}{chapter.1}%\n\
                   \\contentsline {section}{\\numberline {1.1}Sec}{1}{section.1.1}%\n";

#[test]
fn memoir_lists_are_latexml_tocs() {
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("m.tex"), TEX).expect("write m.tex");
  std::fs::write(workdir.path().join("m.toc"), TOC).expect("write m.toc");
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
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  let xml = std::fs::read_to_string(workdir.path().join("m.xml")).expect("read m.xml");
  assert_eq!(
    xml.matches("<TOC lists=\"toc\"").count(),
    2,
    "both forms are a TOC:\n{xml}"
  );
  assert_eq!(
    xml.matches("<TOC lists=\"lof\"").count(),
    1,
    "\\listoffigures is a TOC:\n{xml}"
  );
  assert!(!xml.contains("chapter.1"), "the .toc is not read:\n{xml}");
  assert!(!xml.contains("*\n"), "the star is taken:\n{xml}");
  assert!(
    xml.contains("<resource src=\"ltx-book.css\""),
    "memoir is styled as a book:\n{xml}"
  );
}
