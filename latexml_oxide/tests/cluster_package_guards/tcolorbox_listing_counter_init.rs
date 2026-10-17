//! `\newtcblisting[<init>]` runs tcolorbox's own init processor at definition
//! time. tcolorbox 6.10.0 (TeX Live 2026, tlnet 2026-10-16) renamed that
//! processor `\tcb@proc@options@init` -> `\__tcobox_proc_options_init:nn`
//! (tcolorbox.sty:2472, called from `\__tcobox_process_newtcolorbox:nn` :2556)
//! and dropped the per-environment `\thetcb@cnt@<env>` alias; `\thetcbcounter`
//! is now `the\tcbcounter`, resolved at each use (:2428-2435). RED: the binding
//! (`tcb_listing_startend`) still called the removed initializer, so every
//! listing env with `use counter=` got `Error:undefined:\tcb@proc@options@init`,
//! `\thetcbcounter` undefined and `\tcb@cnt@<env>` csname errors, and the
//! `listing file=` was never written.

fn count(stderr: &str, kind: &str) -> usize {
  let re = regex::Regex::new(&format!(r"^{kind}:[^:\s]+:")).unwrap();
  stderr.lines().filter(|l| re.is_match(l)).count()
}

#[test]
fn listing_use_counter_writes_file_and_inputs_it_back() {
  let (stderr, xml) = super::convert(
    r"\documentclass{article}
\usepackage{tcolorbox}
\tcbuselibrary{listings}
\newcounter{example}
\newtcblisting[use counter=example]{demo}{listing only,listing file={\jobname.\thetcbcounter.listing}}
\begin{document}
\begin{demo}
roundtrip\_marker
\end{demo}
\input{t.1.listing}
\end{document}
",
    true,
  );
  assert_eq!(count(&stderr, "Error"), 0, "{stderr}");
  assert_eq!(count(&stderr, "Fatal"), 0, "{stderr}");
  assert_eq!(count(&stderr, "Warning"), 0, "{stderr}");
  // The listing file named by `\thetcbcounter` (= 1) was written, so the
  // `\input` finds it.
  assert!(
    !stderr.contains("missing_file"),
    "listing file not written:\n{stderr}"
  );
  // In the box the listing keeps the source verbatim (`\_` stays a backslash and
  // `_marker`); the `\input` of the written file typesets it.
  assert!(
    xml.contains(
      r#"<text class="ltx_lst_identifier">roundtrip</text>\<text class="ltx_lst_identifier">_marker</text>"#
    ),
    "the box did not carry the verbatim listing:\n{xml}"
  );
  assert!(
    xml.contains("<p>roundtrip_marker</p>"),
    "the written listing did not replay via \\input:\n{xml}"
  );
}

/// tcolorbox 6.10.0 builds `number within=` with the kernel's
/// `\counterwithin[<format>]` (tcolorbox.sty:2411); the binding took no
/// optional format and read `[` as the counter (`Counter '' was not defined`,
/// then `readBalanced` ran out of input). latex.ltx:10374-10382: default
/// `\arabic`, and `\counterwithout` takes the same optional.
#[test]
fn counterwithin_and_counterwithout_take_a_format() {
  let (stderr, xml) = super::convert(
    r"\documentclass{article}
\newcounter{foo}\newcounter{bar}\newcounter{baz}
\counterwithin[ {\Roman} ]{foo}{section}
\counterwithin{bar}{section}
\counterwithin{baz}{section}
\counterwithout[\Alph]{baz}{section}
\begin{document}
\section{A}
\stepcounter{foo}\stepcounter{bar}\stepcounter{baz}
fmt=[\thefoo;\thebar;\thebaz]
\end{document}
",
    true,
  );
  assert_eq!(count(&stderr, "Error"), 0, "{stderr}");
  assert_eq!(count(&stderr, "Fatal"), 0, "{stderr}");
  assert_eq!(count(&stderr, "Warning"), 0, "{stderr}");
  assert!(
    xml.contains("fmt=[1.I;1.1;A]"),
    "formats not applied:\n{xml}"
  );
}
