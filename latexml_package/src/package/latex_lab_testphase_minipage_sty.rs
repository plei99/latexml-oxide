//! latex-lab-testphase-minipage.sty — nothing left to declare.
//!
//! `\DocumentMetadata{tagging=on}` loads `latex-lab-testphase-latest.sty`
//! (documentmetadata-support.ltx:74), which pulls in this module (:51).
//! Through latex-lab 0.81g the module declared the four sockets
//! `tagsupport/minipage|parbox/before|after` (its :47-50), which
//! tagpdfdocu-patches.sty `\AssignTaggingSocketPlug{minipage/before}{noop}`
//! (:113-114 in TL 2026; :146-149 `\AssignSocketPlug` before) needs declared
//! ("Socket undeclared!", latex.ltx:7432 in `\socket_assign_plug:nn`; tagpdf
//! manual: 88 socket error lines; SHARED, pdflatex clean). Since the
//! kernel of LaTeX 2026-06-01 declares them itself (latex.ltx:22091-22094) and
//! module 0.81k (2026-04-24) only defines their `kernel`/`tag/dflt` plugs and
//! the `\@iiiminipage`/`\@iiiparbox` rewrites, re-declaring them raised four
//! `LaTeX socket Error: Socket '…' already declared!` on every
//! `\DocumentMetadata` document (pdflatex clean). The plugs and rewrites are
//! PDF-structure only and stay with our minipage, so the binding is empty; it
//! still shadows the raw module. The binding relies on the 2026-06-01 kernel
//! declaring the sockets; on an older kernel
//! `\IfSocketExistsF{…}{\NewTaggingSocket{…}{0}}` (`\IfSocketExistsF`,
//! latex.ltx:7498-7500) would be the portable form.
//! Guards: `perfect_kernel_batch56::testphase_tagging_sockets_and_block_templates_are_declared`,
//! `perfect_kernel_batch56::documentmetadata_does_not_redeclare_kernel_tagging_sockets`.
use crate::prelude::*;

LoadDefinitions!({});
