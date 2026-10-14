//! `\setbeamertemplate{element}` takes a body `{…}` or a predefined `[name]` with its own `[…]` options; the stub read two
//! mandatory arguments, so `\setbeamertemplate{theorems}[numbered]` printed `numbered]`.

#[test]
fn setbeamertemplate_options_print_nothing() {
  let tex = "\\documentclass{beamer}\n\\setbeamertemplate{theorems}[numbered]\n\
             \\setbeamertemplate{blocks}[rounded][shadow=true]\n\\setbeamertemplate{navigation symbols}{}\n\
             \\begin{document}\n\\begin{frame}A\\end{frame}\n\\end{document}\n";
  let (stderr, xml) = super::convert(tex, true);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  for leak in ["numbered", "rounded", "shadow"] {
    assert!(
      !xml.contains(leak),
      "{leak} leaked into the document:\n{xml}"
    );
  }
}
