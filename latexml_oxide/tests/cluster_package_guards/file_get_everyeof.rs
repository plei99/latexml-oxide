//! `\everyeof` at the end of an `\input` file whose content is consumed before
//! `\input` returns (a binding, or a file read as definitions).

use super::perfect_kernel_batch46::{convert_files, error_count};

/// expl3's `\file_get:nnN` (expl3-code.tex:12666-12705) runs its setup in a
/// group, `\tex_input:D`s the file and scans `\__file_get_do:Nw` up to the
/// `\everyeof` marker, whose `\group_end:` undoes the setup. LaTeX 2026-06-01
/// does this while the format is built (latex.ltx:22508-22513, setup
/// `\ExplSyntaxOff`) to `glyphtounicode` (a contrib binding) and
/// `glyphtounicode-cmex` (raw), both reached while reading definitions. RED: no
/// `\everyeof` was inserted after either, the scan ran past the file,
/// `\ExplSyntaxOff` stayed on and the following expl3 code read as `\tl` `_` …
/// (25 errors here; `--init=latex.ltx` logged 137). Covered: the binding read as
/// content (preamble), the binding and the raw file read as definitions (a
/// package; the format build's mode).
///
/// pdflatex prints `[doc full 2][pkg/full/full/5]`: `\file_get` captures the
/// file. Here every capture is empty, a known gap: the binding has no content
/// (glyphtounicode_tex.rs), and a raw file read as definitions is digested
/// rather than handed to the scan (Perl Package.pm:2287-2288 alike;
/// `docs/SYNC_STATUS.md`). What only the fix yields is the scan ending at the
/// file, so expl3 syntax is intact after each call and `\int_eval:n` evaluates.
#[test]
fn file_get_on_a_binding_ends_at_the_marker() {
  let (stderr, xml) = convert_files(
    r"\documentclass{article}
\usepackage{fgpkg}
\ExplSyntaxOn
\file_get:nnN {glyphtounicode} {\ExplSyntaxOff} \l_tmpa_tl
\tl_gset:Ne \g_tmpa_tl {doc~\tl_if_empty:NTF \l_tmpa_tl {empty} {full}~\int_eval:n {1+1}}
\ExplSyntaxOff
\begin{document}
\ExplSyntaxOn [\tl_use:N \g_tmpa_tl] [\tl_use:N \g_tmpb_tl] \ExplSyntaxOff
\end{document}
",
    &[(
      "fgpkg.sty",
      r"\ProvidesPackage{fgpkg}
\ExplSyntaxOn
\file_get:nnN {glyphtounicode-cmex} {\ExplSyntaxOff} \l_tmpb_tl
\file_get:nnN {glyphtounicode} {\ExplSyntaxOff} \l_tmpc_tl
\tl_gset:Ne \g_tmpb_tl {pkg/\tl_if_empty:NTF \l_tmpb_tl {empty} {full}
  /\tl_if_empty:NTF \l_tmpc_tl {empty} {full}/\int_eval:n {2+3}}
\ExplSyntaxOff
",
    )],
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(stderr.matches("Warning:").count(), 0, "{stderr}");
  assert!(xml.contains("[doc empty 2]"), "{xml}");
  assert!(xml.contains("[pkg/empty/empty/5]"), "{xml}");
}
