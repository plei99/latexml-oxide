//! calligra.sty (fundus-calligra, v1.11 2026/05/19) — the Calligra calligraphic text font.
//!
//! No Perl binding exists; native binding, OXIDIZED_DESIGN #588(r). The package declares the T1
//! family `calligra` (calligra.sty:30-31) and switches to it with `\calligra` / `\textcalligra{…}`
//! (:32-43), redefining `\ss` for the font's own ß slot (char 255, kerned). The family selection is
//! the meaning: the family code `calligra` is the script family in the font model (font.rs), so the
//! text carries the script font the package sets it in; `\ss` stays T1's ß.
//! Witnesses (errors in the default configuration, before → after): 2610.17646 (61 → 7 with its xifthen, quiver and dynkin-diagrams), 2610.20862.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // calligra.sty:32-43
  DefMacro!("\\calligra",
    "\\renewcommand\\ss{\\symbol{255}\\kern-.22em}\\fontfamily{calligra}\\fontencoding{T1}\\selectfont",
    robust => true);
  DefMacro!("\\textcalligra{}",
    "\\begingroup\\renewcommand\\ss{\\symbol{255}\\kern-.22em}\\fontfamily{calligra}\\fontencoding{T1}\\selectfont #1\\endgroup");
});
