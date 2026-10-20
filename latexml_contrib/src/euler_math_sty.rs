//! Binding for `euler-math.sty` — Hermann Zapf's Euler as the math font through unicode-math.
//!
//! Perl LaTeXML ships no `euler-math.sty.ltxml` binding and, at the default `notex` setting, does not raw-load the
//! real file; under `--preload=latexml.sty` (no `rawstyles`) the package was skipped with `Warning:missing_file`.
//! Not raw-loaded: euler-math.sty:38-44 redefines `\hbar`, `\varPhi` and some seventy more symbols with
//! `\Umathchardef` (from :42) at `\AtBeginDocument`, which this engine takes as an empty macro, so the raw file would
//! blank them. The font is presentation and LaTeXML's math is Unicode-native, so — as for `eulervm` (Perl
//! `eulervm.sty.ltxml`: "There's really nothing to do for LaTeXML") — the binding requires unicode-math with
//! `math-style=upright` as euler-math.sty:9 does, accepts the key options (`CharacterVariant`, `StylisticSet`,
//! `Style`, `Scale`, `Numbers`, :19-26) without effect, and defines nothing else. `\RequireTUTeX` (:8) is kept: the
//! iftex binding passes it (a XeTeX document converts under the default persona), so this calls the engine test
//! directly and halts under the pdfTeX persona as pdflatex does ("LuaTeX or XeTeX is required"), while the xetex and
//! luatex personas load clean. Not provided: `\setmathfont` features (euler-math.sty:30-35) and the names only the
//! file defines (`\varemptyset`, `\lesseqslantgtr`, …). OXIDIZED_DESIGN #592.
use latexml_package::prelude::*;

LoadDefinitions!({
  RequirePackage!("iftex");
  RawTeX!(r"\IFTEX@Require\iftutex{LuaTeX or XeTeX}\fi");
  RequirePackage!("unicode-math", options => vec![s!("math-style=upright")]);
  DeclareOption!(None, {});
  ProcessOptions!();
});
