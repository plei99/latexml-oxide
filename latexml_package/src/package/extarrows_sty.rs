//! extarrows.sty (v1.2.0, 2020/03/12) — extensible arrows with labels above and below.
//!
//! No Perl binding exists; this is a native binding (OXIDIZED_DESIGN #588(f)). extarrows.sty
//! defines each `\x<arrow>[below]{above}` as `\providecommand{…}[2][]{\ext@arrow 0099\x<arrow>fill@{#1}{#2}}`
//! (extarrows.sty:31-58): amsmath's `\ext@arrow` stretches a glyph built from `\arrowfill@` pieces
//! under the label. Here each one is amsmath's `\lx@long@arrow` over a stretchy operator, as
//! `\xrightarrow` is, so the label sits over a stretchy `<mo>` in `<munderover>`. `\providecommand`
//! is kept: a definition made before the package (mathtools' `\xleftrightarrow`) stays.
//! Witnesses (errors in the default configuration, before → after): 2610.17060 (7 → 0).
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // extarrows.sty:28
  RequirePackage!("amsmath");

  // The stretchy bases. A long arrow is the stretchy form of its short glyph (as amsmath's
  // `\xlongrightarrow`, amsmath_sty.rs): MathML stretches `<mo>` to the label's width either way.
  DefMath!("\\lx@extarrows@longequal", "=",
    role => "RELOP", meaning => "equals", stretchy => true, alias => "=");
  DefMath!("\\lx@extarrows@Longleftrightarrow", "\u{21D4}",
    role => "ARROW", stretchy => true, alias => "\\Longleftrightarrow");
  DefMath!("\\lx@extarrows@longleftrightarrow", "\u{2194}",
    role => "ARROW", stretchy => true, alias => "\\longleftrightarrow");
  DefMath!("\\lx@extarrows@Leftrightarrow", "\u{21D4}",
    role => "ARROW", stretchy => true, alias => "\\Leftrightarrow");
  DefMath!("\\lx@extarrows@leftrightarrow", "\u{2194}",
    role => "ARROW", stretchy => true, alias => "\\leftrightarrow");
  DefMath!("\\lx@extarrows@Longleftarrow", "\u{21D0}",
    role => "ARROW", stretchy => true, alias => "\\Longleftarrow");
  DefMath!("\\lx@extarrows@Longrightarrow", "\u{21D2}",
    role => "ARROW", stretchy => true, alias => "\\Longrightarrow");
  DefMath!("\\lx@extarrows@longleftarrow", "\u{2190}",
    role => "ARROW", stretchy => true, alias => "\\longleftarrow");
  DefMath!("\\lx@extarrows@longrightarrow", "\u{2192}",
    role => "ARROW", stretchy => true, alias => "\\longrightarrow");

  // extarrows.sty:31-58, in its order. Each is `\providecommand`: an existing definition wins.
  for (cs, base) in [
    ("\\xlongequal", "\\lx@extarrows@longequal"),
    ("\\xLongleftrightarrow", "\\lx@extarrows@Longleftrightarrow"),
    ("\\xlongleftrightarrow", "\\lx@extarrows@longleftrightarrow"),
    ("\\xLeftrightarrow", "\\lx@extarrows@Leftrightarrow"),
    ("\\xleftrightarrow", "\\lx@extarrows@leftrightarrow"),
    ("\\xLongleftarrow", "\\lx@extarrows@Longleftarrow"),
    ("\\xLongrightarrow", "\\lx@extarrows@Longrightarrow"),
    ("\\xlongleftarrow", "\\lx@extarrows@longleftarrow"),
    ("\\xlongrightarrow", "\\lx@extarrows@longrightarrow"),
  ] {
    if !is_defined_token(&T_CS!(cs)) {
      def_macro(T_CS!(cs), None,
        ExpansionBody::Tokens(mouth::tokenize_internal(TeXString::assembled(format!("\\lx@long@arrow{{{cs}}}{{{base}}}")))), None)?;
    }
  }
});
