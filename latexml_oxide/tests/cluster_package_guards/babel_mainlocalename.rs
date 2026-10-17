//! babel 26.14 (TL 2026-10-16) expands `\mainlocalename` in a `begindocument/end` hook
//! (babel.sty:4500-4501, "Main language set to '…'"). Babel sets it in `\main@language`
//! (babel.sty:1208), which our Rust-bound `french`/`german` `.ldf`s never reach, so every
//! such document raised `Error:undefined:\mainlocalename`. `\lx@babel@activate@mainlang`
//! now snapshots it from `\bbl@main@language` with `\let` semantics, as babel does.
use super::perfect_kernel_batch46::{error_count, warning_count};

fn convert(options: &str, body: &str) -> (String, String) {
  latexml::util::test::convert_with(
    &format!(
      "\\documentclass{{article}}\\usepackage[{options}]{{babel}}\\makeatletter\n\\begin{{document}}\n{body}\n\\end{{document}}\n"
    ),
    None,
  )
}

/// pdflatex 2026 (babel 26.14): `[french]` logs "Main language set to 'french'", 0 errors; the page text
/// is `A[french][french]`.
#[test]
fn french_main_language_is_defined() {
  let (log, xml) = convert("french", r"A[\mainlocalename][\languagename]");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains("A[french][french]"), "{xml}");
  assert!(log.contains("Main language set to 'french'"), "{log}");
}

#[test]
fn german_main_language_is_defined() {
  let (log, xml) = convert("ngerman", r"A[\mainlocalename][\languagename]");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains("A[ngerman][ngerman]"), "{xml}");
}

/// The main language is the LAST option (babel's rule), and `\selectlanguage` moves `\languagename`
/// but not `\mainlocalename` (a `\let` snapshot). pdflatex 2026 prints `A[french][french]B[french][english]`
/// for `[english,french]` with a switch to english between them.
#[test]
fn selectlanguage_does_not_move_the_main_language() {
  let (log, xml) = convert(
    "english,french",
    r"A[\mainlocalename][\languagename]\selectlanguage{english}B[\mainlocalename][\languagename]",
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  // The switch to english wraps its text in `<text xml:lang="en">`, so the two halves are checked apart.
  assert!(xml.contains("A[french][french]"), "{xml}");
  assert!(xml.contains("B[french][english]"), "{xml}");
  assert!(log.contains("Main language set to 'french'"), "{log}");
}
