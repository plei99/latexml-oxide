//! Binding for `euler-math.sty` — Hermann Zapf's Euler as the math font through unicode-math.
//!
//! Perl LaTeXML ships no `euler-math.sty.ltxml` binding and, at the default `notex` setting, does not raw-load the
//! real file; under `--preload=latexml.sty` (no `rawstyles`) the package was skipped with `Warning:missing_file`.
//! Not raw-loaded: euler-math.sty:32-54 redefines `\hbar`, `\varPhi` and some seventy more symbols with
//! `\Umathchardef` at `\AtBeginDocument`, which this engine takes as an empty macro, so the raw file would blank
//! them. The font is presentation and LaTeXML's math is Unicode-native, so — as for `eulervm` (Perl
//! `eulervm.sty.ltxml`: "There's really nothing to do for LaTeXML") — the binding requires unicode-math with
//! `math-style=upright` as euler-math.sty:9 does, accepts the key options (`CharacterVariant`, `StylisticSet`,
//! `Style`, `Scale`, `Numbers`, euler-math.sty:18-24) without effect, and defines nothing else.
use latexml_package::prelude::*;

LoadDefinitions!({
  DeclareOption!(None, {});
  ProcessOptions!();
  RequirePackage!("unicode-math", options => vec![s!("math-style=upright")]);
});
