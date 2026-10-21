//! Three places where the spacing and the size of a math formula came out other than TeX sets them
//! (OXIDIZED_DESIGN_DIVERGENCES #611, #612, #613), each seen in a note's PDF and web page:
//!
//! * #611 `\delimiter`/`\mathchar` of class 3 (Rel): `\left\{d\delimiter"326A30C D\right\}`, the set-builder bar,
//!   is set with `\thickmuskip` either side. The glyph's own role (VERTBAR, re-roled MIDDLE by the fence rule, an Ord
//!   to the spacing walk, MathML.pm:1151) won over the class (Package.pm:2993), so the bar had no space.
//! * #612 a `\vphantom` beside a stretchy fence sizes it: `\left.x\vphantom{\int}\right|_{a}`, which physics.sty:181
//!   sets for `\evaluated{x}_{a}`. The strut was dropped (math_common.pool.ltxml:680 makes an `XMHint`, which
//!   MathParser.pm:440 `filter_hints` unlinks), so the bar was as short as the row and the limit hung from its middle.
//! * #613 `D_X\mathsf{-Mod}`: the minus opens a math group, an Ord atom, so it is unary and tight (tex.web §727); it
//!   was flat in the formula and set as the binary minus of `D_X - Mod`.

use latexml::util::test::assert_element;

use super::{perfect_kernel_batch46::warning_count, svg_nested_picture::post_html_logged};
use crate::cluster::{convert_log, convert_to_xml};

const PHYSICS: &str = "tests/cluster_regressions/physics_evaluated_strut.tex";
const STRUT: &str = "tests/cluster_regressions/fence_strut.tex";
const BAR: &str = "tests/cluster_regressions/relation_class_bar.tex";
const GROUP: &str = "tests/cluster_regressions/group_leading_binary.tex";

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

