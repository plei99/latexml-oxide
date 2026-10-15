//! biblatex's alphabetic styles label from the `.bib` when no `.bbl` is beside the
//! source: `style=alphabetic` printed numeric `[1]` (the binding left `CITE_STYLE`
//! at the kernel's `numbers`; MakeBibliography's alphabetic generator is Perl's
//! `[AS64]` one, not biblatex's). The label is biber's `labelalpha` + `extraalpha`
//! (biblatex.def:1416-1436 `\DeclareLabelalphaTemplate` / `...NameTemplate`;
//! biber `Internals.pm` `_genlabel`, `_labelpart`, `_label_name`): one visible name
//! gives the first three letters of its family name, several the first letter of
//! each, the year its last two digits; a list longer than `maxalphanames` (3) shows
//! `minalphanames` (1) of them and `labelalphaothers` (`+`); equal labels get
//! `\mknumalph` letters in list order. A `.bbl`'s label (biber's own) wins.
//! The citations print the bibitem's label, so one assertion on each covers both.
use std::process::Command;

use super::perfect_kernel_batch46::{error_count, warning_count};

const RAW: &str = "--preload=[rawstyles,rawclasses]latexml.sty";

const REPRO_TEX: &str =
  include_str!("../../../tools/perfect_kernel/repros/index-bib/biblatex_alphabetic_labels.tex");
const REPRO_BIB: &str =
  include_str!("../../../tools/perfect_kernel/repros/index-bib/biblatex_alphabetic_labels.bib");

/// The `.bib` the documents name (`\addbibresource`).
const BIB_NAME: &str = "biblatex_alphabetic_labels.bib";

