//! Math guards: `\substack` letters upright under a raw class, `\varprojlim` limits beside the operator,
//! `\widehat` a small caret, a script on `L_{g*}` unparsed, `\partial_i` scripts floating onto the next
//! atom, and an unparsed formula that lost its `\qquad` and its scripts' bases.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{convert_html, error_count, warning_count};

/// A raw class's size command (`\DeclareRobustCommand{\scriptsize}{\@setfontsize…}`, mem10.clo:70, so
/// memoir's) ends in `\selectfont`; run in math, that merged the text font's upright shape into the math
/// font and every letter of `\substack`'s `style=\scriptsize` came out `mathvariant="normal"`
/// (a `\substack` under a limit or sum in a memoir document).
/// In math `\@setfontsize` now runs only the size update, as the class bindings' own size switches do
/// (dialect.rs); the letters stay italic. Perl's `\@setfontsize` never reaches `\selectfont`
/// (OXIDIZED_DESIGN_DIVERGENCES #288).
#[test]
fn raw_size_switch_keeps_math_italic() {
  let (stderr, html) = convert_html(
    "\\documentclass{article}\n\\usepackage{amsmath}\n\\makeatletter\n\
     \\DeclareRobustCommand{\\scriptsize}{\\@setfontsize\\scriptsize\\@viipt{8}}\n\\makeatother\n\
     \\begin{document}\n$a{\\scriptsize x}$\n\
     \\[ \\lim_{\\substack{\\to \\\\ x \\in U}} F|_U \\]\n\\end{document}\n",
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &html,
    "math",
    &["id=\"p1.m1\""],
    "<math id=\"p1.m1\" class=\"ltx_Math\" alttext=\"a{x}\" display=\"inline\"><mrow><mi>a</mi><mo>\u{2062}</mo><mi \
     mathsize=\"0.700em\">x</mi></mrow></math>",
  );
  assert_element(
    &html,
    "mtable",
    &[],
    "<mtable rowspacing=\"0pt\"><mtr><mtd><mo stretchy=\"false\">\u{2192}</mo></mtd></mtr><mtr><mtd><mrow><mi>x</mi><mo>\u{2208}</mo><mi>U</mi></mrow></mtd></mtr></mtable>",
  );
}

/// The witness class itself: memoir's `\scriptsize` is that robust `\@setfontsize` command. memoir.cls
/// raw-loads with the one `Warning:misdefined:\@iffirstamp` (memoir.cls:5454) it always gives.
#[test]
fn memoir_substack_keeps_math_italic() {
  let (stderr, html) = convert_html(
    "\\documentclass{memoir}\n\\usepackage{amsmath}\n\\begin{document}\n\
     \\[ \\lim_{\\substack{\\to \\\\ x \\in U}} F|_U \\]\n\\end{document}\n",
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  assert!(
    stderr.contains("Warning:misdefined:\\@iffirstamp"),
    "{stderr}"
  );
  assert_element(
    &html,
    "mtable",
    &[],
    "<mtable rowspacing=\"0pt\"><mtr><mtd><mo stretchy=\"false\">\u{2192}</mo></mtd></mtr><mtr><mtd><mrow><mi>x</mi><mo>\u{2208}</mo><mi>U</mi></mrow></mtd></mtr></mtable>",
  );
}

