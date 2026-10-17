//! latex-lab-testphase-minipage.sty — nothing left to declare.
//!
//! `\DocumentMetadata{tagging=on}` loads `latex-lab-testphase-latest.sty`
//! (documentmetadata-support.ltx:72), which pulls in this module (:47).
//! Through latex-lab 0.81g the module declared the four sockets
//! `tagsupport/minipage|parbox/before|after` (its :47-50), which
//! tagpdfdocu-patches.sty:146-149 `\AssignSocketPlug{tagsupport/minipage/before}
//! {noop}` needs declared ("Socket undeclared!", latex.ltx:7405). Since the
//! kernel of LaTeX 2026-06-01 declares them itself (latex.ltx:22091-22094) and
//! module 0.81k (2026-04-24) only defines their `kernel`/`tag/dflt` plugs and
//! the `\@iiiminipage`/`\@iiiparbox` rewrites, re-declaring them raised four
//! `LaTeX socket Error: Socket '…' already declared!` on every
//! `\DocumentMetadata` document (pdflatex clean). The plugs and rewrites are
//! PDF-structure only and stay with our minipage, so the binding is empty; it
//! still shadows the raw module.
//! Guards: `perfect_kernel_batch56::testphase_tagging_sockets_and_block_templates_are_declared`,
//! `perfect_kernel_batch56::documentmetadata_tagging_sockets_are_declared_once`.
use crate::prelude::*;

LoadDefinitions!({});
