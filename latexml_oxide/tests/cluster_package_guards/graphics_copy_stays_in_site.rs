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

#[test]
fn a_graphic_outside_the_source_directory_is_copied_into_the_site() {
  let work = tempfile::tempdir().expect("create tempdir");
  let (src, img) = (work.path().join("src"), work.path().join("img"));
  std::fs::create_dir_all(&src).expect("mkdir src");
  std::fs::create_dir_all(&img).expect("mkdir img");
  std::fs::write(img.join("p.png"), PNG).expect("write p.png");
  // Not byte-identical to p.png, which a content-hash dedup would share.
  std::fs::write(src.join("in.png"), [PNG, b"\0"].concat()).expect("write in.png");
  std::fs::write(
    src.join("doc.tex"),
    "\\documentclass{article}\n\\usepackage{graphicx}\n\\begin{document}\n\
     \\includegraphics{../img/p.png}\\includegraphics{../img/p.png}\\includegraphics{in.png}\n\\end{document}\n",
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
    .current_dir(&src)
    .output()
    .expect("spawn latexml_oxide");
  let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
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
  assert!(
    src.join("out/x1.png").exists(),
    "the generated copy is in the site"
  );
  assert!(
    html.contains(r#"src="in.png""#),
    "a graphic in the source directory keeps its path:\n{html}"
  );
  assert!(src.join("out/in.png").exists(), "and is copied there");
}
