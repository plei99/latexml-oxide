//! anyfontsize.sty (2007/11/22, pts) — fonts at any requested size.
//!
//! No Perl binding exists; native binding, OXIDIZED_DESIGN #588(v). In TeX a `\fontsize{13}{15}`
//! request for a font shape declared only at fixed sizes is substituted by the nearest declared size
//! with a warning; anyfontsize patches NFSS's size lookup (`\tryif@simple`, anyfontsize.sty:47-104)
//! to load the font scaled to the requested size instead, and sets `\fontsubfuzz` to `9999pt`
//! (:44) so no substitution warning is printed. This converter's font model has no font files and
//! no size table: every `\fontsize` is kept at its requested value already, which is the behaviour the
//! package provides. What remains observable is `\fontsubfuzz`.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // anyfontsize.sty:44
  DefMacro!("\\fontsubfuzz", "9999pt");
});
