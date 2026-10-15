//! physics' `\qty{…}` with an argument the math parser cannot read on its own
//! (`x \mid y=0,z=1`, `x\middle|y`): the argument stays what it is, an `XMArg` the
//! dual's content `XMRef` points at, and the dual keeps both branches.
//!
//! The dual (`\quantity`, physics.sty.ltxml:132-145) is `XMDual(XMRef→arg,
//! XMWrap({ XMArg }))`. The argument is parsed first, on its own; when that fails it
//! stays an `XMArg`, and the presentation `XMWrap` is parsed next, where Perl's
//! parser takes the standing `XMArg` as ONE terminal (`getGrammaticalRole`: an
//! `ATOM`, MathParser.pm:840-853; `parse_internal`, :789-804). The lexer flattened it
//! into its tokens instead, so the fence was built from the loose tokens, the
//! `XMArg` (and its id) was gone, and the cleanup of the lost id deleted the dual's
//! content `XMRef`: one child left, rendered as the error box "Empty XMDual".
//! Witness: set-builder formulas `\qty{x \mid y=0,z=1}` (26 formulas in 15 notes).
use libxml::{parser::Parser, xpath::Context};

use super::perfect_kernel_batch46::{convert, convert_html, error_count};

const NS: &str = "http://dlmf.nist.gov/LaTeXML";

fn doc(body: &str) -> String {
  format!(
    "\\documentclass{{article}}\n\\usepackage{{physics}}\n\\begin{{document}}\n{body}\n\\end{{document}}\n"
  )
}

/// Every `XMDual` of the document has exactly its two branches, and each `XMRef`
/// resolves to an element of the document. Returns the number of duals checked.
fn assert_duals_whole(xml: &str) -> usize {
  let parsed = Parser::default()
    .parse_string(xml)
    .expect("well-formed XML");
  let mut ctx = Context::new(&parsed).expect("xpath context");
  ctx.register_namespace("ltx", NS).expect("namespace");
  let duals = ctx.findnodes("//ltx:XMDual", None).expect("duals");
  for dual in &duals {
    let branches = dual.get_child_elements();
    assert_eq!(
      branches.len(),
      2,
      "an XMDual keeps its content and presentation branches:\n{xml}"
    );
  }
  for xmref in ctx.findnodes("//ltx:XMRef", None).expect("refs") {
    let idref = xmref.get_attribute("idref").expect("XMRef has an idref");
    let target = ctx
      .findnodes(&format!("//*[@xml:id='{idref}']"), None)
      .expect("target lookup");
    assert_eq!(target.len(), 1, "XMRef idref={idref} resolves:\n{xml}");
  }
  duals.len()
}

/// The unparsed argument is read as one terminal by the fence around it: the
/// content branch is an `XMRef` to the standing `XMArg`.
fn assert_arg_stands(xml: &str) {
  let parsed = Parser::default()
    .parse_string(xml)
    .expect("well-formed XML");
  let mut ctx = Context::new(&parsed).expect("xpath context");
  ctx.register_namespace("ltx", NS).expect("namespace");
  let content_refs = ctx
    .findnodes("//ltx:XMDual/*[1][self::ltx:XMRef]", None)
    .expect("content refs");
  assert_eq!(
    content_refs.len(),
    1,
    "the dual's content is an XMRef:\n{xml}"
  );
  let idref = content_refs[0].get_attribute("idref").unwrap();
  let args = ctx
    .findnodes(&format!("//ltx:XMArg[@xml:id='{idref}']"), None)
    .expect("arg lookup");
  assert_eq!(
    args.len(),
    1,
    "the XMRef points at the standing XMArg:\n{xml}"
  );
}

fn assert_html_whole(body: &str, shown: &[&str]) {
  let (stderr, html) = convert_html(&doc(body));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    !html.contains("Empty XMDual"),
    "the HTML has no 'Empty XMDual' error box:\n{html}"
  );
  for fragment in shown {
    assert!(html.contains(fragment), "HTML shows {fragment}:\n{html}");
  }
}

/// A condition that is a list of relations: the set-builder form of the notes.
#[test]
fn qty_bar_with_comma_condition_keeps_both_branches() {
  let (stderr, xml) = convert(&doc(r"\[\qty{x \mid y=0,z=1}\]"), true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(assert_duals_whole(&xml), 1);
  assert_arg_stands(&xml);
  assert_html_whole(r"\[\qty{x \mid y=0,z=1}\]", &[
    ">{</mo>",
    ">∣</mo>",
    ">}</mo>",
    "<mi>y</mi>",
    "<mn>0</mn>",
    "<mi>z</mi>",
    "<mn>1</mn>",
  ]);
}

/// `\middle|` in the argument: a middle bar is a separator of a fence of its own.
#[test]
fn qty_middle_bar_keeps_both_branches() {
  let (stderr, xml) = convert(&doc(r"\[\qty{x\middle|y}\]"), true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(assert_duals_whole(&xml), 1);
  assert_html_whole(r"\[\qty{x\middle|y}\]", &[
    ">{</mo>",
    ">}</mo>",
    "<mi>x</mi>",
    "<mi>y</mi>",
  ]);
}

/// The controls, which parsed before: the argument is read on its own and the dual
/// keeps both branches as ever.
#[test]
fn qty_controls_keep_both_branches() {
  for body in [
    r"\[\left\{x\middle|y\right\}\]",
    r"\[\{x\mid y\}\]",
    r"\[\qty{x\mid y}\]",
    r"\[\qty{x\mid y=0}\]",
  ] {
    let (stderr, xml) = convert(&doc(body), true);
    assert_eq!(error_count(&stderr), 0, "{body}: {stderr}");
    assert_duals_whole(&xml);
    assert_html_whole(body, &[">{</mo>", ">}</mo>"]);
  }
}
