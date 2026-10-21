//! biblatex's standard bibliography drivers (OXIDIZED_DESIGN_DIVERGENCES #601).
//!
//! A document in `style=alphabetic` or `alphabetic-verb`, whose drivers are standard.bbx's, gets its References as
//! pdflatex + biber print them, not in LaTeXML's rows (`Author (2008)`, "External Links:", "Cited by:"): standard.bbx:28-733
//! drivers over the punctuation tracker of biblatex.sty:4130-4260, the formats of biblatex.def:450-600. The expected
//! text of every entry below is what pdflatex + biber printed for the same `.bib`; the citations separate their keys
//! with alphabetic.cbx's `\multicitedelim` (biblatex.def:97); the heading is biblatex.def:2184-2239's choice from the
//! class. Native protocol (`localrawstyles`), the one a site that converts its own sources uses.
use std::process::Command;

use super::perfect_kernel_batch46::{error_count, warning_count};

const NATIVE: &str = "--preload=[localrawstyles]latexml.sty";

/// An article (with the MathReview fields the layout must not print), an incollection and an arXiv `@misc`, in the
/// shape of a MathSciNet export.
const BIB: &str = r#"@article{gwdmstack,
  author = {Abramovich, Dan and Graber, Tom and Vistoli, Angelo},
  title = {Gromov-{W}itten theory of {D}eligne-{M}umford stacks},
  journal = {Amer. J. Math.},
  fjournal = {American Journal of Mathematics},
  volume = {130},
  year = {2008},
  number = {5},
  pages = {1337--1398},
  issn = {0002-9327,1080-6377},
  mrclass = {14N35 (14A20 53D45)},
  mrnumber = {2450211},
  mrreviewer = {Johannes\ Walcher},
  doi = {10.1353/ajm.0.0017},
}

