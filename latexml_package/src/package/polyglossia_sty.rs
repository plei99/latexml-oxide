//! polyglossia.sty — raw load.
//!
//! polyglossia.sty:719-721 `\__xpg_setup_font:nn {rm}/{sf}/{tt}` redefines
//! `\rmfamily`/`\sffamily`/`\ttfamily` (:690-718) so that a font switch inside
//! a non-Latin language calls `\__xpg_add_font_feature_script:nnn` (:632),
//! whose `\fontspec_if_script:nTF {script}` (:681, 2.13; the 2.12 file called
//! `\xpg_if_script:nTF` at :641, an alias of it) either adds
//! `\addfontfeature{Script=…}` or raises "The current main roman font, cmr10,
//! does not contain the "Greek" script!" (:677 — a counted `\errmessage`).
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
