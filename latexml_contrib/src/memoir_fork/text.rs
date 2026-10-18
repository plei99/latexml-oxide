//! Text-level commands and memoir's emulated packages.
use latexml_package::prelude::*;

/// Packages memoir emulates (`\EmulatedPackage`, memoir.cls:130-152, :12205-12256) whose explicit
/// `\usepackage` is a no-op under pdfLaTeX (`\ver@<pkg>.sty` is set) and must be one here too, because oxide's
/// loader does not read `\ver@` (`docs/SYNC_STATUS.md` K13) and so loads its own binding over memoir's
/// definitions. Honouring `\ver@` globally broke abntex2 (2605.08402), hence this list (`fork/MEMOIR_ROADMAP.md`
/// D2): one entry per package whose memoir original is the better one.
///
///  * `parskip`: memoir's traditional `\parskip` (memoir.cls:705, :2422-2434) and `\parindent` stay, as in the PDF
///    ("Package parskip [2008/07/23] emulated by memoir."). The real package zeroes `\parindent`, which flips every
///    paragraph to `ltx_noindent` (`parskip_sty.rs`) and patches commands memoir has already replaced
///    (`Info:unexpected:patchcmd`).
const EMULATED_NOOP_PACKAGES: &[&str] = &["parskip"];

LoadDefinitions!({
  // The loader's early stop (`binding/content.rs`, `already_handled`: Perl `InputDefinitions` checks
  // `<file>_loaded`) turns `\usepackage[opts]{parskip}` into a return before any option or file is looked at.
  for package in EMULATED_NOOP_PACKAGES {
    assign_value(&s!("{package}.sty_loaded"), true, Some(Scope::Global));
  }

  // `\sourceatright[len]{text}` (memoir.cls:4548-4551) ends the paragraph with `text` flush right:
  // `{\unskip\nobreak\hfil\penalty50 \hskip#1\hbox{}\nobreak\hfil{#2}\parfillskip\z@\finalhyphendemerits=0\par}`.
  // The last line of the paragraph is only the break that puts `#2` at the right edge, with at least `#1` between
  // it and the text. HTML has no last line to fill, so the text is a right float at the end of the paragraph's
  // inline content (`ltx_align_floatright`, `float:right` in LaTeXML.css, as `\@qedbox` in ntheorem does for a
  // QED mark flush right at the end of a proof, Perl ntheorem.sty.ltxml:151): it sits on the last line when it
  // fits and drops to a right-aligned line of its own when it does not, as TeX's `\penalty50` allows. `text-align`
  // would not do: it applies to blocks, and the text is the paragraph's last inline. `#1` becomes the float's
  // `margin-left`, which a float keeps clear of the text it shares a line with. The `\par` is memoir's, so the
  // paragraph ends here; the group keeps `\parfillskip` local.
  DefMacro!(
    "\\sourceatright[Default:2em]{}",
    "{\\unskip\\lx@memoir@sourceatright#1{#2}\\par}"
  );
  DefConstructor!("\\lx@memoir@sourceatright Dimension {}",
  "<ltx:text class='ltx_align_floatright' cssstyle='#gap'>#2</ltx:text>",
  mode => "restricted_horizontal", enter_horizontal => true,
  after_digest => sub[whatsit] {
    if let Some(gap) = whatsit.get_arg(1) {
      whatsit.set_property("gap", s!("margin-left:{}", gap.to_attribute()));
    }
  });
});
