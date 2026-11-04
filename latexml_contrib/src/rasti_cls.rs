//! rasti.cls (RAS Techniques and Instruments): the Royal Astronomical Society's class built from mnras.cls ("See
//! mnras_guide"; v3.0 defines the same commands as mnras.cls, with a catch-all `\DeclareOption*`), loaded as the mnras
//! binding: raw, its `\author` block of `\newauthor` lines and superscripted affiliations left the frontmatter's
//! groups unbalanced (2609.08700, 2609.09329; 18 papers in 2609). TeX Live 2026 distributes the class (rasti-journal
//! v3.1, 2026/9/9: rasti.cls:35-39), so it is a TeX Live file's binding, not an author-file stand-in
//! (`latexml_core::binding::author_files`).
use latexml_package::prelude::*;

LoadDefinitions!({
  DeclareOption!(None, {
    Digest!("\\PassOptionsToClass{\\CurrentOption}{mnras}")?;
  });
  ProcessOptions!();
  LoadClass!("mnras");
});