/// One formula of the page, the whole `<math>` element.
#[track_caller]
fn assert_math(html: &str, id: &str, expected: &str) {
  assert_element(html, "math", &[&format!(r#"id="{id}""#)], expected);
}

// ---- #611: a Rel-class bar ----------------------------------------------------------------------------------------

#[test]
fn a_class_3_delimiter_records_its_class() {
  assert_element(
    &xml(BAR),
    "XMTok",
    &[r#"mathclass="Rel""#, r#"role="MIDDLE""#],
    r#"<XMTok mathclass="Rel" role="MIDDLE">|</XMTok>"#,
  );
}

#[test]
fn a_set_builder_bar_of_class_3_has_relation_spacing() {
  // No `lspace`/`rspace="0em"`: the bar keeps the operator dictionary's thick space on both sides.
  assert_math(
    &html(BAR),
    "p1.m1",
    r#"<math id="p1.m1" class="ltx_Math" alttext="\left\{d\delimiter 52863756D\right\}" display="inline"><mrow><mo>{</mo><mi>d</mi><mo>|</mo><mi>D</mi><mo>}</mo></mrow></math>"#,
  );
}

#[test]
fn a_middle_bar_keeps_its_tight_spacing() {
  assert_math(
    &html(BAR),
    "p2.m1",
    r#"<math id="p2.m1" class="ltx_Math" alttext="\left\{d\middle|D\right\}" display="inline"><mrow><mo>{</mo><mi>d</mi><mo lspace="0em" rspace="0em" stretchy="true">|</mo><mi>D</mi><mo>}</mo></mrow></math>"#,
  );
}

#[test]
fn a_class_3_mathchar_outside_a_fence_is_unchanged() {
  // Its role is the relation `conditional` makes of it, so the class adds nothing.
  assert_math(
    &html(BAR),
    "p3.m1",
    r#"<math id="p3.m1" class="ltx_Math" alttext="d\mathchar 12906D" display="inline"><mrow><mi>d</mi><mo fence="false">|</mo><mi>D</mi></mrow></math>"#,
  );
}

#[test]
fn a_class_0_delimiter_bar_is_still_an_ordinary_atom() {
  assert_math(
    &html(BAR),
    "p4.m1",
    r#"<math id="p4.m1" class="ltx_Math" alttext="\left\{d\delimiter 2532108D\right\}" display="inline"><mrow><mo>{</mo><mi>d</mi><mo lspace="0em" rspace="0em">|</mo><mi>D</mi><mo>}</mo></mrow></math>"#,
  );
}

#[test]
fn a_class_3_delimiter_after_middle_is_a_fence_not_a_relation() {
  // `\middle\delimiter"326A30C` is the `\middle` bar: tight, as in TeX.
  assert_math(
    &html(BAR),
    "p5.m1",
    r#"<math id="p5.m1" class="ltx_Math" alttext="\left\{d\middle|D\right\}" display="inline"><mrow><mo>{</mo><mi>d</mi><mo lspace="0em" rspace="0em" stretchy="true">|</mo><mi>D</mi><mo>}</mo></mrow></math>"#,
  );
}

// ---- #612: a strut beside a stretchy fence ------------------------------------------------------------------------

#[test]
fn a_vphantom_gives_the_closing_fence_its_extent() {
  assert_element(
    &xml(STRUT),
    "XMTok",
    &[r#"stretchy="true""#, r#"role="CLOSE""#, r#"depth="8.6pt""#],
    r#"<XMTok depth="8.6pt" height="13.6pt" role="CLOSE" stretchy="true">|</XMTok>"#,
  );
}

#[test]
fn a_display_integral_strut_makes_the_bar_as_tall_as_the_integral() {
  // The display-style integral is 13.61pt + 8.61pt (TeX), 2.22em.
  assert_math(
    &html(STRUT),
    "S0.Ex1.m1",
    r#"<math id="S0.Ex1.m1" class="ltx_Math" alttext="\left.\lambda(\alpha)\vphantom{\int}\right|_{X}" display="block"><msub><mrow><mrow><mi>λ</mi><mo>⁡</mo><mrow><mo stretchy="false">(</mo><mi>α</mi><mo stretchy="false">)</mo></mrow></mrow><mo minsize="2.220em">|</mo></mrow><mi>X</mi></msub></math>"#,
  );
}

#[test]
fn a_fence_without_a_strut_has_no_minimum() {
  assert_math(
    &html(STRUT),
    "S0.Ex2.m1",
    r#"<math id="S0.Ex2.m1" class="ltx_Math" alttext="\left.\lambda(\alpha)\right|_{X}" display="block"><msub><mrow><mrow><mi>λ</mi><mo>⁡</mo><mrow><mo stretchy="false">(</mo><mi>α</mi><mo stretchy="false">)</mo></mrow></mrow><mo>|</mo></mrow><mi>X</mi></msub></math>"#,
  );
}

#[test]
fn a_strut_is_the_height_of_its_argument() {
  // `\vphantom{x}` is 4.3pt high, the bar 0.43em at least.
  assert_math(
    &html(STRUT),
    "p3.m1",
    r#"<math id="p3.m1" class="ltx_Math" alttext="\left.\lambda(\alpha)\vphantom{x}\right|_{X}" display="inline"><msub><mrow><mrow><mi>λ</mi><mo>⁡</mo><mrow><mo stretchy="false">(</mo><mi>α</mi><mo stretchy="false">)</mo></mrow></mrow><mo minsize="0.430em">|</mo></mrow><mi>X</mi></msub></math>"#,
  );
}

#[test]
fn a_strut_beside_a_bar_that_is_no_fence_is_dropped() {
  assert_math(
    &html(STRUT),
    "p4.m1",
    r#"<math id="p4.m1" class="ltx_Math" alttext="a\vphantom{\int}|_{X}" display="inline"><msub><mrow><mi>a</mi><mo stretchy="false">|</mo></mrow><mi>X</mi></msub></math>"#,
  );
}

#[test]
fn a_strut_sizes_both_fences_of_the_pair() {
  let page = html(STRUT);
  assert_math(
    &page,
    "S0.Ex3.m1",
    r#"<math id="S0.Ex3.m1" class="ltx_Math" alttext="\left(x\vphantom{\int}\right)" display="block"><mrow><mo minsize="2.220em">(</mo><mi>x</mi><mo minsize="2.220em">)</mo></mrow></math>"#,
  );
  assert_math(
    &page,
    "S0.Ex4.m1",
    r#"<math id="S0.Ex4.m1" class="ltx_Math" alttext="\left|x\vphantom{\int}\right|" display="block"><mrow><mo minsize="2.220em">|</mo><mi>x</mi><mo minsize="2.220em">|</mo></mrow></math>"#,
  );
}

#[test]
fn physics_evaluated_bar_is_as_tall_as_its_strut() {
  assert_math(
    &html(PHYSICS),
    "S0.Ex1.m1",
    r#"<math id="S0.Ex1.m1" class="ltx_Math" alttext="\evaluated{\lambda(\alpha)}_{X \wedge X}" display="block"><msub><mrow><mrow><mi>λ</mi><mo>⁡</mo><mrow><mo stretchy="false">(</mo><mi>α</mi><mo stretchy="false">)</mo></mrow></mrow><mo fence="false" minsize="2.220em">|</mo></mrow><mrow><mi>X</mi><mo>∧</mo><mi>X</mi></mrow></msub></math>"#,
  );
  assert_math(
    &html(PHYSICS),
    "S0.Ex2.m1",
    r#"<math id="S0.Ex2.m1" class="ltx_Math" alttext="K_{E}=\evaluated{(K_X+E)}_{E}" display="block"><mrow><msub><mi>K</mi><mi>E</mi></msub><mo>=</mo><msub><mrow><mrow><mo stretchy="false">(</mo><mrow><msub><mi>K</mi><mi>X</mi></msub><mo>+</mo><mi>E</mi></mrow><mo stretchy="false">)</mo></mrow><mo fence="false" minsize="2.220em">|</mo></mrow><mi>E</mi></msub></mrow></math>"#,
  );
}

#[test]
fn physics_evaluated_with_a_delimited_argument_sizes_both_fences() {
  let page = html(PHYSICS);
  assert_math(
    &page,
    "S0.Ex3.m1",
    r#"<math id="S0.Ex3.m1" class="ltx_Math" alttext="\evaluated(f|_{a}" display="block"><msub><mrow><mo minsize="2.220em">(</mo><mi>f</mi><mo fence="false" minsize="2.220em">|</mo></mrow><mi>a</mi></msub></math>"#,
  );
  // Inline, the strut is the text-style integral's, 0.88em.
  assert_math(
    &page,
    "p3.m1",
    r#"<math id="p3.m1" class="ltx_Math" alttext="\evaluated{f}_{a}" display="inline"><msub><mrow><mi>f</mi><mo fence="false" minsize="0.880em">|</mo></mrow><mi>a</mi></msub></math>"#,
  );
}

// ---- #613: a minus that opens a group ------------------------------------------------------------------------------

#[test]
fn a_group_leading_minus_records_the_ordinary_class() {
  assert_element(
    &xml(GROUP),
    "XMTok",
    &[r#"mathclass="Ord""#],
    r#"<XMTok mathclass="Ord" meaning="minus" role="ADDOP">-</XMTok>"#,
  );
}

#[test]
fn a_minus_opening_a_font_group_is_tight() {
  assert_math(
    &html(GROUP),
    "p1.m1",
    r#"<math id="p1.m1" class="ltx_Math" alttext="D_{X}\mathsf{-Mod}" display="inline"><mrow><msub><mi>D</mi><mi>X</mi></msub><mo lspace="0em" rspace="0em">−</mo><mi>𝖬𝗈𝖽</mi></mrow></math>"#,
  );
}

#[test]
fn a_minus_opening_a_brace_group_is_tight() {
  assert_math(
    &html(GROUP),
    "p4.m1",
    r#"<math id="p4.m1" class="ltx_Math" alttext="a{-b}" display="inline"><mrow><mi>a</mi><mo lspace="0em" rspace="0em">−</mo><mi>b</mi></mrow></math>"#,
  );
}

#[test]
fn a_minus_opening_a_mathrm_group_is_tight() {
  assert_math(
    &html(GROUP),
    "p5.m1",
    r#"<math id="p5.m1" class="ltx_Math" alttext="a\mathrm{-b}" display="inline"><mrow><mi>a</mi><mo lspace="0em" rspace="0em">−</mo><mi mathvariant="normal">b</mi></mrow></math>"#,
  );
}

#[test]
fn an_ordinary_binary_minus_keeps_its_spacing() {
  let page = html(GROUP);
  assert_math(
    &page,
    "p2.m1",
    r#"<math id="p2.m1" class="ltx_Math" alttext="a-b" display="inline"><mrow><mi>a</mi><mo>−</mo><mi>b</mi></mrow></math>"#,
  );
  assert_math(
    &page,
    "p3.m1",
    r#"<math id="p3.m1" class="ltx_Math" alttext="a\mathsf{x}-b" display="inline"><mrow><mrow><mi>a</mi><mo>⁢</mo><mi>𝗑</mi></mrow><mo>−</mo><mi>b</mi></mrow></math>"#,
  );
}

#[test]
fn a_group_that_opens_its_formula_is_unchanged() {
  assert_math(
    &html(GROUP),
    "p6.m1",
    r#"<math id="p6.m1" class="ltx_Math" alttext="\mathsf{-1}" display="inline"><mrow><mo>−</mo><mn>𝟣</mn></mrow></math>"#,
  );
}

#[test]
fn a_negative_exponent_is_unchanged() {
  assert_math(
    &html(GROUP),
    "p7.m1",
    r#"<math id="p7.m1" class="ltx_Math" alttext="x^{-1}" display="inline"><msup><mi>x</mi><mrow><mo>−</mo><mn>1</mn></mrow></msup></math>"#,
  );
}

#[test]
fn a_script_argument_records_no_class() {
  // The group of `x^{-1}`, `x^\mathrm{-1}` and `x_i^{-1}` is the script's own: its minus is a prefix, with no
  // `mathclass`; nor has the minus of a group that opens its formula (`\mathsf{-1}`).
  let xml = xml(GROUP);
  for (id, expected) in [
    (
      "p6.m1",
      r#"<Math mode="inline" tex="\mathsf{-1}" text="- 1" xml:id="p6.m1"><XMath><XMApp><XMTok meaning="minus" role="ADDOP">-</XMTok><XMTok font="sansserif" meaning="1" role="NUMBER">1</XMTok></XMApp></XMath></Math>"#,
    ),
    (
      "p7.m1",
      r#"<Math mode="inline" tex="x^{-1}" text="x ^ (- 1)" xml:id="p7.m1"><XMath><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">x</XMTok><XMApp><XMTok fontsize="70%" meaning="minus" role="ADDOP">-</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp></XMApp></XMath></Math>"#,
    ),
    (
      "p8.m1",
      r#"<Math mode="inline" tex="x^{\mathrm{-1}}" text="x ^ (- 1)" xml:id="p8.m1"><XMath><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">x</XMTok><XMApp><XMTok fontsize="70%" meaning="minus" role="ADDOP">-</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp></XMApp></XMath></Math>"#,
    ),
    (
      "p9.m1",
      r#"<Math mode="inline" tex="x_{i}^{-1}" text="(x _ i) ^ (- 1)" xml:id="p9.m1"><XMath><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">x</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">i</XMTok></XMApp><XMApp><XMTok fontsize="70%" meaning="minus" role="ADDOP">-</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp></XMApp></XMath></Math>"#,
    ),
  ] {
    assert_element(&xml, "Math", &[&format!(r#"xml:id="{id}""#)], expected);
  }
}
