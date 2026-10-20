//! Binding for `ctex.sty` — the CTeX Chinese typesetting package (`\zihao`, `\CTEXsetup`, Chinese `\contentsname`
//! and `\today`, heading and indent defaults), which loads xeCJK under XeTeX.
//!
//! Perl LaTeXML ships no `ctex.sty.ltxml` binding and, at the default `notex` setting, does not raw-load the real
//! file; under `--preload=latexml.sty` (no `rawstyles`) the package was skipped with `Warning:missing_file`.
//! Raw-loads the genuine TL file. Under the default (pdfTeX) persona ANY `\usepackage{ctex}` errors, as under
//! pdflatex: with no `fontset` option ctex.sty:830-841 detects the platform (`fandol`, or `mac` where
//! `/System/Library/Fonts/Menlo.ttc` exists), and the fontset file is unavailable under pdfTeX
//! (`Critical Package ctex Error: CTeX fontset … is unavailable in current mode`, ctex.sty:802); `\msg_critical` ends
//! only the current file (`\tex_endinput:D`, expl3-code.tex:11101-11107), so pdflatex continues as this engine does
//! (`\zihao` is defined). One difference: on the mac fontset path pdflatex reports a second error (`File ended while
//! scanning use of \use_none_delimit_by_q_stop:w`, ctex-fontset-mac.def l.90), where this engine reports one
//! (SYNC_STATUS.md). The xetex persona
//! (`--preload=[xetex]latexml.sty`, which the corpus protocol gives documents loading ctex or xeCJK) loads it clean,
//! xeCJK reaching `xecjk_sty.rs`.
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("ctex", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
