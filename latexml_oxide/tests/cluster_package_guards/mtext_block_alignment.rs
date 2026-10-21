//! A stackengine stack in a formula is held in a line of its own, where it is centred (OXIDIZED_DESIGN_DIVERGENCES
//! #607).
//!
//! `\Centerstack` in a formula is an `ltx:inline-block vattach="middle"` in an `ltx:XMText`, which is
//! `<m:mtext><span class="ltx_inline-block ltx_align_middle">` in the page. A child of an `mtext` is a block-level box
//! of the math layout whatever its `display`, so the `vertical-align:middle` of `ltx_align_middle` has no line to
//! act on: Chrome puts the stack's last line on the baseline, Firefox its first, and the fences around it stretch
//! to one side only (the other half empty). TeX centres the stack on the axis. The page now holds the box in a
//! line of a `span.ltx_mtext_line`, where `vertical-align:middle` applies, and LaTeXML.css gives that box the width
//! of its content (Chromium sized the `mtext` without it and drew the following `.` over the box). Checked in
//! Chromium 1243 (playwright) and Firefox. `\parbox` and `minipage` (an `mtext` with a class, itself an
//! inline-block in LaTeXML.css), `\shortstack` (no `vattach`), `top`/`bottom` boxes and boxes outside a formula are
//! not wrapped.

use latexml::util::test::assert_element;

use super::{perfect_kernel_batch46::warning_count, svg_nested_picture::post_html_logged};
use crate::cluster::{convert_log, convert_to_xml};

const FIXTURE: &str = "tests/cluster_regressions/mtext_centered_stack.tex";

fn html() -> String {
  let engine_log = convert_log(FIXTURE);
  assert_eq!(warning_count(&engine_log), 0, "{engine_log}");
  let (html, post_log) = post_html_logged(&convert_to_xml(FIXTURE));
  assert_eq!(warning_count(&post_log), 0, "{post_log}");
  html
}

#[test]
fn a_centered_stack_in_a_formula_is_aligned_in_a_line_of_its_own() {
  assert_element(
    &html(),
    "mtext",
    &[],
    // `~` is the no-break space the page has for each blank of the stack's text.
    &r##"<mtext><span class="ltx_mtext_line"><span class="ltx_inline-block ltx_align_middle"><span class="ltx_p">principal~<math alttext="G" class="ltx_Math" display="inline"><mi>G</mi></math>-bundle~<math alttext="E\to U" class="ltx_Math" display="inline"><mrow><mi>E</mi><mo stretchy="false">→</mo><mi>U</mi></mrow></math></span><span class="ltx_p">with a~<math alttext="G" class="ltx_Math" display="inline"><mi>G</mi></math>-morphism~<math alttext="E\to X" class="ltx_Math" display="inline"><mrow><mi>E</mi><mo stretchy="false">→</mo><mi>X</mi></mrow></math></span></span></span></mtext>"##
      .replace('~', "\u{a0}"),
  );
}

/// The rule that goes with the wrapper (a Chromium sizing quirk, see the header); a box with a width of its own
/// is left to it.
#[test]
fn the_wrapped_box_has_the_width_of_its_content() {
  let css = include_str!("../../../latexml_post/resources/CSS/LaTeXML.css");
  assert!(
    css.lines().any(|l| l
      == r#".ltx_mtext_line > .ltx_inline-block:not([style*="width"]) { width:max-content; }"#),
    "LaTeXML.css has no max-content rule for .ltx_mtext_line"
  );
}

const UNWRAPPED: &str = "tests/cluster_regressions/mtext_unwrapped_boxes.tex";

const SHORTSTACK: &str = "tests/cluster_regressions/mtext_shortstack.tex";

fn unwrapped_html(fixture: &str) -> String {
  let engine_log = convert_log(fixture);
  assert_eq!(warning_count(&engine_log), 0, "{engine_log}");
  let (html, post_log) = post_html_logged(&convert_to_xml(fixture));
  assert_eq!(warning_count(&post_log), 0, "{post_log}");
  assert!(
    !html.contains("ltx_mtext_line"),
    "a box that is not a centred stack was wrapped:\n{html}"
  );
  html
}

/// `\parbox[t]` in a formula is an `mtext.ltx_parbox` whose box is `ltx_align_top`: untouched.
#[test]
fn a_top_parbox_in_a_formula_is_not_wrapped() {
  assert_element(
    &unwrapped_html(UNWRAPPED),
    "mtext",
    &[r#"class="ltx_parbox""#],
    &r##"<mtext class="ltx_parbox"><span class="ltx_inline-block ltx_parbox ltx_align_top" style="width:56.9pt;"><span class="ltx_p">top~<math alttext="q" class="ltx_Math" display="inline"><mi>q</mi></math></span><span class="ltx_p">two</span></span></mtext>"##
      .replace('~', "\u{a0}"),
  );
}

/// `\shortstack` has no `vattach`: untouched.
#[test]
fn a_shortstack_in_a_formula_is_not_wrapped() {
  assert_element(
    &unwrapped_html(SHORTSTACK),
    "mtext",
    &[],
    &r##"<mtext><span class="ltx_inline-block"><span class="ltx_p">top~<math alttext="t" class="ltx_Math" display="inline"><mi>t</mi></math></span><span class="ltx_p">bottom</span></span></mtext>"##
      .replace('~', "\u{a0}"),
  );
}

/// A centred stack in running text is a plain inline-block: no `mtext` to be a child of.
#[test]
fn a_centered_stack_in_text_is_not_wrapped() {
  assert_element(
    &unwrapped_html(UNWRAPPED),
    "span",
    &[r#"class="ltx_inline-block ltx_align_middle""#],
    r##"<span class="ltx_inline-block ltx_align_middle"><span class="ltx_p">one</span><span class="ltx_p">two</span></span>"##,
  );
}
