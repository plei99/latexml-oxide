//! The labels of a picture are HTML, set in the picture's `foreignObject`s, and two stylesheet rules lost part of
//! them (both are Perl's: `LaTeXML-math-xhtml.xsl` and `LaTeXML-picture-xhtml.xsl` are the same files).
//!
//! 1. A picture inside a formula (`\overline{\left\{\begin{tikzpicture}...\right\}}`, the boundary divisor of
//!    a boundary-divisor picture in a formula) reaches the HTML as an `m:mtext` holding the `svg`. A `$T^C$` label of that
//!    picture is a nested `ltx:Math`, which the MathML pass converts to a bare `m:math` (OXIDIZED_DESIGN #101), and
//!    the `foreignObject` copies its content in `copy-foreign` mode, where `m:*` "resumed" with
//!    `<xsl:apply-templates/>`: the children of the `m:math`, an `msup` inside a span, which is no math in HTML and
//!    drew "TC" on one baseline.
//! 2. `<xsl:strip-space elements="ltx:picture svg:*"/>` dropped the blank text node between the words and the math
//!    of a label (`\textbf{Family:} $xy$` in a `\node`, a figure of `\node` labels: "Family:xy").
//!
//! And the picture in the formula itself: the `m:mtext` that holds it had a NBSP text node on either side of the
//! `svg` (the indentation of the picture's markup), two lines of height around a block, so the fences stretched to
//! twice the picture and the picture stood at the bottom of them (3 in the same figure).

use latexml::util::test::assert_element;

use super::svg_nested_picture::post_html;
use crate::cluster::convert_to_xml;

fn html() -> String {
  post_html(&convert_to_xml(
    "tests/cluster_regressions/foreignobject_math_label.tex",
  ))
}

#[test]
fn a_math_label_of_a_picture_in_a_formula_keeps_its_math_root() {
  assert_element(
    &html(),
    "foreignObject",
    &[r#"width="19.31""#],
    r##"<foreignObject height="11.64" overflow="visible" style="--ltx-fo-width:1.4em;--ltx-fo-height:0.84em;--ltx-fo-depth:0em;font-size:10pt;" transform="matrix(1 0 0 -1 0 11.64)" width="19.31"><span class="ltx_foreignobject_container"><span class="ltx_foreignobject_content"><math alttext="T^{C}" display="inline"><msup><mi>T</mi><mi>C</mi></msup></math></span></span></foreignObject>"##,
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

/// The `svg` is the whole content of its `m:mtext`: its height is the picture's, and the fences stretch to it.
#[test]
fn a_picture_in_a_formula_is_the_only_content_of_its_mtext() {
  let html = html();
  let at = html.find("<mtext>").expect("the mtext of the picture");
  let end = at + html[at..].find("</mtext>").expect("</mtext>");
  let mtext = &html[at..end + "</mtext>".len()];
  assert!(
    mtext.starts_with(r#"<mtext><svg height="32.05" "#),
    "{mtext}"
  );
  assert!(mtext.ends_with("</svg></mtext>"), "{mtext}");
  assert!(
    html[..at].ends_with("<mo>{</mo>") && html[end..].starts_with("</mtext><mo>}</mo>"),
    "the fences around the picture:\n{html}"
  );
}