@incollection{giventalmirror,
  author = {Givental, Alexander},
  title = {A mirror theorem for toric complete intersections},
  booktitle = {Topological field theory, primitive forms and related topics
               ({K}yoto, 1996)},
  series = {Progr. Math.},
  volume = {160},
  pages = {141--175},
  publisher = {Birkh\"{a}user Boston, Boston, MA},
  year = {1998},
  isbn = {0-8176-3975-6},
  mrnumber = {1653024},
}

@misc{mirrornonconvex,
  title = {A mirror theorem for Gromov-Witten theory without convexity},
  author = {Jun Wang},
  year = {2020},
  eprint = {1910.14440},
  archiveprefix = {arXiv},
  primaryclass = {math.AG},
}
"#;

/// Entries for the option guards: four authors, a book with an edition and a page total, an `@online` entry.
const OPTIONS_BIB: &str = r#"@article{four, author = {Smith, John and Doe, Jane and Roe, Richard and Poe, Edgar}, title = {Four authors}, journal = {J. Alg.}, volume = {1}, year = {2001}, pages = {5--9}, issn = {1111-2222}, doi = {10.1/x}}
@book{bk, author = {Hartshorne, Robin}, title = {Algebraic Geometry}, publisher = {Springer}, location = {New York}, series = {Graduate Texts in Mathematics}, number = {52}, year = {1977}, isbn = {0-387-90244-9}, pagetotal = {496}, edition = {2}}
@online{web, author = {On, Ona}, title = {Online thing}, url = {https://example.org/o}, urldate = {2021-05-17}, year = {2021}}
"#;

/// Entries for the driver details: no `type` on a techreport, an edition on a proceedings volume, an article
/// without a date, an eprint without a type, a manual with an editor and a translator, eprint types with and
/// without a format of their own.
const DETAILS_BIB: &str = r#"@techreport{t3, author={Rep, Rita}, title={Typeless report}, institution={Univ}, number={RR-6}, year={2002}}
@proceedings{pr2, editor={Ed, Edgar}, title={Edition proceedings}, edition={2}, year={1995}, publisher={Pub}}
@article{a7, author={Dated, Dan}, title={No year}, journal={J. Nodate}, volume={1}}
@misc{m2, author={Mis, Mike}, title={Untyped eprint}, eprint={2999.12345}, eprintclass={math.XX}, year={2007}}
@manual{m3, editor={Ed, Edgar}, translator={Trans, Tom}, title={Manual with editor}, organization={Org}, year={2003}}
@misc{m4, author={Mis, Mike}, title={Hdl eprint}, eprint={2013/ABC}, eprinttype={hdl}, year={2007}}
@misc{m5, author={Mis, Mike}, title={Other eprint}, eprint={xyz-42}, eprinttype={foo}, year={2007}}
"#;

/// Entries for the details of series, issues, translators, languages, publication states and title formats; the
/// expected texts are pdflatex + biber's for this file.
const FIX_BIB: &str = r#"@article{iss, author={Aa, Ann}, title={Issue article}, journaltitle={J. Iss}, issuetitle={Special Issue on Things}, issue={Spring}, volume={9}, year={2015}, pages={1--2}}
@article{ser, author={Bb, Bob}, title={Series article}, journaltitle={J. Ser.}, series={3}, volume={9}, year={2016}}
@article{ser2, author={Cc, Cy}, title={Text series article}, journaltitle={J. Tser}, series={newseries}, volume={9}, year={2016}}
@book{tro, translator={Trans, Tom}, title={Translated Only}, publisher={Pub}, year={2000}}
@article{tra, translator={Trans, Tom}, title={Translated article}, journaltitle={J. T}, year={2000}}
@incollection{tri, translator={Trans, Tom}, title={Translated incoll}, booktitle={Big Book}, publisher={Pub}, year={2000}}
@book{la1, author={Ll, Lee}, title={Lang english}, language={english}, publisher={Pub}, year={2001}}
@book{la2, author={Ll, Lia}, title={Lang en}, language={en}, publisher={Pub}, year={2001}}
@book{la3, author={Ll, Lio}, title={Lang french}, language={french}, publisher={Pub}, year={2001}}
@book{la4, author={Ll, Lou}, title={Lang enUS}, language={en-US}, publisher={Pub}, year={2001}}
@book{la5, author={Ll, Luc}, title={Lang langgerman}, language={langgerman}, publisher={Pub}, year={2001}}
@book{la6, author={Ll, Lyn}, title={Lang list}, language={english and french}, publisher={Pub}, year={2001}}
@book{ps1, author={Pp, Pat}, title={Pub inpress}, pubstate={inpress}, publisher={Pub}, year={2002}}
@book{ps2, author={Pp, Pam}, title={Pub submitted}, pubstate={submitted}, publisher={Pub}, year={2002}}
@book{ps3, author={Pp, Pia}, title={Pub forthcoming}, pubstate={forthcoming}, publisher={Pub}, year={2002}}
@book{ps4, author={Pp, Pol}, title={Pub other}, pubstate={Draft text}, publisher={Pub}, year={2002}}
@book{ed1, editor={Same, Sam}, translator={Same, Sam}, title={Same Person}, publisher={Pub}, year={2003}}
@book{ed2, author={Au, Al}, editor={Same, Sam}, translator={Same, Sam}, title={Author and Same}, publisher={Pub}, year={2003}}
@inbook{ed3, author={Au, Ab}, editor={Same, Sam}, translator={Same, Sam}, title={Chapter same}, booktitle={The Book}, publisher={Pub}, year={2003}}
@review{rv1, author={Rr, Rob}, title={A review}, journaltitle={J. Rev}, year={2004}}
@bookinbook{bb1, author={Bk, Bea}, title={Book in book}, booktitle={Outer}, publisher={Pub}, year={2004}}
@suppbook{sb1, author={Sb, Sue}, title={Supp book}, booktitle={Outer2}, publisher={Pub}, year={2004}}
@book{lb1, author={Qa, Quin}, title={Lang Capital}, language={English}, publisher={Pub}, year={2005}}
@book{lb2, author={Qb, Quin}, title={Lang british}, language={british}, publisher={Pub}, year={2005}}
@book{lb3, author={Qc, Quin}, title={Lang USenglish}, language={USenglish}, publisher={Pub}, year={2005}}
@book{lb4, author={Qd, Quin}, title={Lang two english}, language={english and american}, publisher={Pub}, year={2005}}
@book{lb5, author={Qe, Quin}, title={Lang FRENCH}, language={FRENCH}, publisher={Pub}, year={2005}}
@book{lb6, author={Qf, Quin}, title={Lang american}, language={american}, publisher={Pub}, year={2005}}
@manual{mn1, editor={Twin, Tia}, translator={Twin, Tia}, title={Manual twin}, organization={Org}, year={2003}}
"#;

/// Convert `t.tex` (with side `files`) to HTML. Returns (ANSI-stripped stderr, HTML).
fn convert(tex: &str, files: &[(&str, &str)]) -> (String, String) {
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
      NATIVE,
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  let html = std::fs::read_to_string(workdir.path().join("t.html")).unwrap_or_default();
  (stderr, html)
}

/// A clean conversion: no error and no warning.
fn clean_html(tex: &str, bib: &str) -> String {
  let (stderr, html) = convert(tex, &[("refs.bib", bib)]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  html
}

/// `\documentclass{<class>}\usepackage[<options>]{biblatex}` over `refs.bib`, with `body` after `\begin{document}`.
fn document(class: &str, options: &str, body: &str) -> String {
  format!(
    "\\documentclass{{{class}}}\n\\usepackage[backend=biber,{options}]{{biblatex}}\n\\addbibresource{{refs.bib}}\n\\begin{{document}}\n{body}\n\\end{{document}}\n"
  )
}

/// The `ltx:bibitem` (HTML `li`) labelled `[label]`, whole.
fn bibitem(html: &str, label: &str) -> String {
  let tag = format!(">[{label}]</span>");
  let at = html
    .find(&tag)
    .unwrap_or_else(|| panic!("no bibitem [{label}] in {html}"));
  let start = html[..at].rfind("<li ").expect("li before the label");
  let end = at + html[at..].find("</li>").expect("end of the li") + "</li>".len();
  html[start..end].to_string()
}

/// The printed text of every bibitem, white space collapsed.
fn bibitem_texts(html: &str) -> Vec<String> {
  html
    .split("<li ")
    .skip(1)
    .map(|item| {
      let item = item.split("</li>").next().unwrap_or_default();
      let mut text = String::new();
      // The split removed the `<li`: its attributes are in the tag still open.
      let mut in_tag = true;
      for c in item.chars() {
        match c {
          '<' => in_tag = true,
          '>' => in_tag = false,
          c if !in_tag => text.push(c),
          _ => {},
        }
      }
      text
        .replace('\u{a0}', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
    })
    .collect()
}

/// The warnings of a conversion, each as `Warning:<category>:<object>`.
fn warnings(stderr: &str) -> Vec<String> {
  stderr
    .lines()
    .filter(|line| line.starts_with("Warning:"))
    .map(|line| {
      line
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_string()
    })
    .collect()
}

/// Every citation, whole.
fn cites(html: &str) -> Vec<String> {
  html
    .split("<cite ")
    .skip(1)
    .map(|rest| {
      format!(
        "<cite {}</cite>",
        rest.split("</cite>").next().unwrap_or_default()
      )
    })
    .collect()
}

/// The article of `standard.bbx`: names, the quoted title, `In:` the journal in italics, `volume.number (year)`,
/// the pages, `ISSN:` and `DOI:` in small capitals, a final period. No "External Links:", no MathReview link.
#[test]
fn article_is_printed_as_standard_bbx_prints_it() {
  let html = clean_html(
    &document(
      "article",
      "style=alphabetic",
      "\\nocite{*}\\printbibliography",
    ),
    BIB,
  );
  assert_eq!(
    bibitem(&html, "AGV08"),
    "<li id=\"bib.bib1\" class=\"ltx_bibitem ltx_bib_article\"><span class=\"ltx_tag ltx_bib_abbrv ltx_role_refnum ltx_tag_bibitem\">[AGV08]</span>\n<span class=\"ltx_bibblock\"><span class=\"ltx_text ltx_bib_author\">Dan Abramovich, Tom Graber, and Angelo Vistoli</span>. <span class=\"ltx_text ltx_bib_title\">“Gromov-Witten theory of Deligne-Mumford stacks”</span>. In: <span class=\"ltx_text ltx_bib_journal\"><em class=\"ltx_emph\">Amer. J. Math.</em></span> <span class=\"ltx_text ltx_bib_volume\">130</span>.<span class=\"ltx_text ltx_bib_number\">5</span> (<span class=\"ltx_text ltx_bib_date\">2008</span>), <span class=\"ltx_text ltx_bib_pages\">pp.\u{a0}1337–1398</span>. <span class=\"ltx_text ltx_bib_issn\"><span class=\"ltx_text ltx_font_smallcaps\">issn</span>: 0002-9327,1080-6377</span>. <span class=\"ltx_text ltx_bib_doi\"><span class=\"ltx_text ltx_font_smallcaps\">doi</span>: <a href=\"https://doi.org/10.1353/ajm.0.0017\" title=\"\" class=\"ltx_ref doi ltx_bib_external\">10.1353/ajm.0.0017</a></span>.\n</span></li>"
  );
}

/// The incollection: `In:` the italic book title, `Vol. 160`, the series, `Publisher, year`, the pages, `ISBN:`.
#[test]
fn incollection_is_printed_as_standard_bbx_prints_it() {
  let html = clean_html(
    &document(
      "article",
      "style=alphabetic",
      "\\nocite{*}\\printbibliography",
    ),
    BIB,
  );
  assert_eq!(
    bibitem(&html, "Giv98"),
    "<li id=\"bib.bib2\" class=\"ltx_bibitem ltx_bib_incollection\"><span class=\"ltx_tag ltx_bib_abbrv ltx_role_refnum ltx_tag_bibitem\">[Giv98]</span>\n<span class=\"ltx_bibblock\"><span class=\"ltx_text ltx_bib_author\">Alexander Givental</span>. <span class=\"ltx_text ltx_bib_title\">“A mirror theorem for toric complete intersections”</span>. In: <span class=\"ltx_text ltx_bib_booktitle\"><em class=\"ltx_emph\">Topological field theory, primitive forms and related topics (Kyoto, 1996)</em></span>. <span class=\"ltx_text ltx_bib_volume\">Vol.\u{a0}160</span>. <span class=\"ltx_text ltx_bib_series\">Progr. Math.</span> <span class=\"ltx_text ltx_bib_publisher\">Birkhäuser Boston, Boston, MA</span>, <span class=\"ltx_text ltx_bib_date\">1998</span>, <span class=\"ltx_text ltx_bib_pages\">pp.\u{a0}141–175</span>. <span class=\"ltx_text ltx_bib_isbn\"><span class=\"ltx_text ltx_font_smallcaps\">isbn</span>: 0-8176-3975-6</span>.\n</span></li>"
  );
}

/// An arXiv `@misc`: the italic title, the year, `arXiv: <id> [<class>]` as one link, no publisher.
#[test]
fn arxiv_misc_is_printed_as_standard_bbx_prints_it() {
  let html = clean_html(
    &document(
      "article",
      "style=alphabetic",
      "\\nocite{*}\\printbibliography",
    ),
    BIB,
  );
  assert_eq!(
    bibitem(&html, "Wan20"),
    "<li id=\"bib.bib3\" class=\"ltx_bibitem ltx_bib_misc\"><span class=\"ltx_tag ltx_bib_abbrv ltx_role_refnum ltx_tag_bibitem\">[Wan20]</span>\n<span class=\"ltx_bibblock\"><span class=\"ltx_text ltx_bib_author\">Jun Wang</span>. <span class=\"ltx_text ltx_bib_title\"><em class=\"ltx_emph\">A mirror theorem for Gromov-Witten theory without convexity</em></span>. <span class=\"ltx_text ltx_bib_date\">2020</span>. <span class=\"ltx_text ltx_bib_eprint\">arXiv: <a href=\"https://arxiv.org/abs/1910.14440\" title=\"\" class=\"ltx_ref arxiv ltx_bib_external\">1910.14440 <span class=\"ltx_text ltx_font_typewriter\">[math.AG]</span></a></span>.\n</span></li>"
  );
}

/// `\cite{a,b}` of an alphabetic style separates its keys with `\multicitedelim`, a semicolon
/// (alphabetic.cbx:64-69, biblatex.def:97); a note after the keys keeps `\postnotedelim`'s comma; numeric.cbx:6 makes
/// the delimiter a comma.
#[test]
fn alphabetic_cite_separates_keys_with_a_semicolon() {
  let body = "\\cite{gwdmstack,giventalmirror}. \\cite[Theorem 1]{gwdmstack}.\n\\printbibliography";
  let html = clean_html(&document("article", "style=alphabetic", body), BIB);
  assert_eq!(
    cites(&html),
    [
      "<cite class=\"ltx_cite ltx_citemacro_cite\">[<a href=\"#bib.bib1\" title=\"Gromov-Witten theory of Deligne-Mumford stacks\" class=\"ltx_ref\">AGV08</a>; <a href=\"#bib.bib2\" title=\"A mirror theorem for toric complete intersections\" class=\"ltx_ref\">Giv98</a>]</cite>",
      "<cite class=\"ltx_cite ltx_citemacro_cite\">[<a href=\"#bib.bib1\" title=\"Gromov-Witten theory of Deligne-Mumford stacks\" class=\"ltx_ref\">AGV08</a>, Theorem 1]</cite>",
    ],
    "{html}"
  );
  let html = clean_html(&document("article", "style=numeric", body), BIB);
  assert_eq!(
    cites(&html)[0],
    "<cite class=\"ltx_cite ltx_citemacro_cite\">[<a href=\"#bib.bib1\" title=\"Gromov-Witten theory of Deligne-Mumford stacks\" class=\"ltx_ref\">1</a>, <a href=\"#bib.bib2\" title=\"A mirror theorem for toric complete intersections\" class=\"ltx_ref\">2</a>]</cite>"
  );
}

/// The heading is biblatex.def:2184-2239's choice from the class: `\refname` for the article-like classes and
/// memoir's `article` option, `\bibname` for memoir's book mode, `book` and `report`, and an explicit `title=` wins.
/// memoir with babel, which defines both names, says "Bibliography" as the PDF does.
#[test]
fn heading_follows_the_class() {
  let heading = |class: &str, preamble: &str, print: &str, expected_warnings: &[&str]| {
    let tex = document(
      class,
      "style=alphabetic",
      &format!("\\cite{{gwdmstack}}\n{print}"),
    )
    .replace(
      "\\usepackage[backend",
      &format!("{preamble}\\usepackage[backend"),
    );
    let (stderr, html) = convert(&tex, &[("refs.bib", BIB)]);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(warnings(&stderr), expected_warnings, "{class}: {stderr}");
    html
      .split("ltx_title_bibliography\">")
      .nth(1)
      .and_then(|rest| rest.split("</h").next())
      .unwrap_or_default()
      .to_string()
  };
  let print = "\\printbibliography";
  assert_eq!(heading("article", "", print, &[]), "References");
  assert_eq!(heading("amsart", "", print, &[]), "References");
  // memoir's `\@iffirstamp` redefinition, which every memoir document reports.
  let iffirstamp = ["Warning:misdefined:\\@iffirstamp"];
  assert_eq!(heading("memoir", "", print, &iffirstamp), "Bibliography");
  assert_eq!(
    heading(
      "memoir",
      "\\usepackage[english]{babel}\n",
      print,
      &iffirstamp
    ),
    "Bibliography"
  );
  assert_eq!(heading("report", "", print, &[]), "Bibliography");
  // The `subbib*` headings say `\refname` in every class (biblatex.def:2257-2264).
  for heading_key in ["subbibliography", "subbibintoc", "subbibnumbered"] {
    let print = format!("\\printbibliography[heading={heading_key}]");
    assert_eq!(
      heading("report", "", &print, &[]),
      "References",
      "{heading_key}"
    );
    assert_eq!(
      heading("book", "", &print, &[]),
      "References",
      "{heading_key}"
    );
  }
  assert_eq!(
    heading("report", "", "\\printbibliography[heading=bibintoc]", &[]),
    "Bibliography"
  );
  assert_eq!(heading("book", "", print, &[]), "Bibliography");
  assert_eq!(
    heading("article", "", "\\printbibliography[title={参考文献}]", &[]),
    "参考文献"
  );
  assert_eq!(
    heading(
      "memoir",
      "",
      "\\printbibliography[title={Notes}]",
      &iffirstamp
    ),
    "Notes"
  );
}

/// `backref=true` prints "(cit. on p. N)" in the PDF and nothing in the page: the entries are the same as without it,
/// with neither LaTeXML's "Cited by:" nor "External Links:" nor a MathReview link.
#[test]
fn entries_have_no_cited_by_external_links_or_reviews() {
  let body = "\\nocite{*}\\printbibliography";
  let plain = clean_html(&document("article", "style=alphabetic", body), BIB);
  let backref = clean_html(
    &document(
      "article",
      "style=alphabetic,hyperref,backref=true,backrefstyle=none",
      "\\cite{gwdmstack,giventalmirror}\\cite{mirrornonconvex}\n\\printbibliography",
    ),
    BIB,
  );
  for label in ["AGV08", "Giv98", "Wan20"] {
    assert_eq!(bibitem(&backref, label), bibitem(&plain, label), "{label}");
  }
  for word in [
    "Cited by",
    "External Links",
    "MathReview",
    "cit.",
    "Document",
  ] {
    assert!(!backref.contains(word), "{word} in {backref}");
  }
}

/// `maxnames`/`minnames` (biblatex.sty:15284-15295, :4690-4720 `\blx@namesetup`): a list longer than `maxnames` shows
/// `minnames` names and "et al."; `isbn=false` hides ISSN as well as ISBN (standard.bbx:50); `doi=false`,
/// `url=false` (an `@online` entry still shows its url, standard.bbx:23); `giveninits` abbreviates.
#[test]
fn options_shape_the_entries() {
  let body = "\\nocite{*}\\printbibliography";
  let texts = |options: &str| {
    bibitem_texts(&clean_html(
      &document("article", options, body),
      OPTIONS_BIB,
    ))
  };
  assert_eq!(texts("style=alphabetic,maxnames=2"), [
    "[Har77] Robin Hartshorne. Algebraic Geometry. 2nd ed. Graduate Texts in Mathematics 52. New York: Springer, 1977. 496 pp. isbn: 0-387-90244-9.",
    "[On21] Ona On. Online thing. 2021. url: https://example.org/o (visited on 05/17/2021).",
    "[Smi+01] John Smith et al. “Four authors”. In: J. Alg. 1 (2001), pp. 5–9. issn: 1111-2222. doi: 10.1/x.",
  ]);
  assert_eq!(
    texts("style=alphabetic,maxnames=3,minnames=2")[2],
    "[Smi+01] John Smith, Jane Doe, et al. “Four authors”. In: J. Alg. 1 (2001), pp. 5–9. issn: 1111-2222. doi: 10.1/x."
  );
  assert_eq!(
    texts("style=alphabetic,maxnames=5,isbn=false,doi=false,url=false,giveninits"),
    [
      "[Har77] R. Hartshorne. Algebraic Geometry. 2nd ed. Graduate Texts in Mathematics 52. New York: Springer, 1977. 496 pp.",
      "[On21] O. On. Online thing. 2021. url: https://example.org/o (visited on 05/17/2021).",
      "[Smi+01] J. Smith, J. Doe, R. Roe, and E. Poe. “Four authors”. In: J. Alg. 1 (2001), pp. 5–9.",
    ]
  );
}

/// A style whose drivers are not standard.bbx's keeps MakeBibliography's rows.
#[test]
fn authoryear_style_keeps_the_formatter_rows() {
  let html = clean_html(
    &document(
      "article",
      "style=authoryear",
      "\\cite{gwdmstack}\n\\printbibliography",
    ),
    BIB,
  );
  assert_eq!(
    html
      [html.find("<li ").expect("a bibitem")..html.find("</li>").expect("its end") + "</li>".len()]
      .to_string(),
    "<li id=\"bib.bib1\" class=\"ltx_bibitem ltx_bib_article\"><span class=\"ltx_tag ltx_bib_author-year ltx_role_refnum ltx_tag_bibitem\">Abramovich <span class=\"ltx_text ltx_bib_etal\">et al.</span> (2008)</span>\n<span class=\"ltx_bibblock\"><span class=\"ltx_text ltx_bib_author\">Dan Abramovich, Tom Graber, and Angelo Vistoli</span>\n</span>\n<span class=\"ltx_bibblock\"><span class=\"ltx_text ltx_bib_title\">Gromov-Witten theory of Deligne-Mumford stacks</span>.\n</span>\n<span class=\"ltx_bibblock\"><span class=\"ltx_text ltx_bib_journal\">Amer. J. Math.</span> <span class=\"ltx_text ltx_bib_volume\">130</span> (<span class=\"ltx_text ltx_bib_number\">5</span>), <span class=\"ltx_text ltx_bib_pages\">pp.\u{a0}1337–1398</span>.\n</span>\n<span class=\"ltx_bibblock\">External Links: <span class=\"ltx_text ltx_bib_links\"><span class=\"ltx_text issn ltx_bib_external\">ISSN 0002-9327,1080-6377</span>,\n<a href=\"https://dx.doi.org/10.1353/ajm.0.0017\" title=\"\" class=\"ltx_ref doi ltx_bib_external\">Document</a>,\n<a href=\"https://www.ams.org/mathscinet-getitem?mr=2450211\" title=\"\" class=\"ltx_ref mr ltx_bib_external\">MathReview (Johannes Walcher)</a></span>\n</span>\n<span class=\"ltx_bibblock ltx_bib_cited\">Cited by: <a href=\"#p1\" title=\"\" class=\"ltx_ref\">p1</a>.\n</span></li>"
  );
}

/// Details of the standard drivers that pdflatex + biber print and the formatter rows did not: a `techreport` without
/// a `type` says "Tech. rep." (english.lbx), a proceedings volume has no edition (standard.bbx:569-615), an article
/// without a date keeps biblatex's empty "()" (`\printtext` tests its code for blankness), an eprint without a type
/// is "eprint: <id> (<class>)" and one of another type "<type>: <id>", a manual prints its editor and no translator
/// (standard.bbx:387-433).
#[test]
fn drivers_print_what_standard_bbx_prints() {
  let html = clean_html(
    &document(
      "article",
      "style=alphabetic",
      "\\nocite{*}\\printbibliography",
    ),
    DETAILS_BIB,
  );
  assert_eq!(
    bibitem_texts(&html),
    [
      "[Dat] Dan Dated. “No year”. In: J. Nodate 1 ().",
      "[Ed03] Edgar Ed, ed. Manual with editor. Org. 2003.",
      "[Ed95] Edgar Ed, ed. Edition proceedings. Pub, 1995.",
      "[Mis07a] Mike Mis. Hdl eprint. 2007. HDL: 2013/ABC.",
      "[Mis07b] Mike Mis. Other eprint. 2007. foo: xyz-42.",
      "[Mis07c] Mike Mis. Untyped eprint. 2007. eprint: 2999.12345 (math.XX).",
      "[Rep02] Rita Rep. Typeless report. Tech. rep. RR-6. Univ, 2002.",
    ],
    "{html}"
  );
}

/// A style built on `alphabetic` keeps the formatter rows: a package's own `.bbx` that requires it (LNI.bbx:19,
/// ext-alphabetic.bbx) redefines drivers and macros the layout does not know, and only the style the document asks
/// for decides.
#[test]
fn a_style_built_on_alphabetic_keeps_the_formatter_rows() {
  let bbx = "\\ProvidesFile{ownstyle.bbx}\n\\RequireBibliographyStyle{alphabetic}\n\\endinput\n";
  let tex = document(
    "article",
    "style=ownstyle",
    "\\cite{gwdmstack}\n\\printbibliography",
  );
  let (stderr, html) = convert(&tex, &[("refs.bib", BIB), ("ownstyle.bbx", bbx)]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let item = bibitem(&html, "AGV08");
  assert!(
    item.contains("External Links: <span class=\"ltx_text ltx_bib_links\">"),
    "{item}"
  );
  assert!(
    item.contains("<span class=\"ltx_bibblock ltx_bib_cited\">Cited by: "),
    "{item}"
  );
  assert!(!item.contains("In: "), "{item}");
}

/// The printed text of every entry of [`FIX_BIB`], converted once.
fn fix_texts() -> Vec<String> { bibitem_texts(&fix_html()) }

fn fix_html() -> String {
  clean_html(
    &document(
      "article",
      "style=alphabetic",
      "\\nocite{*}\\printbibliography",
    ),
    FIX_BIB,
  )
}

/// The text of the entry labelled `[label]` among `texts`.
fn text_of<'a>(texts: &'a [String], label: &str) -> &'a str {
  let prefix = format!("[{label}] ");
  texts
    .iter()
    .find(|text| text.starts_with(&prefix))
    .unwrap_or_else(|| panic!("no entry [{label}]"))
}

/// A journal's `series` ("3rd ser.", biblatex.def:574-577, english.lbx `jourser`; a key like `newseries` its string),
/// its `issue` (inside the parentheses, in front of the date) and its issue title (emphasised, after a colon),
/// standard.bbx:784-804.
#[test]
fn journal_series_issue_and_issue_title() {
  let html = clean_html(
    &document(
      "article",
      "style=alphabetic",
      "\\nocite{*}\\printbibliography",
    ),
    FIX_BIB,
  );
  assert_eq!(
    bibitem(&html, "Aa15"),
    "<li id=\"bib.bib1\" class=\"ltx_bibitem ltx_bib_article\"><span class=\"ltx_tag ltx_bib_abbrv ltx_role_refnum ltx_tag_bibitem\">[Aa15]</span>\n<span class=\"ltx_bibblock\"><span class=\"ltx_text ltx_bib_author\">Ann Aa</span>. <span class=\"ltx_text ltx_bib_title\">“Issue article”</span>. In: <span class=\"ltx_text ltx_bib_journal\"><em class=\"ltx_emph\">J. Iss</em></span> <span class=\"ltx_text ltx_bib_volume\">9</span> (<span class=\"ltx_text ltx_bib_issue\">Spring</span> <span class=\"ltx_text ltx_bib_date\">2015</span>): <span class=\"ltx_text ltx_bib_issuetitle\"><em class=\"ltx_emph\">Special Issue on Things</em></span>, <span class=\"ltx_text ltx_bib_pages\">pp.\u{a0}1–2</span>.\n</span></li>"
  );
  assert_eq!(
    bibitem(&html, "Bb16"),
    "<li id=\"bib.bib2\" class=\"ltx_bibitem ltx_bib_article\"><span class=\"ltx_tag ltx_bib_abbrv ltx_role_refnum ltx_tag_bibitem\">[Bb16]</span>\n<span class=\"ltx_bibblock\"><span class=\"ltx_text ltx_bib_author\">Bob Bb</span>. <span class=\"ltx_text ltx_bib_title\">“Series article”</span>. In: <span class=\"ltx_text ltx_bib_journal\"><em class=\"ltx_emph\">J. Ser.</em></span> <span class=\"ltx_text ltx_bib_series\">3rd\u{a0}ser.</span> <span class=\"ltx_text ltx_bib_volume\">9</span> (<span class=\"ltx_text ltx_bib_date\">2016</span>).\n</span></li>"
  );
  assert_eq!(
    bibitem(&html, "Cc16"),
    "<li id=\"bib.bib3\" class=\"ltx_bibitem ltx_bib_article\"><span class=\"ltx_tag ltx_bib_abbrv ltx_role_refnum ltx_tag_bibitem\">[Cc16]</span>\n<span class=\"ltx_bibblock\"><span class=\"ltx_text ltx_bib_author\">Cy Cc</span>. <span class=\"ltx_text ltx_bib_title\">“Text series article”</span>. In: <span class=\"ltx_text ltx_bib_journal\"><em class=\"ltx_emph\">J. Tser</em></span>. <span class=\"ltx_text ltx_bib_series\">New ser.</span> <span class=\"ltx_text ltx_bib_volume\">9</span> (<span class=\"ltx_text ltx_bib_date\">2016</span>).\n</span></li>"
  );
}

/// The translators never lead an entry (`usetranslator=false`, biblatex.sty:16427; biblatex.def:2849-2859): the entry
/// starts with its title and says "Trans. by …" later; editors who are the translators too are "ed. and trans."
/// (biblatex.def:2911-2915), or "Ed. and trans. by" behind an author (:3062-3065).
#[test]
fn translators_do_not_lead() {
  let texts = fix_texts();
  for (label, text) in [
    (
      "00a",
      "[00a] “Translated article”. Trans. by Tom Trans. In: J. T (2000).",
    ),
    (
      "00b",
      "[00b] “Translated incoll”. In: Big Book. Trans. by Tom Trans. Pub, 2000.",
    ),
    (
      "00c",
      "[00c] Translated Only. Trans. by Tom Trans. Pub, 2000.",
    ),
    (
      "Sam03",
      "[Sam03] Sam Same, ed. and trans. Same Person. Pub, 2003.",
    ),
    (
      "Au03a",
      "[Au03a] Ab Au. “Chapter same”. In: The Book. Ed. and trans. by Sam Same. Pub, 2003.",
    ),
    (
      "Au03b",
      "[Au03b] Al Au. Author and Same. Ed. and trans. by Sam Same. Pub, 2003.",
    ),
    // The manual driver's `author/editor` is the plain `editor` macro, whose string never takes the translators along
    // (standard.bbx:387-433), and prints no translator.
    ("Twi03", "[Twi03] Tia Twin, ed. Manual twin. Org. 2003."),
  ] {
    assert_eq!(text_of(&texts, label), text);
  }
}

/// A `language` list of the one item `english`, `american`, `langenglish` or `langamerican` is not printed
/// (`\blx@clearlang`, biblatex.sty:7058-7078, english.lbx:4); every lookup is case-sensitive, so `English`, `british`
/// and `USenglish` print as written, two english items are both printed, and the others are english.lbx's `lang<name>`
/// strings (`langgerman` too); a `pubstate` is english.lbx's string (biblatex.def:578, english.lbx:433-437) or as
/// written.
#[test]
fn languages_and_publication_states() {
  let texts = fix_texts();
  for (label, text) in [
    ("Ll01a", "[Ll01a] Lee Ll. Lang english. Pub, 2001."),
    ("Ll01b", "[Ll01b] Lia Ll. Lang en. en. Pub, 2001."),
    ("Ll01c", "[Ll01c] Lio Ll. Lang french. French. Pub, 2001."),
    ("Ll01d", "[Ll01d] Lou Ll. Lang enUS. en-US. Pub, 2001."),
    (
      "Ll01e",
      "[Ll01e] Luc Ll. Lang langgerman. German. Pub, 2001.",
    ),
    (
      "Ll01f",
      "[Ll01f] Lyn Ll. Lang list. English and French. Pub, 2001.",
    ),
    ("Qa05", "[Qa05] Quin Qa. Lang Capital. English. Pub, 2005."),
    ("Qb05", "[Qb05] Quin Qb. Lang british. british. Pub, 2005."),
    (
      "Qc05",
      "[Qc05] Quin Qc. Lang USenglish. USenglish. Pub, 2005.",
    ),
    (
      "Qd05",
      "[Qd05] Quin Qd. Lang two english. English and American. Pub, 2005.",
    ),
    ("Qe05", "[Qe05] Quin Qe. Lang FRENCH. FRENCH. Pub, 2005."),
    ("Qf05", "[Qf05] Quin Qf. Lang american. Pub, 2005."),
    (
      "Pp02a",
      "[Pp02a] Pam Pp. Pub submitted. Pub, 2002. Submitted.",
    ),
    ("Pp02b", "[Pp02b] Pat Pp. Pub inpress. Pub, 2002. In press."),
    (
      "Pp02c",
      "[Pp02c] Pia Pp. Pub forthcoming. Pub, 2002. Forthcoming.",
    ),
    ("Pp02d", "[Pp02d] Pol Pp. Pub other. Pub, 2002. Draft text."),
  ] {
    assert_eq!(text_of(&texts, label), text);
  }
}

/// A title is set by the entry's own type (biblatex.def:579-585), not by the driver it prints with: an `@review` and a
/// `@bookinbook` are emphasised, a `@suppbook` plain, an `@article` quoted.
#[test]
fn titles_are_set_by_the_entry_type() {
  let html = fix_html();
  let li = |id: &str, expected: &str| {
    latexml::util::test::assert_element(&html, "li", &[format!("id=\"{id}\"").as_str()], expected);
  };
  li(
    "bib.bib20",
    "<li id=\"bib.bib20\" class=\"ltx_bibitem ltx_bib_article\"><span class=\"ltx_tag ltx_bib_abbrv ltx_role_refnum ltx_tag_bibitem\">[Rr04]</span>\n<span class=\"ltx_bibblock\"><span class=\"ltx_text ltx_bib_author\">Rob Rr</span>. <span class=\"ltx_text ltx_bib_title\"><em class=\"ltx_emph\">A review</em></span>. In: <span class=\"ltx_text ltx_bib_journal\"><em class=\"ltx_emph\">J. Rev</em></span> (<span class=\"ltx_text ltx_bib_date\">2004</span>).\n</span></li>",
  );
  // A `@bookinbook` and a `@suppbook` have no host in the reader: their `booktitle` is plain data, emphasised all the same.
  li(
    "bib.bib21",
    "<li id=\"bib.bib21\" class=\"ltx_bibitem ltx_bib_inbook\"><span class=\"ltx_tag ltx_bib_abbrv ltx_role_refnum ltx_tag_bibitem\">[Bk04]</span>\n<span class=\"ltx_bibblock\"><span class=\"ltx_text ltx_bib_author\">Bea Bk</span>. <span class=\"ltx_text ltx_bib_title\"><em class=\"ltx_emph\">Book in book</em></span>. In: <span class=\"ltx_text ltx_bib_booktitle\"><em class=\"ltx_emph\">Outer</em></span>. <span class=\"ltx_text ltx_bib_publisher\">Pub</span>, <span class=\"ltx_text ltx_bib_date\">2004</span>.\n</span></li>",
  );
  li(
    "bib.bib22",
    "<li id=\"bib.bib22\" class=\"ltx_bibitem ltx_bib_inbook\"><span class=\"ltx_tag ltx_bib_abbrv ltx_role_refnum ltx_tag_bibitem\">[Sb04]</span>\n<span class=\"ltx_bibblock\"><span class=\"ltx_text ltx_bib_author\">Sue Sb</span>. <span class=\"ltx_text ltx_bib_title\">Supp book</span>. In: <span class=\"ltx_text ltx_bib_booktitle\"><em class=\"ltx_emph\">Outer2</em></span>. <span class=\"ltx_text ltx_bib_publisher\">Pub</span>, <span class=\"ltx_text ltx_bib_date\">2004</span>.\n</span></li>",
  );
  assert!(
    bibitem(&html, "Bb16")
      .contains("<span class=\"ltx_text ltx_bib_title\">“Series article”</span>")
  );
}

/// LaTeXML.css sets `.ltx_bib_title` italic and `.ltx_bib_volume` bold for the `.bst` rows; the standard layout sets
/// its own fonts (a quoted title upright, the emphasis an `<em>`, a regular volume and label), so its list carries the
/// class the scoped rules of LaTeXML.css key on, and the `.bst` rows' list does not.
#[test]
fn the_standard_list_carries_the_class_that_scopes_its_fonts() {
  let html = fix_html();
  assert!(
    html.contains("<ul id=\"bib.L1\" class=\"ltx_biblist ltx_bib_standard\">"),
    "{html}"
  );
  let ay = clean_html(
    &document(
      "article",
      "style=authoryear",
      "\\cite{gwdmstack}\n\\printbibliography",
    ),
    BIB,
  );
  assert!(
    ay.contains("<ul id=\"bib.L1\" class=\"ltx_biblist\">"),
    "{ay}"
  );
  let css = include_str!("../../../latexml_post/resources/CSS/LaTeXML.css");
  for rule in [
    ".ltx_bib_standard .ltx_bib_title { font-style:normal; }",
    ".ltx_bib_standard .ltx_bib_volume { font-weight:normal; }",
    ".ltx_bib_standard .ltx_tag { font-weight:normal; }",
    ".ltx_bib_standard em em { font-style:normal; }",
  ] {
    assert!(css.contains(rule), "{rule}");
  }
}

/// `alphabetic-verb.cbx:31-40` brackets every key, `[AGV08]; [Giv98]`; the core citation gives the semicolon inside one
/// pair of brackets (DIVERGENCES #601, "Citations").
#[test]
fn alphabetic_verb_cite_has_the_semicolon_in_one_pair_of_brackets() {
  let html = clean_html(
    &document(
      "article",
      "style=alphabetic-verb",
      "\\cite{gwdmstack,giventalmirror}.\n\\printbibliography",
    ),
    BIB,
  );
  assert_eq!(
    cites(&html),
    [
      "<cite class=\"ltx_cite ltx_citemacro_cite\">[<a href=\"#bib.bib1\" title=\"Gromov-Witten theory of Deligne-Mumford stacks\" class=\"ltx_ref\">AGV08</a>; <a href=\"#bib.bib2\" title=\"A mirror theorem for toric complete intersections\" class=\"ltx_ref\">Giv98</a>]</cite>"
    ],
    "{html}"
  );
}

/// A `journaltitle` on an `@incollection` (what biber's `.bbl` makes of `JOURNAL`) is ignored by the incollection
/// driver, which prints the book part: `In: vol. 1. 4. Dedicated. 2001, pp. 551–568.` — the reader files the journal
/// and the book part (`volume`, `number`) as two hosts, and the book's is the one this driver reads.
#[test]
fn a_journal_title_on_an_incollection_is_ignored() {
  let bib = "@incollection{vf, AUTHOR = {Givental, Alexander B.}, TITLE = {Gromov-{W}itten invariants}, NOTE = {Dedicated}, JOURNALTITLE = {Mosc. Math. J.}, VOLUME = {1}, NUMBER = {4}, YEAR = {2001}, PAGES = {551--568}}\n";
  for field in ["JOURNALTITLE", "JOURNAL"] {
    let bib = bib.replace("JOURNALTITLE", field);
    let html = clean_html(
      &document(
        "article",
        "style=alphabetic",
        "\\nocite{*}\\printbibliography",
      ),
      &bib,
    );
    latexml::util::test::assert_element(
      &html,
      "li",
      &["id=\"bib.bib1\""],
      "<li id=\"bib.bib1\" class=\"ltx_bibitem ltx_bib_incollection\"><span class=\"ltx_tag ltx_bib_abbrv ltx_role_refnum ltx_tag_bibitem\">[Giv01]</span>\n<span class=\"ltx_bibblock\"><span class=\"ltx_text ltx_bib_author\">Alexander B. Givental</span>. <span class=\"ltx_text ltx_bib_title\">“Gromov-Witten invariants”</span>. In: <span class=\"ltx_text ltx_bib_volume\">vol.\u{a0}1</span>. <span class=\"ltx_text ltx_bib_number\">4</span>. <span class=\"ltx_text ltx_bib_note\">Dedicated</span>. <span class=\"ltx_text ltx_bib_date\">2001</span>, <span class=\"ltx_text ltx_bib_pages\">pp.\u{a0}551–568</span>.\n</span></li>",
    );
  }
}

/// An `\emph` inside an emphasised title or journal is upright (the emphasis toggles): the markup nests an `<em>` in the
/// `<em>`, and LaTeXML.css makes the inner one upright for the standard list; in a quoted title it stays italic.
#[test]
fn emphasis_nested_in_an_emphasised_title_nests_its_markup() {
  let bib = "@book{b1, author={A, A}, title={The \\emph{Foo} book}, publisher={Pub}, year={2001}}\n@article{a1, author={B, B}, title={On \\emph{Bar}}, journal={J. \\emph{Baz}}, year={2001}}\n";
  let html = clean_html(
    &document(
      "article",
      "style=alphabetic",
      "\\nocite{*}\\printbibliography",
    ),
    bib,
  );
  let item = bibitem(&html, "A01");
  assert!(
    item.contains("<span class=\"ltx_text ltx_bib_title\"><em class=\"ltx_emph\">The <em class=\"ltx_emph ltx_font_italic\">Foo</em> book</em></span>"),
    "{item}"
  );
  let item = bibitem(&html, "B01");
  assert!(
    item.contains("“On <em class=\"ltx_emph ltx_font_italic\">Bar</em>”"),
    "{item}"
  );
  assert!(
    item.contains("<em class=\"ltx_emph\">J. <em class=\"ltx_emph ltx_font_italic\">Baz</em></em>"),
    "{item}"
  );
}
