//! A large operator in a script is typeset in the script style (OXIDIZED_DESIGN_DIVERGENCES #606; the Perl
//! defect is KNOWN_PERL_ERRORS #668).
//!
//! Perl's `doVariablesizeOp` (TeX_Math.pool.ltxml:353) records `mathstyle="text"` on every `\sum`, `\int`, ... that
//! is not in display style, so one in a superscript (`e^{\sum x_iB_i}`, `z^{\sum D_i\cdot d}`) read as a text-style
//! token in a script, and MathML.pm:240-268 put the whole script back to full size: `<mstyle scriptlevel="-1">`,
//! the operator and its operands at the size of the line, and per-token sizes to compensate in the port. TeX sets
//! the operator, like everything else in the script, in script size. The same holds for a `\mathchar` of class
//! "1 (Package.pm:2997-3001), for the style `\over` moves a large operator to (TeX_Math.pool.ltxml:1051-1060)
//! and for a base the script is laid out under (`\scriptstyle\sum_i a_i`, MathML.pm:883-887).

use latexml::util::test::assert_element;

use super::{perfect_kernel_batch46::warning_count, svg_nested_picture::post_html_logged};
use crate::cluster::{convert_log, convert_to_xml};

const EXPONENT: &str = "tests/cluster_regressions/script_largeop_exponent.tex";
const IN_A_FRACTION: &str = "tests/cluster_regressions/script_largeop_fraction.tex";
const MATHCHAR: &str = "tests/cluster_regressions/script_largeop_mathchar.tex";
const OVER: &str = "tests/cluster_regressions/script_largeop_over.tex";
const GROUPED: &str = "tests/cluster_regressions/script_largeop_grouped.tex";
const DISPLAY: &str = "tests/cluster_regressions/script_largeop_display.tex";
const SCRIPTSTYLE: &str = "tests/cluster_regressions/script_largeop_scriptstyle.tex";

/// The core XML, the engine's warnings pinned (none expected; errors are asserted zero by `convert_to_xml`).
fn xml(fixture: &str) -> String {
  let engine_log = convert_log(fixture);
  assert_eq!(warning_count(&engine_log), 0, "{engine_log}");
  convert_to_xml(fixture)
}

/// The page, with the engine and POST warnings pinned (none expected).
fn html(fixture: &str) -> String {
  let (html, post_log) = post_html_logged(&xml(fixture));
  assert_eq!(warning_count(&post_log), 0, "{post_log}");
  html
}

