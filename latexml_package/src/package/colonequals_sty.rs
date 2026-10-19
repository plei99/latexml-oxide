//! colonequals.sty (oberdiek, v1.1 2016/05/16) — relations built from a vertically centred colon.
//!
//! No Perl binding exists; native binding, OXIDIZED_DESIGN #588(d). colonequals.sty builds each
//! symbol as `\ensuremath{\mathrel{<centred colons>\colonsep<symbol>}}` (:58-76, the
//! `\@make@colon@set` family at :78-153). Each is one relation token here, with the Unicode
//! character where one exists (U+2236 RATIO, U+2237 PROPORTION, U+2254 COLON EQUALS, U+2255 EQUALS
//! COLON, U+2A74 DOUBLE COLON EQUAL) and the composed string otherwise, as mathtools' colon
//! relations are (mathtools_sty.rs, Perl mathtools.sty.ltxml:722-745). `\colonsep` and
//! `\doublecolonsep` are the package's spacing hooks (:42-43, empty by default): they stay
//! definable so a document's `\renewcommand` works, and only move glyphs, so they do not enter
//! the symbol.
//! Witnesses (errors in the default configuration, before → after): 2610.19434 (1 → 0), 2610.20379 (1 → 0), 2610.22203 (2 → 1).
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // colonequals.sty:42-43
  DefMacro!("\\colonsep", None);
  DefMacro!("\\doublecolonsep", None);

  // `\ensuremath` (colonequals.sty:58, :67, :92…): each command is usable in text too.
  for (cs, text) in [
    // :58-63, :65-72
    ("ratio", "\u{2236}"),
    ("coloncolon", "\u{2237}"),
    // :148 `\@make@colon@set{equals}{=}`
    ("colonequals", "\u{2254}"),
    ("coloncolonequals", "\u{2A74}"),
    ("equalscolon", "\u{2255}"),
    ("equalscoloncolon", "=\u{2237}"),
    // :149 `{minus}{-}`
    ("colonminus", ":\u{2212}"),
    ("coloncolonminus", "\u{2237}\u{2212}"),
    ("minuscolon", "\u{2212}:"),
    ("minuscoloncolon", "\u{2212}\u{2237}"),
    // :150 `{approx}{\approx}`
    ("colonapprox", ":\u{2248}"),
    ("coloncolonapprox", "\u{2237}\u{2248}"),
    ("approxcolon", "\u{2248}:"),
    ("approxcoloncolon", "\u{2248}\u{2237}"),
    // :151 `{sim}{\sim}`
    ("colonsim", ":\u{223C}"),
    ("coloncolonsim", "\u{2237}\u{223C}"),
    ("simcolon", "\u{223C}:"),
    ("simcoloncolon", "\u{223C}\u{2237}"),
  ] {
    let inner = format!("\\lx@colonequals@{cs}");
    def_math(T_CS!(&inner), None, text.to_string(), MathPrimitiveOptions {
      role: Some("RELOP".to_string()),
      alias: Some(format!("\\{cs}")),
      ..MathPrimitiveOptions::default()
    })?;
    def_macro(T_CS!(&format!("\\{cs}")), None,
      ExpansionBody::Tokens(mouth::tokenize_internal(TeXString::assembled(format!("\\ensuremath{{{inner}}}")))), None)?;
  }
});
