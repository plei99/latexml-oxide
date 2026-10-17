//! polyglossia.sty — raw load.
//!
//! polyglossia.sty:759-761 `\__xpg_setup_font:nn {rm}/{sf}/{tt}` redefines
//! `\rmfamily`/`\sffamily`/`\ttfamily` (:730-758) so that a font switch inside
//! a non-Latin language calls `\__xpg_add_font_feature_script:nnn` (:672),
//! whose `\fontspec_if_script:nTF {script}` (:681, 2.13; 2.9, the previous TL
//! revision, called `\xpg_if_script:nTF` at :641, an alias of it) either adds
//! `\addfontfeature{Script=…}` or raises "The current main roman font, cmr10,
//! does not contain the "Greek" script!" (:715-717 — a counted `\errmessage`).
//! fontspec_sty.rs answers `\fontspec_if_script:n…` TRUE (no OpenType font
//! model to ask; see the comment there), so nothing is overridden here.
//! Perl raw-loads polyglossia too (no `.ltxml`) but fails earlier on the
//! fontspec conditionals it never defines.
//! Witnesses: fontsetup/fspsample ×2 (11 → 0, lualatex clean), greektonoi
//! (16 → 0), latex-mr (98 + Fatal → 3).
//! Guard: `perfect_kernel_batch56::polyglossia_script_check_passes_without_font`.
use crate::prelude::*;

LoadDefinitions!({
  InputDefinitions!("polyglossia", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
