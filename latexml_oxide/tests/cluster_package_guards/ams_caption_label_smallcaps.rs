//! amsart, amsproc and amsbook set the label of a figure or table caption, `Figure 1` or `Table 1`,
//! in small capitals: `\@makecaption` prints `#1` under `\@captionheadfont`, `\scshape`
//! (amsart.cls:1354 and :1389, amsproc.cls:1290 and :1325, amsbook.cls:1250 and :1285), and the title
//! after it in `\@captionfont\upshape`. Perl leaves the label upright and leaves the small capitals
//! to ltx-amsart.css. OXIDIZED_DESIGN_DIVERGENCES #605.

fn captions(class: &str, preamble: &str) -> String {
  let (stderr, xml) = super::convert(
    &format!(
      "\\documentclass{{{class}}}\n{preamble}\\begin{{document}}\n\
       \\begin{{figure}}\\centering X\\caption{{A figure caption}}\\end{{figure}}\n\
       \\begin{{table}}\\centering Y\\caption{{A table caption}}\\end{{table}}\n\
       \\end{{document}}\n"
    ),
    false,
  );
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  assert_eq!(
    super::perfect_kernel_batch46::warning_count(&stderr),
    0,
    "{stderr}"
  );
  xml
}

fn assert_smallcaps_labels(class: &str) {
  let xml = captions(class, "");
  for (kind, label, title) in [
    ("figure", "Figure 1", "A figure caption"),
    ("table", "Table 1", "A table caption"),
  ] {
    let caption = format!(
      "<caption class=\"ltx_centering\"><tag close=\". \"><text font=\"smallcaps\">{label}</text></tag>{title}</caption>"
    );
    assert_eq!(xml.matches(&caption).count(), 1, "{class} {kind}:\n{xml}");
  }
}

#[test]
fn amsart_caption_labels_are_small_capitals() { assert_smallcaps_labels("amsart"); }

#[test]
fn amsproc_caption_labels_are_small_capitals() { assert_smallcaps_labels("amsproc"); }

#[test]
fn amsbook_caption_labels_are_small_capitals() { assert_smallcaps_labels("amsbook"); }

/// The size a caption package binding sets (subcaption's `\small`, the binding's: real caption-ams-smf.sto keeps
/// `\@captionfont`) still applies to the label, and the small capitals come with it.
#[test]
fn amsart_caption_label_keeps_the_size_the_caption_binding_sets() {
  let xml = captions("amsart", "\\usepackage{subcaption}\n");
  assert_eq!(
    xml.matches(
      "<caption class=\"ltx_centering\"><tag close=\". \"><text font=\"smallcaps\" fontsize=\"90%\">Figure 1</text></tag><text fontsize=\"90%\">A figure caption</text></caption>"
    )
    .count(),
    1,
    "{xml}"
  );
}

/// Other classes are untouched: the label is plain text.
#[test]
fn article_caption_label_is_upright() {
  let xml = captions("article", "");
  assert_eq!(
    xml.matches("<caption class=\"ltx_centering\"><tag close=\": \">Figure 1</tag>A figure caption</caption>").count(),
    1,
    "{xml}"
  );
}

/// acmart loads amsart and sets the label fonts through caption (acmart.cls:878-906), not amsart's `\@makecaption`:
/// its label is not in small capitals.
#[test]
fn acmart_caption_label_is_not_small_capitals() {
  let xml = captions("acmart", "");
  assert_eq!(
    xml.matches("<caption class=\"ltx_centering\"><tag close=\". \">Figure 1</tag>A figure caption</caption>")
      .count(),
    1,
    "{xml}"
  );
  assert!(!xml.contains("smallcaps"), "{xml}");
}