#[test]
fn a_sum_in_a_script_records_the_script_style() {
  assert_element(
    &xml(EXPONENT),
    "XMTok",
    &[r#"meaning="sum""#],
    r#"<XMTok fontsize="70%" mathstyle="script" meaning="sum" role="SUMOP" scriptpos="post">∑</XMTok>"#,
  );
}

#[test]
fn a_sum_in_an_exponent_is_script_size_without_compensation() {
  assert_element(
    &html(EXPONENT),
    "msup",
    &[],
    r#"<msup><mi>e</mi><mrow><mo>∑</mo><mrow><msub><mi>x</mi><mi>i</mi></msub><mo>⁢</mo><msub><mi>B</mi><mi>i</mi></msub></mrow></mrow></msup>"#,
  );
}

#[test]
fn a_sum_in_a_script_of_a_fraction_is_script_size_without_compensation() {
  assert_element(
    &html(IN_A_FRACTION),
    "mfrac",
    &[],
    r#"<mfrac><msup><mi>y</mi><mi>d</mi></msup><msup><mi>z</mi><mrow><mo>∑</mo><mrow><msub><mi>D</mi><mi>i</mi></msub><mo lspace="0.222em" rspace="0.222em">⋅</mo><mi>d</mi></mrow></mrow></msup></mfrac>"#,
  );
}

/// `\mathchar"1350` is `\sum`'s class-1 character code: Package.pm:2997-3001 records `need_mathstyle` the
/// same way, and `mathchar.rs` / `register.rs` resolve it from the font's style.
#[test]
fn a_mathchar_operator_in_a_script_records_the_script_style() {
  assert_element(
    &html(MATHCHAR),
    "msup",
    &[],
    r#"<msup><mi>e</mi><mrow><mo>∑</mo><mi>x</mi></mrow></msup>"#,
  );
}

/// `\over` moves what it holds to the next style (Perl `adjustMathStyle_internal` sets the recorded mathstyle
/// from the FONT's style, TeX_Math.pool.ltxml:1054-1056): the numerator of an inline fraction is script, and
/// that of a fraction in a script scriptscript. A recorded `text` is not a style to step from.
#[test]
fn over_moves_a_large_operator_to_the_next_style() {
  let xml = xml(OVER);
  assert_element(
    &xml,
    "XMTok",
    &[r#"meaning="sum""#],
    r#"<XMTok fontsize="70%" mathstyle="script" meaning="sum" role="SUMOP" scriptpos="post">∑</XMTok>"#,
  );
  assert_element(
    &xml,
    "XMTok",
    &[r#"fontsize="50%" mathstyle="scriptscript" meaning="sum""#],
    r#"<XMTok fontsize="50%" mathstyle="scriptscript" meaning="sum" role="SUMOP" scriptpos="post">∑</XMTok>"#,
  );
}

/// A base the scripts are laid out under, in a style other than its context's, is wrapped for it as a
/// whole, scriptlevel included (Perl: `displaystyle` only), and its limits are sized from it.
#[test]
fn a_scriptstyle_sum_with_a_limit_is_wrapped_to_script_level() {
  assert_element(
    &html(SCRIPTSTYLE),
    "mstyle",
    &[],
    r#"<mstyle scriptlevel="+1"><msub><mo>∑</mo><mi>i</mi></msub></mstyle>"#,
  );
}

/// A style set in a group around the base alone (`{\scriptstyle\sum}_i`, `\mathop{\scriptstyle\sum}\limits_{i=1}^n`)
/// does not reach the scripts: the XML records them at the context's script step (70%), so they are laid out as Perl
/// does, under the context, with the base alone wrapped. Laid out under the base's style they were stepped to
/// scriptscript and given `mathsize="1.400em"` inside the wrap (limits larger than the operator in Chromium: 15.7px
/// against 11.2px).
#[test]
fn a_grouped_scriptstyle_sum_wraps_the_base_alone() {
  let html = html(GROUPED);
  assert_element(
    &html,
    "msub",
    &[],
    r#"<msub><mstyle scriptlevel="+1"><mo>∑</mo></mstyle><mi>i</mi></msub>"#,
  );
  assert_element(
    &html,
    "munderover",
    &[],
    r#"<munderover><mstyle scriptlevel="+1"><mo movablelimits="false">∑</mo></mstyle><mrow><mi>i</mi><mo>=</mo><mn>1</mn></mrow><mi>n</mi></munderover>"#,
  );
}

/// In display style `\over` moves the operators of the numerator and the denominator to text style (the recorded
/// `text` is the font's style stepped, `%mathstyle_adjust_map{display}{display}`, as in Perl).
#[test]
fn over_in_display_style_records_text() {
  let xml = xml(DISPLAY);
  assert_element(
    &xml,
    "XMTok",
    &[r#"meaning="sum""#],
    r#"<XMTok mathstyle="text" meaning="sum" role="SUMOP" scriptpos="mid">∑</XMTok>"#,
  );
  assert_element(
    &xml,
    "XMTok",
    &[r#"meaning="integral""#],
    r#"<XMTok mathstyle="text" meaning="integral" name="int" role="INTOP">∫</XMTok>"#,
  );
}

/// `\displaystyle` over a text-style context is the `displaystyle` of the stylemap and no scriptlevel.
#[test]
fn a_displaystyle_sum_stays_displaystyle_without_a_scriptlevel() {
  assert_element(
    &html(DISPLAY),
    "mstyle",
    &[],
    r#"<mstyle displaystyle="true"><munder><mo movablelimits="false">∑</mo><mi>i</mi></munder></mstyle>"#,
  );
}
