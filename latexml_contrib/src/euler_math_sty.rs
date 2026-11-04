//! Binding for `euler-math.sty` — Hermann Zapf's Euler as the math font through unicode-math.
//!
//! Perl LaTeXML ships no `euler-math.sty.ltxml` binding and, at the default `notex` setting, does not raw-load the
//! real file; under `--preload=latexml.sty` (no `rawstyles`) the package was skipped with `Warning:missing_file`.
//! Not raw-loaded: the `\AtBeginDocument` block (euler-math.sty:38-192) redefines `\hbar`, `\varPhi` (:49) and
//! about ninety more symbols with `\Umathchardef` (92 of them, :42-153), which this engine takes as an empty macro,
//! so the raw file would blank them. The font is presentation and LaTeXML's math is Unicode-native, so — as for
//! `eulervm` (Perl `eulervm.sty.ltxml`: "There's really nothing to do for LaTeXML") — the binding loads iftex (:7)
//! and runs `\RequireTUTeX` (:8) as the file does, requires unicode-math with `math-style=upright` (:9), accepts the
//! key options (`CharacterVariant`, `StylisticSet`, `Style`, `Scale`, `Numbers`, :19-26) without effect, and of the
//! `\AtBeginDocument` block keeps only `\varPhi`, upright where amsmath's is the math italic's; the other symbols keep
//! their default glyphs (`\hbar` stays U+210F, where Euler's slot is U+0127, :42). The iftex binding passes `\RequireTUTeX` under every persona (iftex_sty.rs:91; Perl
//! `iftex.sty.ltxml:61` makes it a no-op; witnesses 2605.02089, 2605.16477), so the package loads under the default
//! persona too, `\iftutex` stays defined for the document, and a later change to that rule applies here. Not
//! provided: `\setmathfont` features (:30-35) and the names only the file defines (`\varemptyset`,
//! `\lesseqslantgtr`, …). OXIDIZED_DESIGN #971.
use latexml_package::prelude::*;

LoadDefinitions!({
  RequirePackage!("iftex");
  RawTeX!(r"\RequireTUTeX");
  RequirePackage!("unicode-math", options => vec![s!("math-style=upright")]);
  // euler-math.sty:38, :48-49 `\AtBeginDocument{… \Umathchardef\mupvarPhi = 0 0 "E04E \protected\def\varPhi{\mupvarPhi}…}`:
  // `\varPhi` is Euler's upright variant Phi (a private-use slot of Euler Math), here U+03A6 upright, named as the
  // `\varPhi` it stands for. At `\begin{document}`, over amsmath's math-italic `\varPhi` (amsmath.sty:384-395).
  DefMath!("\\mupvarPhi", None, "\u{03A6}", name => "varPhi");
  RawTeX!(r"\AtBeginDocument{\protected\def\varPhi{\mupvarPhi}}");
  DeclareOption!(None, {});
  ProcessOptions!();
});
