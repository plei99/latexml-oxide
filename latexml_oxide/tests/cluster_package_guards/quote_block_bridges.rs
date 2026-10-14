//! A theorem or a sectioning unit inside a `quote` (a list in TeX, article.cls:405-408) is placed in a block of the
//! quote — an `ltx:logical-block`, an `ltx:sectional-block` — instead of "isn't allowed in <ltx:quote>" (KNOWN_PERL_ERRORS
//! #664; OXIDIZED_DESIGN_DIVERGENCES #271): an author's theorem in a quote, moduli.cls's `\tableofcontents` that wraps
//! its `\section*` in one. The quote stays whole and the text after it stays outside.

#[test]
fn theorem_in_a_quote_opens_a_logical_block() {
  let tex = "\\documentclass{article}\n\\newtheorem{thm}{Theorem}\n\\begin{document}\n\\begin{quote}\n\
             \\begin{thm}Inner.\\end{thm}\nAfter.\n\\end{quote}\nOutside.\n\\end{document}\n";
  let (stderr, xml) = super::convert(tex, true);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  let quote = &xml[xml.find("<quote").expect("a quote")..xml.find("</quote>").expect("its end")];
  assert!(
    quote.contains("<logical-block") && quote.contains("<theorem"),
    "{xml}"
  );
  assert!(
    quote.contains("After."),
    "the text after the theorem stays in the quote:\n{xml}"
  );
  assert!(
    xml[xml.find("</quote>").unwrap()..].contains("Outside."),
    "{xml}"
  );
}

#[test]
fn section_in_a_quote_opens_a_sectional_block() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\\begin{quote}\n\\section*{Contents}\nText.\n\
             \\end{quote}\nOutside.\n\\end{document}\n";
  let (stderr, xml) = super::convert(tex, true);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  let quote = &xml[xml.find("<quote").expect("a quote")..xml.find("</quote>").expect("its end")];
  assert!(
    quote.contains("<sectional-block") && quote.contains("<section"),
    "{xml}"
  );
  assert!(
    quote.contains("Contents") && quote.contains("Text."),
    "{xml}"
  );
  assert!(
    xml[xml.find("</quote>").unwrap()..].contains("Outside."),
    "{xml}"
  );
}
