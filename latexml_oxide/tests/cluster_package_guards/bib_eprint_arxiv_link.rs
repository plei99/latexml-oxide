//! An arXiv eprint in a `.bib` is a LINK, as biblatex prints it (biblatex.def:526:
//! `arXiv: <id> [<class>]`, the id and class linked to `https://arxiv.org/abs/<id>`; the
//! `archiveprefix` / `primaryclass` aliases of `eprinttype` / `eprintclass`,
//! biblatex.def:1365-1367). Perl's reader (`BibTeX.pool.ltxml:724`) files `eprint` as
//! plain `ltx:bib-links` text and leaves the type and class as unformatted
//! `ltx:bib-data`, so the id was never a link. SURPASS-PERL
//! (OXIDIZED_DESIGN_DIVERGENCES #585). The eprint becomes an
//! `ltx:bib-identifier scheme="arxiv"`, the same element the DOI uses, so the
//! post stage renders it as an `ltx:ref` among the "External Links". Witness:
//! 2404.01499 (`archivePrefix={arXiv}`, `primaryClass={math.AG}`).
//! Both readers share it: the biblatex `.bib` path and classic `\bibliography`.
use std::process::Command;

const RAW: &str = "--preload=[rawstyles,rawclasses]latexml.sty";

const BIB: &str = r#"
@misc{aliased, author={Doe, Jane}, title={Aliased}, year={2025},
  eprint={2404.01499}, archivePrefix={arXiv}, primaryClass={math.AG}}
@misc{typed, author={Doe, Jane}, title={Typed}, year={2025},
  eprint={1201.4501}, eprinttype={arXiv}, eprintclass={math.QA},
  doi={10.1000/example}, url={https://example.org/b}}
@misc{noclass, author={Doe, Jane}, title={Noclass}, year={2025},
  eprint={2301.00001}, eprinttype={ARXIV}}
@misc{oldstyle, author={Doe, Jane}, title={Oldstyle}, year={2025},
  eprint={math/0211159}, eprinttype={arXiv}, eprintclass={math.GT}}
@misc{other, author={Doe, Jane}, title={Other}, year={2025},
  eprint={hep-th/9901001}, eprinttype={jstor}}
@misc{untyped, author={Doe, Jane}, title={Untyped}, year={2025},
  eprint={2999.12345}}
"#;

const BIBLATEX_TEX: &str = r"\documentclass{article}
\usepackage[backend=biber,style=alphabetic]{biblatex}
\addbibresource{x.bib}
\begin{document}
\cite{aliased,typed,noclass,oldstyle,other,untyped}\printbibliography
\end{document}
";

const CLASSIC_TEX: &str = r"\documentclass{article}
\begin{document}
\cite{aliased,typed,noclass,oldstyle,other,untyped}
\bibliographystyle{alpha}\bibliography{x}
\end{document}
";

/// Convert `x.tex` (with `x.bib` beside it) to HTML. Returns (stderr, html).
fn convert_html(tex: &str) -> (String, String) {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("x.tex"), tex).expect("write x.tex");
  std::fs::write(workdir.path().join("x.bib"), BIB).expect("write x.bib");
  let output = Command::new(bin)
    .args([
      "x.tex",
      "--dest",
      "x.html",
      "--nocomments",
      "--timeout=110",
      RAW,
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  let html = std::fs::read_to_string(workdir.path().join("x.html")).unwrap_or_default();
  (stderr, html)
}

/// The bibitem `<li>` whose body mentions `title`.
fn item<'a>(html: &'a str, title: &str) -> &'a str {
  let at = html
    .find(&format!(r#"ltx_bib_title">{title}</span>"#))
    .unwrap_or_else(|| panic!("no bibitem titled {title}:\n{html}"));
  let end = html[at..].find("</li>").map_or(html.len(), |e| at + e);
  &html[at..end]
}

fn check(html: &str) {
  // Aliased field names, with a class: the label is the link, the class follows.
  let aliased = item(html, "Aliased");
  assert!(
    aliased.contains(r#"<a href="https://arxiv.org/abs/2404.01499" title="" class="ltx_ref arxiv ltx_bib_external">arXiv:2404.01499 [math.AG]</a>"#),
    "{aliased}"
  );
  // Canonical names; DOI and URL beside it are unchanged.
  let typed = item(html, "Typed");
  assert!(
    typed.contains(r#"<a href="https://arxiv.org/abs/1201.4501" title="" class="ltx_ref arxiv ltx_bib_external">arXiv:1201.4501 [math.QA]</a>"#),
    "{typed}"
  );
  assert!(
    typed.contains(r#"<a href="https://dx.doi.org/10.1000/example" title="" class="ltx_ref doi ltx_bib_external">Document</a>"#),
    "{typed}"
  );
  assert!(
    typed.contains(r#"<a href="https://example.org/b" title="" class="ltx_ref ltx_bib_external">https://example.org/b</a>"#)
      || typed.contains(r#"class="ltx_ref ltx_bib_external">Link</a>"#),
    "{typed}"
  );
  // No class: just the id; the type compares case-insensitively.
  let noclass = item(html, "Noclass");
  assert!(
    noclass.contains(r#"href="https://arxiv.org/abs/2301.00001" title="" class="ltx_ref arxiv ltx_bib_external">arXiv:2301.00001</a>"#),
    "{noclass}"
  );
  // Old-style identifier keeps its slash in text and address.
  let old = item(html, "Oldstyle");
  assert!(
    old.contains(r#"href="https://arxiv.org/abs/math/0211159" title="" class="ltx_ref arxiv ltx_bib_external">arXiv:math/0211159 [math.GT]</a>"#),
    "{old}"
  );
  // A non-arXiv eprint, and one with no type, stay plain text.
  for title in ["Other", "Untyped"] {
    let it = item(html, title);
    assert!(!it.contains("arxiv.org"), "{it}");
    assert!(!it.contains("arXiv:"), "{it}");
  }
  assert!(
    item(html, "Other")
      .contains(r#"<span class="ltx_text ltx_bib_external">hep-th/9901001</span>"#),
    "{}",
    item(html, "Other")
  );
  assert!(
    item(html, "Untyped").contains(r#"<span class="ltx_text ltx_bib_external">2999.12345</span>"#),
    "{}",
    item(html, "Untyped")
  );
}

#[test]
fn biblatex_bib_arxiv_eprint_is_linked() {
  let (stderr, html) = convert_html(BIBLATEX_TEX);
  assert!(
    !stderr.contains("Error:") && !stderr.contains("Fatal:") && !stderr.contains("Warning:"),
    "{stderr}"
  );
  check(&html);
}

#[test]
fn classic_bibtex_arxiv_eprint_is_linked() {
  let (stderr, html) = convert_html(CLASSIC_TEX);
  assert!(
    !stderr.contains("Error:") && !stderr.contains("Fatal:") && !stderr.contains("Warning:"),
    "{stderr}"
  );
  check(&html);
}
