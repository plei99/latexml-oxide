//! Native mode (`--preload=latexml.sty`, no `rawstyles`: only registered bindings load) for TeX Live packages that
//! have no Perl binding and were skipped there (`Warning:missing_file:<pkg> … raw TeX loading is off`). Each
//! `latexml_contrib` wrapper force-raw-loads the genuine file (`alegreyasans_sty.rs` pattern), so every guard asserts
//! a result only the loaded package yields, zero errors, and the exact warnings the real file prints.
use super::{
  error_count,
  perfect_kernel_batch46::{convert_with, warning_count},
};

/// Convert `tex` natively; zero errors and exactly `warnings` warnings, none of them a skipped package.
#[track_caller]
fn native(tex: &str, warnings: usize) -> String { native_with(tex, "latexml.sty", warnings) }

/// [`native`] under the given preload (a persona such as `[xetex]latexml.sty`).
#[track_caller]
fn native_with(tex: &str, preload: &str, warnings: usize) -> String {
  let (stderr, xml) = convert_with(tex, Some(preload));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), warnings, "{stderr}");
  assert!(
    !stderr.contains("missing_file"),
    "a package was skipped:\n{stderr}"
  );
  xml
}

/// fixdif.sty:68-69: `\d` is the differential in math and the dot-under accent in text; `\d{x_i}` keeps its
/// subscript (the kernel accent errors `Script _ can only appear in math mode`).
#[test]
fn fixdif_d_is_the_differential_in_math_and_the_accent_in_text() {
  let xml = native(
    "\\documentclass{article}\n\\usepackage{fixdif}\n\\begin{document}\n\
     $\\int f\\d x$ and $\\frac{\\d{y}}{\\d{x_i}}$ and \\d{o} text.\n\\end{document}\n",
    0,
  );
  assert_eq!(
    xml.matches(r#"meaning="differential-d""#).count(),
    3,
    "{xml}"
  );
  assert!(
    xml.contains("ọ text."),
    "text-mode \\d is the accent:\n{xml}"
  );
}

/// `\usepackage[normal]{fixdif}` reaches the real file (fixdif.sty:58 `\DeclareOption{normal}`): the differential is
/// the italic `d`, not `\mathrm{d}`.
#[test]
fn fixdif_options_reach_the_package() {
  let rm = native(
    "\\documentclass{article}\n\\usepackage{fixdif}\n\\begin{document}\n$\\d x$\n\\end{document}\n",
    0,
  );
  let normal = native(
    "\\documentclass{article}\n\\usepackage[normal]{fixdif}\n\\begin{document}\n$\\d x$\n\\end{document}\n",
    0,
  );
  assert!(rm.contains(r"\mathinner{\mathrm{d}}"), "{rm}");
  assert!(
    normal.contains(r"\mathinner{d}") && !normal.contains(r"\mathrm{d}"),
    "{normal}"
  );
}

/// physics2's `\ab` after fixdif's `\d`: without fixdif `\d` keeps its one-argument accent and `\d\ab(x)` hands `\ab` to
/// the accent alone; its dispatcher (phy-ab.sty:74) loops (501 `Error:unexpected:\def … \phy@d@lx`, then
/// `Fatal:TooManyErrors`). With fixdif the roman `d` (kerned by `\mskip-\thinmuskip`, `rpadding="-1.7pt"`) applies to
/// the `\ab(x)` group, which `\ab` makes a stretchy-fenced `XMDual` of `x`. (Outside an integral the parser gives
/// this `d` no `meaning`.)
#[test]
fn fixdif_d_before_physics2_ab_does_not_loop() {
  let xml = native(
    "\\documentclass{article}\n\\usepackage{physics2}\n\\usephysicsmodule{ab}\n\\usepackage{fixdif}\n\
     \\begin{document}\n$\\d\\ab(x)$ and \\d{o}\n\\end{document}\n",
    0,
  );
  assert!(
    xml.contains(
      r##"tex="\mathinner{\mathrm{d}}\mathchoice{\mskip-3.0mu}{\mskip-3.0mu}{}{}{}\mathopen{}\mathclose{{\left(x\right)}}""##
    ),
    "the differential before the \\ab group:\n{xml}"
  );
  assert!(
    xml.contains(r#"<XMTok role="UNKNOWN" rpadding="-1.7pt">d</XMTok>"#),
    "the kerned roman d:\n{xml}"
  );
  assert!(
    xml.contains(r#"<XMTok role="OPEN" stretchy="true">(</XMTok>"#)
      && xml.contains(r#"<XMTok font="italic" role="UNKNOWN" xml:id="p1.m1.1">x</XMTok>"#)
      && xml.contains(r#"<XMTok role="CLOSE" stretchy="true">)</XMTok>"#),
    "the parenthesised x from \\ab:\n{xml}"
  );
  assert!(xml.contains(" ọ"), "text-mode \\d is the accent:\n{xml}");
}

/// berasans.sty:3-4,25: `[scaled=0.86]` reaches the real file's keyval.
#[test]
fn berasans_scaled_option_is_read() {
  let xml = native(
    "\\documentclass{article}\n\\usepackage[scaled=0.86]{berasans}\n\\begin{document}\n\
     \\makeatletter[\\fvs@Scale]\n\\end{document}\n",
    0,
  );
  latexml::util::test::assert_element(&xml, "p", &[], "<p>[0.86]</p>");
}

/// PTSerif.sty:8-9 and ebgaramond.sty:25: the `scaled` option reaches each file; ebgaramond's own
/// `\RequirePackage{fontaxes,mweights}` loads raw too (no skipped package).
#[test]
fn font_packages_read_their_scaled_option() {
  let xml = native(
    "\\documentclass{article}\n\\usepackage[scaled=0.9]{PTSerif}\n\\begin{document}\n\
     \\makeatletter[\\PTSerif@scale]\n\\end{document}\n",
    0,
  );
  latexml::util::test::assert_element(&xml, "p", &[], "<p>[0.9]</p>");
  let xml = native(
    "\\documentclass{article}\n\\usepackage[scaled=0.8]{ebgaramond}\n\\begin{document}\n\
     \\makeatletter[\\EBGaramond@scale]\n\\end{document}\n",
    0,
  );
  latexml::util::test::assert_element(&xml, "p", &[], "<p>[0.8]</p>");
}

/// fouriernc.sty:13 requires fourier, whose `[noOTF]{fourier-orns}` (fourier.sty:61) the raw file reads: `\warning` is the
/// warning sign.
#[test]
fn fouriernc_requires_fourier_and_its_ornaments() {
  let xml = native(
    "\\documentclass{article}\n\\usepackage{fouriernc}\n\\begin{document}\n\\warning\n\\end{document}\n",
    0,
  );
  latexml::util::test::assert_element(&xml, "p", &[], "<p>⚠</p>");
}

/// euler-math.sty under the luatex persona (the corpus gives fontspec documents that persona): the native binding
/// requires unicode-math with `math-style=upright` (euler-math.sty:9) and ignores the key options
/// (euler-math.sty:19-26, here `Scale=0.9`). The raw file would blank `\hbar` and `\varPhi` (its `\Umathchardef`s,
/// :42-153, run as empty macros at `\AtBeginDocument`, :38); here both stay their symbols.
#[test]
fn euler_math_keeps_the_symbols_it_redefines() {
  let xml = native_with(
    "\\documentclass{article}\n\\usepackage[Scale=0.9]{euler-math}\n\\begin{document}\n\
     $\\hbar + \\varPhi + x$\n\\end{document}\n",
    "[luatex]latexml.sty",
    0,
  );
  latexml::util::test::assert_element(
    &xml,
    "XMath",
    &[],
    r##"<XMath><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok meaning="Planck-constant-over-2-pi" name="hbar" role="ID">ℏ</XMTok><XMTok name="varPhi" role="UNKNOWN">Φ</XMTok><XMTok font="italic" role="UNKNOWN">x</XMTok></XMApp></XMath>"##,
  );
}

/// euler-math.sty:8 `\RequireTUTeX` is not run (the iftex binding passes it under every persona, Perl
/// `iftex.sty.ltxml:61` makes it a no-op): the default persona loads the package with no error and no warning, and
/// the symbols the raw file would blank stay.
#[test]
fn euler_math_loads_under_the_default_persona() {
  let xml = native(
    "\\documentclass{article}\n\\usepackage{euler-math}\n\\begin{document}\n$\\hbar$\n\\end{document}\n",
    0,
  );
  latexml::util::test::assert_element(
    &xml,
    "XMath",
    &[],
    r##"<XMath><XMTok meaning="Planck-constant-over-2-pi" name="hbar" role="ID">ℏ</XMTok></XMath>"##,
  );
}

/// futs is also Fourier's Utopia TEXT family (t1futs.fd; fourier.sty:58 `\rmdefault`): the ornament map is bound to
/// `\FourierOrns`, not to the family, so T1 body text in that family stays text.
#[test]
fn futs_text_family_is_not_decoded_as_ornaments() {
  let xml = native(
    "\\documentclass{article}\n\\usepackage[T1]{fontenc}\n\\begin{document}\n\
     {\\fontfamily{futs}\\selectfont Theorem 1 A}\n\\end{document}\n",
    0,
  );
  latexml::util::test::assert_element(&xml, "p", &[], "<p>Theorem 1 A</p>");
  let xml = native(
    "\\documentclass{article}\n\\usepackage{fourier}\n\\begin{document}\nTheorem 1 A \\warning\n\\end{document}\n",
    0,
  );
  latexml::util::test::assert_element(&xml, "p", &[], "<p>Theorem 1 A ⚠</p>");
  // fourier.sty:61 passes `noOTF`, so the xetex/luatex personas stay on the `futs` branch (fourier-orns.sty:11-27).
  let xml = native_with(
    "\\documentclass{article}\n\\usepackage{fourier}\n\\begin{document}\n\\warning\n\\end{document}\n",
    "[xetex]latexml.sty",
    0,
  );
  latexml::util::test::assert_element(&xml, "p", &[], "<p>⚠</p>");
}

/// A direct `\usepackage{fourier-orns}` under a TU persona takes the file's fontspec branch (fourier-orns.sty:16-27,
/// the OTF has the ornaments at the `futs` slots): the same ornaments, and `\grappe`, `\leftgrappe` and
/// `\rightgrappe` (:80-82, `\char"2766` etc.) their code points, not the slot letters ("1", "M").
#[test]
fn fourier_orns_under_a_tu_persona() {
  for preload in ["[xetex]latexml.sty", "[luatex]latexml.sty"] {
    let xml = native_with(
      "\\documentclass{article}\n\\usepackage{fourier-orns}\n\\begin{document}\n\
       A \\warning\\ B \\lefthand\\ C \\grappe\\ \\leftgrappe\\ \\rightgrappe\n\\end{document}\n",
      preload,
      0,
    );
    latexml::util::test::assert_element(&xml, "p", &[], "<p>A ⚠ B ☜ C ❦ ☙ ❧</p>");
  }
}

/// `\FourierOrns` stays a font switch for user text (fourier-orns-doc.tex:118-121, 150-151): `{\FourierOrns E 1 F}`
/// puts all three characters in the ornament font (pdflatex's `\showbox` shows E, 1, F in `futs`).
#[test]
fn fourier_orns_is_a_font_switch() {
  let xml = native(
    "\\documentclass{article}\n\\usepackage{fourier-orns}\n\\begin{document}\n\
     {\\FourierOrns E 1 F} {\\FourierOrns\\char`\\A}\n\\end{document}\n",
    0,
  );
  latexml::util::test::assert_element(&xml, "p", &[], "<p>❀ ⚠ ❀ ☙</p>");
}

/// fourier-orns.sty:29-72: each ornament is a slot of the `futs` font, selected by `\FourierOrns` and decoded by
/// `futs_fontmap.rs` into its Unicode ornament, not the slot letter ("1", "M", "U", "e").
#[test]
fn fourier_orns_print_their_ornaments() {
  let xml = native(
    "\\documentclass{article}\n\\usepackage{fourier-orns}\n\\begin{document}\n\
     \\noway\\ \\warning\\ \\caution\\ \\bomb\\ \\aldineleft\\ \\aldineright\\ \\aldine\\ \\lefthand\\ \\righthand\\ \\decoone\\ \\eurologo\n\
     \\end{document}\n",
    0,
  );
  latexml::util::test::assert_element(&xml, "p", &[], "<p>⛔ ⚠ ☡ 💣 ☙ ❧ ❦ ☜ ☞ ❖ €</p>");
}

/// extdash.sty:78-105 under `[shortcuts]`: `\-` followed by `--` is the em dash with half-spaces around it, `\--` the
/// en dash, `\-/` a breakable hyphen, `\-,` a half-space; `\Emdash` and `\Endash` are the same by name.
#[test]
fn extdash_shortcuts_make_the_dashes() {
  let xml = native(
    "\\documentclass{article}\n\\usepackage[shortcuts]{extdash}\n\\begin{document}\n\
     a\\---b c\\-- d e\\-/f g\\-,h i\\Emdash j k\\Endash l\n\\end{document}\n",
    0,
  );
  // The half-spaces are U+200A (hair space) before the em dash, U+2009 (thin space) after, and U+200A for `\-,`.
  latexml::util::test::assert_element(
    &xml,
    "p",
    &[],
    "<p>a\u{200A}\u{2014}\u{2009}b c\u{2013}d e-f g\u{200A}h i\u{200A}\u{2014}\u{2009}j k\u{2013}l</p>",
  );
}

/// ninecolors.sty:165 `\__ninecolors_define_one:nnn {red6}` (the default `medium` contrast set): the colour exists (without the file `Can't find color
/// named 'red6'`).
#[test]
fn ninecolors_defines_the_numbered_shades() {
  let xml = native(
    "\\documentclass{article}\n\\usepackage{ninecolors}\n\\begin{document}\n\\textcolor{red6}{Hello}\n\
     \\end{document}\n",
    0,
  );
  latexml::util::test::assert_element(
    &xml,
    "p",
    &[],
    r##"<p><text color="#DE6666">Hello</text></p>"##,
  );
}

/// memoir.cls requests memhfixc.sty after hyperref. The binding provides only `\bookautorefname` and
/// `\subfigureautorefname` (memhfixc.sty:113-114): its patches call hyperref internals LaTeXML lacks (a raw load makes
/// `\book*{Bk}` fail with `Error:undefined:\Hy@MakeCurrentHrefAuto`, :76-83), so a memoir document with hyperref
/// converts with 0 errors, LaTeXML's own anchors, ids and structure, and no skipped package. The one warning is the
/// `\@iffirstamp` conditional memoir/hyperref define without `\if` (also present under `rawstyles`).
#[test]
fn memhfixc_is_a_no_op_for_memoir_with_hyperref() {
  let tex = "\\documentclass{memoir}\n\\usepackage{hyperref}\n\\setsecnumdepth{subsection}\n\\begin{document}\n\
     \\book*{Bk}\n\\chapter{Alpha}\\label{c}\n\\section{Beta}\\footnote{Note.}\n\\subsubsection{Delta}\n\\section*{Epsilon}\n\
     See \\ref{c}.\n\\begin{figure}\\caption{Cap}\\label{f}\\end{figure}\nFig \\ref{f}. \\bookautorefname/\\subfigureautorefname.\n\\end{document}\n";
  let (stderr, xml) = convert_with(tex, Some("latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  assert_eq!(
    stderr
      .matches("Warning:misdefined:\\@iffirstamp The conditional \\@iffirstamp is being defined but doesn't start with \\if")
      .count(),
    1,
    "{stderr}"
  );
  assert!(!stderr.contains("missing_file"), "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "text",
    &[r#"fontsize="207%""#],
    r#"<text align="center" font="bold" fontsize="207%">Bk</text>"#,
  );
  latexml::util::test::assert_element(
    &xml,
    "title",
    &[],
    r#"<title><tag close=" ">Chapter 1</tag>Alpha</title>"#,
  );
  latexml::util::test::assert_element(
    &xml,
    "section",
    &[r#"xml:id="chapter1.section1""#],
    r##"<section inlist="toc" xml:id="chapter1.section1"><tags><tag>1.1</tag><tag role="autoref">section 1.1</tag><tag role="refnum">1.1</tag><tag role="typerefnum">§1.1</tag></tags><title><tag close=" ">1.1</tag>Beta</title><toctitle><tag close=" ">1.1</tag>Beta</toctitle><note mark="1" role="footnote" xml:id="footnote1"><tags><tag>1</tag><tag role="autoref">footnote 1</tag><tag role="refnum">1</tag><tag role="typerefnum">footnote 1</tag></tags>Note.</note><subsubsection inlist="toc" xml:id="chapter1.section1.subsection0.subsubsectionx1"><title>Delta</title><toctitle>Delta</toctitle></subsubsection></section>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "section",
    &[r#"xml:id="chapter1.sectionx1""#],
    r##"<section xml:id="chapter1.sectionx1"><title>Epsilon</title><para xml:id="chapter1.sectionx1.p1"><p>See <ref labelref="LABEL:c"/>.</p></para><figure inlist="lof" labels="LABEL:f" xml:id="chapter1.figure1"><tags><tag>Figure 1.1</tag><tag role="autoref">Figure 1.1</tag><tag role="refnum">1.1</tag><tag role="typerefnum">Figure 1.1</tag></tags><toccaption><tag close=" ">1.1</tag>Cap</toccaption><caption><tag close=": ">Figure 1.1</tag>Cap</caption></figure><para xml:id="chapter1.sectionx1.p2"><p>Fig <ref labelref="LABEL:f"/>. Book/Subfigure.</p><pagination role="newpage"/></para></section>"##,
  );
}

/// spectralsequences.sty: `{sseqdata}` + `\printpage` draw the diagram (an `ltx:picture`); the class labels keep
/// their math subscripts; `\d` outside the diagram stays the accent.
#[test]
fn spectralsequences_draws_the_diagram() {
  let xml = native(
    "\\documentclass{article}\n\\usepackage{spectralsequences}\n\\begin{document}\nBefore \\d{o} after.\n\
     \\begin{sseqdata}[name=a, homological Serre grading, y axis gap=2em]\n\\class[\"H_0(F)\"](0,0)\n\
     \\class[\"H_1(F)\"](0,1)\n\\class[\"H_0(F)\"](2,0)\n\\d[\"g_0\", ->]2(2,0)\n\\end{sseqdata}\n\
     \\printpage[name=a, page=2]\n\\end{document}\n",
    0,
  );
  assert!(
    xml.contains(r#"<picture height="118.39" width="165.75" xml:id="p2.pic1">"#),
    "{xml}"
  );
  // The class at (0,1) is labelled by its math `H_1(F)`.
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"tex="H_{1}(F)""#],
    r##"<Math mode="inline" tex="H_{1}(F)" text="H _ 1 * F" xml:id="p2.pic1.m7"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">H</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp><XMDual><XMRef idref="p2.pic1.m7.1"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p2.pic1.m7.1">F</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp></XMath></Math>"##,
  );
  assert!(xml.contains("Before ọ after."), "{xml}");
}

/// rank-2-roots.sty: `{rootSystem}{G}` (:111) with `\roots` (:481) and `\simpleroots` (:134) draws the G2 root system: an
/// `ltx:picture` of the package's size holding its 31 paths, the first the 12-point root polygon.
#[test]
fn rank_2_roots_draws_the_root_system() {
  let xml = native(
    "\\documentclass{article}\n\\usepackage{rank-2-roots}\n\\begin{document}\n\
     \\begin{tikzpicture}[baseline=-.5]\n\\begin{rootSystem}{G}\n\\roots\n\\simpleroots\n\\end{rootSystem}\n\
     \\end{tikzpicture}\n\\end{document}\n",
    0,
  );
  assert!(
    xml.contains(r#"<picture height="72.07" width="79.29" xml:id="p1.pic1">"#),
    "{xml}"
  );
  assert_eq!(xml.matches("<svg:path").count(), 31, "{xml}");
  assert!(
    xml.contains(r#"<svg:path d="M 19.69 0 L 29.53 17.05 L 9.84 17.05 L 0 34.1 L -9.84 17.05"#),
    "{xml}"
  );
}

/// tkz-euclide.sty: `\tkzInit`/`\tkzDefPoints`/`\tkzDrawSegment` draw a segment (an `ltx:picture` holding the 3cm path).
#[test]
fn tkz_euclide_draws_a_segment() {
  let xml = native(
    "\\documentclass{article}\n\\usepackage{tkz-euclide}\n\\begin{document}\n\
     \\begin{tikzpicture}\\tkzInit[xmin=0,xmax=4,ymin=0,ymax=4]\\tkzDefPoints{0/0/A,3/0/B}\\tkzDrawSegment(A,B)\n\
     \\end{tikzpicture}\n\\end{document}\n",
    0,
  );
  assert_eq!(xml.matches("<picture").count(), 1, "{xml}");
  assert!(
    xml.contains(r#"<svg:path d="M 0 0 M 0 0 L 118.11 0""#),
    "the 3cm segment A–B:\n{xml}"
  );
}

/// `\usetheme{metropolis}` requests `beamerthememetropolis` (beamer_cls.rs `\beamer@calltheme`); its
/// `\metroset` is defined. The one warning is the real file's (beamerthememetropolis.sty: Fira fonts need XeLaTeX
/// or LuaLaTeX).
#[test]
fn beamer_metropolis_theme_loads() {
  let (stderr, xml) = convert_with(
    "\\documentclass{beamer}\n\\usetheme{metropolis}\n\\begin{document}\n\
     [\\ifdefined\\metroset yes\\else no\\fi]\n\\end{document}\n",
    Some("latexml.sty"),
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  assert!(
    stderr.contains(
      "Warning:latex:(beamerthememetropolis) Package beamerthememetropolis Warning: You need to compile with \
       XeLaTeX or LuaLaTeX to use the Fira fonts"
    ),
    "{stderr}"
  );
  assert!(!stderr.contains("missing_file"), "{stderr}");
  latexml::util::test::assert_element(&xml, "p", &[], "<p>[yes]</p>");
}

/// `\usetheme[shadow=false]{Darmstadt}` reaches beamerthemeDarmstadt.sty:11-12, which passes the option on to
/// beamerinnerthemerounded; that theme (loaded by the wrapped file's `\useinnertheme{rounded}`) records it in
/// `\beamer@themerounded@shadow` (beamerinnerthemerounded.sty:11-12).
#[test]
fn beamer_darmstadt_theme_loads_its_nested_themes() {
  let xml = native(
    "\\documentclass{beamer}\n\\usetheme[shadow=false]{Darmstadt}\n\\begin{document}\n\
     \\makeatletter[\\ifdefined\\beamer@themerounded@shadow \\beamer@themerounded@shadow\\else undef\\fi]\n\
     \\end{document}\n",
    0,
  );
  latexml::util::test::assert_element(&xml, "p", &[], "<p>[false]</p>");
}

const CTEX_DOC: &str = "\\documentclass{article}\n\\usepackage[UTF8,fontset=mac]{ctex}\n\\begin{document}\n\
  [\\ifdefined\\CTEXsetup yes\\else no\\fi][\\contentsname]\\zihao{4} 你好\n\\section{标题}\n\\end{document}\n";

/// ctex.sty under the xetex persona (what the corpus protocol gives a document loading ctex or xeCJK): `\\zihao{4}` is
/// the 14pt size (`fontsize="133%"`, also on the heading), `\\contentsname` is 目录, `\\CTEXsetup` is defined, and xeCJK
/// loads through `xecjk_sty.rs`.
#[test]
fn ctex_loads_under_the_xetex_persona() {
  let xml = native_with(CTEX_DOC, "[xetex]latexml.sty", 0);
  assert!(xml.contains("[yes][目录]"), "{xml}");
  assert!(
    xml.contains(r#"<text fontsize="133%">你好</text>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<title fontsize="133%"><tag close=" ">1</tag>标题</title>"#),
    "{xml}"
  );
}

const CTEX_BARE: &str =
  "\\documentclass{article}\n\\usepackage{ctex}\n\\begin{document}\nx\n\\end{document}\n";

/// Under the default pdfTeX persona ANY `\usepackage{ctex}` errors, as under pdflatex: with no `fontset` option ctex
/// detects the platform (`fandol`, or `mac` where `/System/Library/Fonts/Menlo.ttc` exists; ctex.sty:830-841) and the
/// fontset is `unavailable in current mode` under pdfTeX (`\msg_critical`, ctex.sty:802), which ends only that file
/// (expl3-code.tex:11101-11107). The binding reports the real file's error, no skipped-package warning. Known
/// difference: on the mac fontset path pdflatex reports a second error (`File ended while scanning use of
/// \use_none_delimit_by_q_stop:w`, ctex-fontset-mac.def l.90); this engine reports one, which is what the count pins.
#[test]
fn ctex_fontset_is_refused_under_pdftex() {
  let (stderr, _xml) = convert_with(CTEX_BARE, Some("latexml.sty"));
  assert_eq!(error_count(&stderr), 1, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    regex::Regex::new(
      r"Critical Package ctex Error: CTeX fontset `(fandol|mac)' is unavailable in current"
    )
    .unwrap()
    .is_match(&stderr),
    "{stderr}"
  );
  assert!(!stderr.contains("missing_file"), "{stderr}");
}