/// Convert `t.tex` (with side `files`) to HTML in one process. Returns
/// (ANSI-stripped stderr, HTML).
fn convert_html(tex: &str, files: &[(&str, &str)]) -> (String, String) {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  for (name, content) in files {
    std::fs::write(workdir.path().join(name), content).expect("write side file");
  }
  let output = Command::new(bin)
    .args([
      "t.tex",
      "--dest",
      "t.html",
      "--nocomments",
      "--timeout=110",
      RAW,
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  let html = std::fs::read_to_string(workdir.path().join("t.html")).unwrap_or_default();
  (stderr, html)
}

/// The text between `open` and the next `]</` of every occurrence of `open` in `html`.
fn bracketed_after(html: &str, open: &str) -> Vec<String> {
  html
    .split(open)
    .skip(1)
    .map(|rest| rest.split("]</").next().unwrap_or_default().to_string())
    .collect()
}

/// The labels the References list prints, in list order.
fn bibitem_labels(html: &str) -> Vec<String> {
  bracketed_after(html, r#"ltx_bib_abbrv ltx_role_refnum ltx_tag_bibitem">["#)
}

/// The numbers the References list prints, in list order.
fn numeric_labels(html: &str) -> Vec<String> {
  bracketed_after(html, r#"ltx_bib_key ltx_role_refnum ltx_tag_bibitem">["#)
}

/// The text of every citation link (a link to a bibitem), in document order.
fn citation_texts(html: &str) -> Vec<String> {
  html
    .split(r##"<a href="#bib."##)
    .skip(1)
    .filter_map(|rest| rest.split_once(r#"class="ltx_ref">"#))
    .map(|(_, rest)| rest.split("</a>").next().unwrap_or_default().to_string())
    .collect()
}

/// `\documentclass{article}\usepackage[<options>]{biblatex}` over [`BIB_NAME`],
/// citing `keys` and printing the bibliography.
fn document(options: &str, preamble: &str, keys: &str) -> String {
  format!(
    "\\documentclass{{article}}\n\\usepackage[{options}]{{biblatex}}\n\\addbibresource{{biblatex_alphabetic_labels.bib}}\n{preamble}\\begin{{document}}\n\\cite{{{keys}}}\n\\printbibliography\n\\end{{document}}\n"
  )
}

/// Convert `tex` over `bib`, require a clean run, and return the HTML.
fn clean_html(tex: &str, bib: &str) -> String {
  let (stderr, html) = convert_html(tex, &[(BIB_NAME, bib)]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  html
}

/// The reporter's reproducer: two authors, 2024 — "[AB24]" in the citation and
/// in the list, not "[1]". The reporter's document is the repro `biblatex_alphabetic_labels.tex`.
#[test]
fn alphabetic_style_without_bbl_labels_alphabetically() {
  let html = clean_html(REPRO_TEX, REPRO_BIB);
  assert_eq!(bibitem_labels(&html), ["AB24"], "{html}");
  assert_eq!(citation_texts(&html), ["AB24"], "{html}");
  assert!(numeric_labels(&html).is_empty(), "{html}");
}

/// `alphabetic-verb` is the same label with its own `\cite` shape
/// (alphabetic-verb.cbx:12-39, brackets and `\iflastcitekey`).
#[test]
fn alphabetic_verb_style_labels_alphabetically() {
  let html = clean_html(&document("style=alphabetic-verb", "", "example"), REPRO_BIB);
  assert_eq!(bibitem_labels(&html), ["AB24"], "{html}");
  assert_eq!(citation_texts(&html), ["AB24"], "{html}");
}

/// `bibstyle=` and `citestyle=` given apart (biblatex.sty:16439-16440, bbx then
/// cbx; both enable `labelalpha`, alphabetic.bbx:5 / alphabetic.cbx:4).
#[test]
fn alphabetic_bibstyle_and_citestyle_combination_labels_alphabetically() {
  let html = clean_html(
    &document("bibstyle=alphabetic,citestyle=alphabetic", "", "example"),
    REPRO_BIB,
  );
  assert_eq!(bibitem_labels(&html), ["AB24"], "{html}");
  assert_eq!(citation_texts(&html), ["AB24"], "{html}");
}

/// The labels of the user's notes, each rule at once: one author gives three
/// letters ("Giv96"), three give one initial each ("LLY97"), a literal "and
/// others" is a truncation ("Zet+20" — `morenames`, Internals.pm:633), a missing
/// year leaves the name alone ("Nog"), a missing author falls back to the
/// editor ("Edi05"), a family-name prefix is not in the label unless
/// `useprefix` ("Neu50"; Internals.pm:575-579), and the year is its last two
/// digits ("Wu01", "Smi99"). The list is in label order (`sorting=anyt`,
/// biblatex.def:1723-1731 sorts on `labelalpha` first).
#[test]
fn alphabetic_label_rules() {
  let bib = "\
@book{giv, author = {Given, Alice}, title = {A}, year = {1996}, publisher = {P}}
@book{lly, author = {Lee, A. and Lin, B. and Young, C.}, title = {B}, year = {1997}, publisher = {P}}
@book{zet, author = {Zeta, A. and others}, title = {C}, year = {2020}, publisher = {P}}
@book{nog, author = {Nogood, Nina}, title = {D}, publisher = {P}}
@book{edi, editor = {Editor, Ed}, title = {E}, year = {2005}, publisher = {P}}
@book{neu, author = {von Neumann, John}, title = {F}, year = {1950}, publisher = {P}}
@book{wu, author = {Wu, X.}, title = {G}, year = {2001}, publisher = {P}}
@book{hyp, author = {Smith-Jones, Pat}, title = {H}, year = {1999}, publisher = {P}}
";
  let html = clean_html(
    &document("style=alphabetic", "", "giv,lly,zet,nog,edi,neu,wu,hyp"),
    bib,
  );
  assert_eq!(
    bibitem_labels(&html),
    [
      "Edi05", "Giv96", "LLY97", "Neu50", "Nog", "Smi99", "Wu01", "Zet+20"
    ],
    "{html}"
  );
}

/// The year part is the last two characters of the year field, digits or not
/// (`strwidth=2, strside=right`, biblatex.def:1424): biber labels `year =
/// {forthcoming}` "Yeang".
#[test]
fn alphabetic_label_year_is_the_last_two_characters() {
  let bib = "@book{y, author = {Yeats, W.}, title = {Y}, year = {forthcoming}, publisher = \
             {P}}\n";
  let html = clean_html(&document("style=alphabetic", "", "y"), bib);
  assert_eq!(bibitem_labels(&html), ["Yeang"], "{html}");
}

/// `extraalpha` is biber's per refsection (Biber.pm:4058-4066), not per printed list. The
/// binding applies no list filter: `\printbibliography[type=book]` prints the one whole list
/// (`biblatex_print_filtered`), whose `a`/`b` are the refsection's; a plain second
/// `\printbibliography` repeats the list (`\biblatex@repeated@bibliography`) with the same labels.
#[test]
fn alphabetic_extraalpha_agrees_across_printed_lists() {
  let bib = "@book{b, author = {Smith, John}, title = {Beta}, year = {2010}, publisher = {P}}
@article{a, author = {Smith, Jane}, title = {Alpha}, journal = {J}, year = {2010}}
";
  let tex = document("style=alphabetic", "", "b,a");
  let filtered = tex.replace(
    "\\printbibliography\n",
    "\\printbibliography[type=book]\n\\printbibliography[type=article]\n",
  );
  let html = clean_html(&filtered, bib);
  assert_eq!(bibitem_labels(&html), ["Smi10a", "Smi10b"], "{html}");
  let twice = tex.replace(
    "\\printbibliography\n",
    "\\printbibliography\n\\printbibliography\n",
  );
  let html = clean_html(&twice, bib);
  assert_eq!(
    bibitem_labels(&html),
    ["Smi10a", "Smi10b", "Smi10a", "Smi10b"],
    "{html}"
  );
}

/// `useprefix=true` puts the prefix's first letter before the family name
/// (the default name template's `pre=true, strwidth=1`, biblatex.def:1432-1435).
#[test]
fn alphabetic_label_with_useprefix() {
  let bib = "@book{neu, author = {von Neumann, John}, title = {F}, year = {1950}, publisher = \
             {P}}\n@book{two, author = {von Neumann, John and de la Cruz, Maria}, title = {G}, \
             year = {1950}, publisher = {P}}\n";
  let html = clean_html(
    &document("style=alphabetic,useprefix=true", "", "neu,two"),
    bib,
  );
  assert_eq!(bibitem_labels(&html), ["vNdlC50", "vNeu50"], "{html}");
}

/// Four authors under the default `maxalphanames=3` are a truncation — one
/// visible name (`minalphanames=1`), so the three-letter form and `+`:
/// "Alp+24" (the user's "Ale+24"); with `maxalphanames=4` all four show, one
/// initial each: "ACGS24".
#[test]
fn alphabetic_maxalphanames() {
  let bib = "@book{four, author = {Alpha, A. and Cole, B. and Gamma, C. and Shaw, D.}, title = \
             {Four}, year = {2024}, publisher = {P}}\n";
  let html = clean_html(&document("style=alphabetic", "", "four"), bib);
  assert_eq!(bibitem_labels(&html), ["Alp+24"], "{html}");
  assert_eq!(citation_texts(&html), ["Alp+24"], "{html}");
  let html = clean_html(
    &document("style=alphabetic,maxalphanames=4", "", "four"),
    bib,
  );
  assert_eq!(bibitem_labels(&html), ["ACGS24"], "{html}");
  assert_eq!(citation_texts(&html), ["ACGS24"], "{html}");
  // `\ExecuteBibliographyOptions` sets it as the package option does.
  let html = clean_html(
    &document(
      "style=alphabetic",
      "\\ExecuteBibliographyOptions{maxalphanames=4}\n",
      "four",
    ),
    bib,
  );
  assert_eq!(bibitem_labels(&html), ["ACGS24"], "{html}");
}

/// A truncated list shows `minalphanames` names then `+`: five authors with
/// `minalphanames=2` are "AB+24"; with the default one name, "Ale+24".
#[test]
fn alphabetic_minalphanames_and_truncation_plus() {
  let bib = "@book{many, author = {Alexander, A. and Beta, B. and Gamma, C. and Delta, S. and \
             Eps, E.}, title = {Many}, year = {2024}, publisher = {P}}\n";
  let html = clean_html(&document("style=alphabetic", "", "many"), bib);
  assert_eq!(bibitem_labels(&html), ["Ale+24"], "{html}");
  let html = clean_html(
    &document("style=alphabetic,minalphanames=2", "", "many"),
    bib,
  );
  assert_eq!(bibitem_labels(&html), ["AB+24"], "{html}");
}

/// Equal labels are told apart by `extraalpha` in list order (Biber.pm:4058-4066;
/// printed through `\mknumalph`, biblatex.def:476): the list is sorted by label,
/// then name, year and title (biblatex.def:1723-1748), so "Smith, Jane"
/// ("Smi10a") precedes "Smith, John" ("Smi10b"); the singleton has none.
#[test]
fn alphabetic_extraalpha_for_equal_labels() {
  let bib = "\
@book{b, author = {Smith, John}, title = {Beta}, year = {2010}, publisher = {P}}
@book{a, author = {Smith, Jane}, title = {Alpha}, year = {2010}, publisher = {P}}
@book{c, author = {Smith, Jill}, title = {Gamma}, year = {2011}, publisher = {P}}
";
  let html = clean_html(&document("style=alphabetic", "", "b,a,c"), bib);
  assert_eq!(
    bibitem_labels(&html),
    ["Smi10a", "Smi10b", "Smi11"],
    "{html}"
  );
  assert_eq!(
    citation_texts(&html),
    ["Smi10b", "Smi10a", "Smi11"],
    "{html}"
  );
}

/// A numeric style stays numeric (the default style, `style=numeric`), and so
/// does an author-year one stay author-year.
#[test]
fn numeric_style_stays_numeric() {
  for options in ["style=numeric", "backend=biber"] {
    let html = clean_html(&document(options, "", "example"), REPRO_BIB);
    assert_eq!(numeric_labels(&html), ["1"], "{options}: {html}");
    assert_eq!(citation_texts(&html), ["1"], "{options}: {html}");
    assert!(bibitem_labels(&html).is_empty(), "{options}: {html}");
  }
}

/// A `.bbl` beside the source is biber's output and its label is the one
/// printed: the `.bib` is not consulted. Here the `.bbl`'s label is not what the
/// `.bib` would give.
#[test]
fn bbl_label_wins_over_the_computed_one() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/index-bib/biblatex_bbl_alphabetic.tex");
  let bbl =
    include_str!("../../../tools/perfect_kernel/repros/index-bib/biblatex_bbl_alphabetic.bbl")
      .replace("{labelalpha}{Ada+01}", "{labelalpha}{Zzz+99}");
  assert!(bbl.contains("{labelalpha}{Zzz+99}"), "fixture changed");
  let (stderr, html) = convert_html(tex, &[("t.bbl", &bbl)]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  let labels = bibitem_labels(&html);
  assert!(labels.contains(&"Zzz+99".to_string()), "{labels:?}\n{html}");
  assert!(
    !labels.iter().any(|label| label.starts_with("Ada")),
    "{labels:?}\n{html}"
  );
}
