//! A copied graphic stays in the site directory (Perl `Post/Graphics.pm` L288-300, `pathname_is_contained`): it
//! keeps its path relative to the source directory only when that path stays within the site; a graphic outside the
//! source directory (`../img/p.png`) gets a generated name instead of being written out of the destination.
use std::process::Command;

/// A 1×1 PNG.
const PNG: &[u8] = &[
  0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52,
  0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4,
  0x89, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0xf8, 0xcf, 0xc0, 0xf0,
  0x1f, 0x00, 0x05, 0x00, 0x01, 0xff, 0x89, 0x99, 0x3d, 0x1d, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45,
  0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
];

/// Converts `src/doc.tex` (`body` in a graphicx article) beside `img/{p,q}.png` and `src/in.png`, all distinct, into
/// `src/out/doc.html` with `args`. Returns (stderr, the work directory).
fn convert(body: &str, args: &[&str]) -> (String, tempfile::TempDir) {
  let work = tempfile::tempdir().expect("create tempdir");
  let (src, img) = (work.path().join("src"), work.path().join("img"));
  std::fs::create_dir_all(&src).expect("mkdir src");
  std::fs::create_dir_all(&img).expect("mkdir img");
  // Distinct bytes, which a content-hash dedup would not share.
  std::fs::write(img.join("p.png"), PNG).expect("write p.png");
  std::fs::write(img.join("q.png"), [PNG, b"q"].concat()).expect("write q.png");
  std::fs::write(src.join("in.png"), [PNG, b"i"].concat()).expect("write in.png");
  std::fs::write(
    src.join("doc.tex"),
    format!("\\documentclass{{article}}\n\\usepackage{{graphicx}}\n\\begin{{document}}\n{body}\n\\end{{document}}\n"),
  )
  .expect("write doc.tex");
  let output = Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args([
      "doc.tex",
      "--dest",
      "out/doc.html",
      "--nocomments",
      "--timeout=110",
    ])
    .args(args)
    .current_dir(&src)
    .output()
    .expect("spawn latexml_oxide");
  let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  assert_eq!(
    super::perfect_kernel_batch46::warning_count(&stderr),
    0,
    "{stderr}"
  );
  (stderr, work)
}

fn bytes(path: std::path::PathBuf) -> Vec<u8> {
  std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

#[test]
fn a_graphic_outside_the_source_directory_is_copied_into_the_site() {
  let (_, work) = convert(
    "\\includegraphics{../img/p.png}\\includegraphics{../img/p.png}\\includegraphics{in.png}",
    &[],
  );
  let (src, img) = (work.path().join("src"), work.path().join("img"));
  let html = std::fs::read_to_string(src.join("out/doc.html")).expect("read doc.html");
  assert!(
    !src.join("img").exists(),
    "nothing is written out of the destination (out/../img/p.png):\n{html}"
  );
  assert_eq!(
    html.matches(r#"src="x1.png""#).count(),
    2,
    "both uses of the outside graphic share one generated copy:\n{html}"
  );
  assert_eq!(
    bytes(src.join("out/x1.png")),
    bytes(img.join("p.png")),
    "the copy is p.png"
  );
  assert!(
    html.contains(r#"src="in.png""#),
    "a graphic in the source directory keeps its path:\n{html}"
  );
  assert_eq!(
    bytes(src.join("out/in.png")),
    bytes(src.join("in.png")),
    "and is copied there"
  );
}

/// The pages of a split document share the names given out in their destination, as Perl's are in its
/// `LaTeXML.cache`: numbered per page, both pages' outside graphics were `x1.png`, one written over the other.
#[test]
fn split_pages_give_their_graphics_distinct_names() {
  let (_, work) = convert(
    "\\section{A}\\includegraphics{../img/p.png}\n\\section{B}\\includegraphics{../img/q.png}",
    &["--split", "--splitat=section"],
  );
  let (out, img) = (work.path().join("src/out"), work.path().join("img"));
  let src = regex::Regex::new(r#"<img src="([^"]+)""#).unwrap();
  let mut srcs = Vec::new();
  for page in ["S1.html", "S2.html"] {
    let html =
      std::fs::read_to_string(out.join(page)).unwrap_or_else(|e| panic!("read {page}: {e}"));
    let name = src
      .captures(&html)
      .unwrap_or_else(|| panic!("{page} has its graphic:\n{html}"))[1]
      .to_string();
    srcs.push(name);
  }
  assert_ne!(srcs[0], srcs[1], "each page's graphic has its own file");
  assert_eq!(
    bytes(out.join(&srcs[0])),
    bytes(img.join("p.png")),
    "section A shows p.png"
  );
  assert_eq!(
    bytes(out.join(&srcs[1])),
    bytes(img.join("q.png")),
    "section B shows q.png"
  );
}
