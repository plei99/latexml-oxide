//! amsart, amsproc and amsbook number their parts with `\arabic{part}` (amsart.cls:1098,
//! amsproc.cls:1033, amsbook.cls:878), where article, report and book use `\Roman{part}`. Perl's
//! `amsart.cls.ltxml` loads the article binding and does not renew `\thepart`, so a document headed
//! `\part{...}` read "Part I" in the heading, and "I" in the contents and in a `\ref`, where the
//! PDF reads "Part 1" and "1". amsart and amsproc also close the number of a heading with a full
//! stop (`\@secnumpunct`, the `\edef` at amsart.cls:1059-1065 inside `\@sect`, :1050-1088;
//! `\@seccntformat`, :1043-1048; `\part`, :1106-1108), "Part 1. Title", as they do for sections.

/// The `<part>` element of the unit titled `title`, from its opening tag to its `</toctitle>`.
fn part_head<'a>(xml: &'a str, title: &str) -> &'a str {
  let end = xml
    .find(&format!("{title}</toctitle>"))
    .unwrap_or_else(|| panic!("no part titled {title}:\n{xml}"));
  let start = xml[..end].rfind("<part ").expect("the part");
  &xml[start..end + title.len() + "</toctitle>".len()]
}

const ARTICLE_LIKE: &str = "\\begin{document}\n\
  \\part{First}\\label{p:first}\n\
  \\section{Sec}\n\
  \\part{Second}\n\
  See \\ref{p:first}.\n\
  \\end{document}\n";

fn assert_arabic_parts(class: &str, close: &str) {
  let (stderr, xml) = super::convert(
    &format!("\\documentclass{{{class}}}\n{ARTICLE_LIKE}"),
    false,
  );
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  assert_eq!(
    super::perfect_kernel_batch46::warning_count(&stderr),
    0,
    "{stderr}"
  );
  let first = part_head(&xml, "First");
  assert!(
    first.contains(&format!(
      "<tags>\n      <tag>Part 1</tag>\n      <tag role=\"refnum\">1</tag>\n      <tag role=\"typerefnum\">Part 1</tag>\n    </tags>\n    <title><tag close=\"{close}\">Part 1</tag>First</title>\n    <toctitle><tag close=\" \">1</tag>First</toctitle>"
    )),
    "the first part of {class}:\n{first}"
  );
  let second = part_head(&xml, "Second");
  assert!(
    second.contains("<tag role=\"refnum\">2</tag>")
      && second.contains(&format!(
        "<title><tag close=\"{close}\">Part 2</tag>Second</title>"
      )),
    "the second part of {class}:\n{second}"
  );
  // A reference to a part shows its number as the PDF does ("see 1", the contents line's number).
  let (html, post_log) = super::svg_nested_picture::post_html_logged(&xml);
  assert_eq!(
    super::perfect_kernel_batch46::warning_count(&post_log),
    0,
    "{post_log}"
  );
  assert!(
    html.contains(r##"See <span class="ltx_ref ltx_ref_self"><span class="ltx_text ltx_ref_tag">1</span></span>."##),
    "the reference to the part:\n{html}"
  );
}

#[test]
fn amsart_parts_are_arabic() { assert_arabic_parts("amsart", ". "); }

#[test]
fn amsproc_parts_are_arabic() { assert_arabic_parts("amsproc", ". "); }

#[test]
fn amsbook_parts_are_arabic() {
  let (stderr, xml) = super::convert(
    "\\documentclass{amsbook}\n\\begin{document}\n\\part{First}\n\\chapter{Chap}\n\\part{Second}\n\\end{document}\n",
    false,
  );
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  assert_eq!(
    super::perfect_kernel_batch46::warning_count(&stderr),
    0,
    "{stderr}"
  );
  // amsbook's `\part` is its own page (amsbook.cls:890-907): the tag has no full stop.
  assert!(
    part_head(&xml, "First").contains(r#"<title><tag close=" ">Part 1</tag>First</title>"#),
    "{xml}"
  );
  assert!(
    part_head(&xml, "Second").contains("<tag role=\"refnum\">2</tag>"),
    "{xml}"
  );
}

/// Classes that do not renew `\thepart` keep the roman numeral.
#[test]
fn article_parts_stay_roman() {
  let (stderr, xml) = super::convert(
    "\\documentclass{article}\n\\begin{document}\n\\part{First}\n\\section{Sec}\n\\end{document}\n",
    false,
  );
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  assert_eq!(
    super::perfect_kernel_batch46::warning_count(&stderr),
    0,
    "{stderr}"
  );
  assert!(
    part_head(&xml, "First").contains("<tag role=\"refnum\">I</tag>"),
    "{xml}"
  );
}
