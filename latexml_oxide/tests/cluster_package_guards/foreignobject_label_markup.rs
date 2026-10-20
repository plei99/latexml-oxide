//! The labels of a picture are HTML, set in the picture's `foreignObject`s. Two things about a picture in a formula
//! were errors of the Rust port (the stylesheet that mishandled the first is Perl's, byte for byte), and one is a
//! stylesheet rule of Perl's that the fork changes (OXIDIZED_DESIGN_DIVERGENCES #597).
//!
//! 1. A picture inside a formula (`\overline{\left\{\begin{tikzpicture}...\right\}}`, the boundary divisor of
//!    a boundary-divisor picture in a formula) reaches the HTML as an `m:mtext` holding the picture. A `$T^C$` label of that
//!    picture is a nested `ltx:Math`; Perl's `convertXMTextContent` rebuilds it as `ltx:Math > m:math`, which the
//!    stylesheet turns into `<math class="ltx_Math">`. Rust's #101 path emitted the bare `m:math`, which the
//!    `foreignObject`'s `copy-foreign` rule (`m:*` resumes with the children of the element) reduced to an `msup`
//!    in a span, no math in HTML: it drew "TC" on one baseline.
//! 2. The picture itself: Perl's `convertXMTextContent($doc, 1, $node)` is given the `ltx:picture` node, so the
//!    `m:mtext` holds `<span class="ltx_inline-block"><svg class="ltx_picture">`; the port gave it the picture's
//!    children, so the `svg` stood bare in the `mtext` between NBSP text nodes (the markup's indentation), two lines
//!    around a block, the fences stretched to twice the picture, and the picture stood at their bottom.
//! 3. `<xsl:strip-space elements="ltx:picture svg:*"/>` dropped the blank text node between the words and the math
//!    of a label (`\textbf{Family:} $xy$` in a `\node`, a figure of `\node` labels: "Family:xy"; Perl's too).

use latexml::util::test::assert_element;

use super::{perfect_kernel_batch46::warning_count, svg_nested_picture::post_html_logged};
use crate::cluster::{convert_log, convert_to_xml};

const FIXTURE: &str = "tests/cluster_regressions/foreignobject_math_label.tex";

/// The page, with the engine and POST warnings pinned (none expected; errors are asserted zero by the helpers).
fn html() -> String {
  let engine_log = convert_log(FIXTURE);
  assert_eq!(warning_count(&engine_log), 0, "{engine_log}");
  let (html, post_log) = post_html_logged(&convert_to_xml(FIXTURE));
  assert_eq!(warning_count(&post_log), 0, "{post_log}");
  html
}

#[test]
fn a_math_label_of_a_picture_in_a_formula_keeps_its_math_root() {
  assert_element(
    &html(),
    "foreignObject",
    &[r#"width="19.31""#],
    r##"<foreignObject height="11.64" overflow="visible" style="--ltx-fo-width:1.4em;--ltx-fo-height:0.84em;--ltx-fo-depth:0em;font-size:10pt;" transform="matrix(1 0 0 -1 0 11.64)" width="19.31"><span class="ltx_foreignobject_container"><span class="ltx_foreignobject_content"><math class="ltx_Math" alttext="T^{C}" display="inline"><msup><mi>T</mi><mi>C</mi></msup></math></span></span></foreignObject>"##,
  );
}

#[test]
fn the_space_between_words_and_math_in_a_label_stays() {
  let html = html();
  let at = html.find("Family:").expect("the label");
  let start = html[..at]
    .rfind("<foreignObject")
    .expect("its foreignObject");
  let end = at + html[at..].find("</foreignObject>").expect("end");
  let label = &html[start..end];
  assert!(
    label.contains(r#"<span class="ltx_text ltx_font_bold">Family:</span> <math"#),
    "{label}"
  );
  assert!(label.contains("</math> in Patch <math"), "{label}");
}

/// Perl's shape: the `mtext` holds the picture's `inline-block` span and its `svg`, between the fences.
#[test]
fn a_picture_in_a_formula_is_the_mtext_content_as_in_perl() {
  let html = html();
  let at = html.find("<mtext>").expect("the mtext of the picture");
  let end = at + html[at..].find("</mtext>").expect("</mtext>");
  let mtext = &html[at..end + "</mtext>".len()];
  assert!(
    mtext.starts_with(r#"<mtext><span class="ltx_inline-block"><svg class="ltx_picture" height="#),
    "{mtext}"
  );
  assert!(mtext.ends_with("</svg></span></mtext>"), "{mtext}");
  assert!(
    html[..at].ends_with("<mo>{</mo>") && html[end..].starts_with("</mtext><mo>}</mo>"),
    "the fences around the picture:\n{html}"
  );
}
