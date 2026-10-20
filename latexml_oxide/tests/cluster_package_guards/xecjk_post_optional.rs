//! xeCJK's font commands take their options before the font name or after it (xeCJK.sty:6316-6329
//! `\__xeCJK_pass_args:nnnn`: the trailing `O{}` is read only when the leading `o` was absent). The binding read
//! `\setCJKmainfont[]{}` and so stopped at the name: `\setCJKmainfont{Songti SC}[BoldFont={Songti SC Bold}]` (in an
//! `amsart` document under XeLaTeX) left "[BoldFont=Songti SC Bold, ...]" as a paragraph of the
//! document.

const PREAMBLE: &str = "\\documentclass{article}\n\\usepackage{xeCJK}\n";

/// The body of the one paragraph of `tex`'s document, from `<p>` to `</p>`.
fn only_paragraph(xml: &str) -> &str {
  let start = xml
    .find("<p>")
    .unwrap_or_else(|| panic!("no paragraph:\n{xml}"));
  let end = start + xml[start..].find("</p>").expect("</p>");
  assert_eq!(xml.matches("<p>").count(), 1, "one paragraph:\n{xml}");
  &xml[start..end + "</p>".len()]
}

fn assert_visible(preamble_commands: &str) {
  let tex =
    format!("{PREAMBLE}{preamble_commands}\\begin{{document}}\nVisible text.\n\\end{{document}}\n");
  let (stderr, xml) = super::convert(&tex, false);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  assert_eq!(
    super::perfect_kernel_batch46::warning_count(&stderr),
    0,
    "{stderr}"
  );
  assert_eq!(only_paragraph(&xml), "<p>Visible text.</p>", "{xml}");
}

#[test]
fn options_after_the_font_name_are_arguments() {
  assert_visible(
    "\\setCJKmainfont{Songti SC}[BoldFont={Songti SC Bold}, ItalicFont={Heiti SC}, BoldItalicFont={Heiti SC Medium}]\n\
     \\setCJKsansfont{Heiti SC}[Scale=0.9]\n\
     \\setCJKmonofont{Heiti SC}\n\
     \\setCJKromanfont{Songti SC}[BoldFont={Songti SC Bold}]\n\
     \\setCJKmathfont{Songti SC}[Scale=0.9]\n\
     \\setCJKfamilyfont{kai}{Kaiti SC}[BoldFont={Kaiti SC Bold}]\n\
     \\setCJKfallbackfamilyfont{kai}{STKaiti}[Scale=0.9]\n",
  );
}

#[test]
fn options_before_the_font_name_are_arguments() {
  assert_visible(
    "\\setCJKmainfont[BoldFont={Songti SC Bold}]{Songti SC}\n\
     \\setCJKsansfont [Scale=0.9] {Heiti SC}\n\
     \\setCJKfamilyfont{kai}[BoldFont={Kaiti SC Bold}]{Kaiti SC}\n\
     \\setCJKfallbackfamilyfont{kai}[Scale=0.9]{STKaiti}\n",
  );
}

/// With the options before the name, a `[` after it is not an argument (xeCJK.sty:6316-6324 reads the trailing
/// optional argument only when the leading one is absent), so it stays text.
#[test]
fn a_bracket_after_a_font_name_with_leading_options_is_text() {
  let tex = format!(
    "{PREAMBLE}\\begin{{document}}\n\
     \\CJKfontspec[Scale=0.9]{{Songti SC}}[kept] text.\n\
     \\end{{document}}\n"
  );
  let (stderr, xml) = super::convert(&tex, false);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  assert_eq!(
    super::perfect_kernel_batch46::warning_count(&stderr),
    0,
    "{stderr}"
  );
  assert_eq!(only_paragraph(&xml), "<p>[kept] text.</p>", "{xml}");
}

/// `\CJKfontspec` ends in `\ignorespaces` in both forms (xeCJK.sty:6378-6382 passes `\tex_ignorespaces:D`), so
/// the blank after the command is dropped: one space between the words, as in the PDF.
#[test]
fn cjkfontspec_ignores_the_spaces_after_it() {
  for call in [
    "\\CJKfontspec[Scale=0.9]{Songti SC} after",
    "\\CJKfontspec{Songti SC}[Scale=0.9] after",
    "\\CJKfontspec{Songti SC} after",
  ] {
    let tex = format!("{PREAMBLE}\\begin{{document}}\nBefore {call}.\n\\end{{document}}\n");
    let (stderr, xml) = super::convert(&tex, false);
    assert_eq!(super::error_count(&stderr), 0, "{stderr}");
    assert_eq!(
      super::perfect_kernel_batch46::warning_count(&stderr),
      0,
      "{stderr}"
    );
    assert_eq!(
      only_paragraph(&xml),
      "<p>Before after.</p>",
      "{call}:\n{xml}"
    );
  }
}
