//! leftindex.sty (v0.2beta, 2024/01/31) — left super- and subscripts.
//!
//! No Perl binding exists; native binding, OXIDIZED_DESIGN #588(o). `\leftindex[#1][#2]^{#3}_{#4}{#5}`
//! (leftindex.sty:73-89) typesets `#3`/`#4` as scripts of an empty `\mathop{}` kerned in front of
//! `#5`, slanted to fit it and sized by `\vphantom`s of `#2`/`#1`
//! (`\__leftindex_auxiliary:nnnnn`, :128-207). Here they are the prescripts of `#5` in one
//! multiscript application (tensor_sty.rs `\lx@multiscripts`): `<mmultiscripts>` with
//! `<mprescripts/>`. The phantoms only size and slant the scripts. `\LeftIndex` and
//! `\manualleftindex` have no base: their scripts are prescripts of an empty base, as their
//! `\mathop{}` is.
//! Witnesses (errors in the default configuration, before → after): 2610.19606, 2610.21475, 2610.21489, 2610.21647 (1 → 0 each).
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // leftindex.sty:1
  RequirePackage!("xparse");
  RequirePackage!("mathtools");
  tensor_sty::define_multiscripts()?;
  // leftindex.sty:62-65, :73-108: the argument specs are the package's, read by the kernel's
  // ltcmd; `E{^_}{{}{}}` takes `^` and `_` in either order, each at most once.
  RawTeX!(r"\DeclareDocumentCommand\manualleftindex{mmmm}{\lx@multiscripts{\manualleftindex{#1}{#2}{#3}{#4}}{\lx@ms@col{#4}{#3}}{}{}}
\DeclareDocumentCommand\leftindex{o o E{^_}{{}{}} m}{\lx@multiscripts{\leftindex^{#3}_{#4}{#5}}{\lx@ms@col{#4}{#3}}{#5}{}}
\DeclareDocumentCommand\LeftIndex{o o E{^_}{{}{}}}{\lx@multiscripts{\LeftIndex^{#3}_{#4}}{\lx@ms@col{#4}{#3}}{}{}}");
});
