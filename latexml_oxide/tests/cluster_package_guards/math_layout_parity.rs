//! Math guards: `\substack` letters upright under a raw class, `\varprojlim` limits beside the operator,
//! `\widehat` a small caret, a script on `L_{g*}` unparsed, `\partial_i` scripts floating onto the next
//! atom, and an unparsed formula that lost its `\qquad` and its scripts' bases.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{convert_html, error_count, warning_count};

/// A raw class's size command (`\DeclareRobustCommand{\scriptsize}{\@setfontsize…}`, mem10.clo:70, so
/// memoir's) ends in `\selectfont`; run in math, that merged the text font's upright shape into the math
/// font and every letter of `\substack`'s `style=\scriptsize` came out `mathvariant="normal"`
/// (a `\substack` under a limit or sum in a memoir document).
/// In math `\@setfontsize` now runs only the size update, as the class bindings' own size switches do
/// (dialect.rs); the letters stay italic. Perl's `\@setfontsize` never reaches `\selectfont`
/// (OXIDIZED_DESIGN_DIVERGENCES #288).
#[test]
fn raw_size_switch_keeps_math_italic() {
  let (stderr, html) = convert_html(
    "\\documentclass{article}\n\\usepackage{amsmath}\n\\makeatletter\n\
     \\DeclareRobustCommand{\\scriptsize}{\\@setfontsize\\scriptsize\\@viipt{8}}\n\\makeatother\n\
     \\begin{document}\n$a{\\scriptsize x}$\n\
     \\[ \\lim_{\\substack{\\to \\\\ x \\in U}} F|_U \\]\n\\end{document}\n",
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &html,
    "math",
    &["id=\"p1.m1\""],
    "<math id=\"p1.m1\" class=\"ltx_Math\" alttext=\"a{x}\" display=\"inline\"><mrow><mi>a</mi><mo>\u{2062}</mo><mi \
     mathsize=\"0.700em\">x</mi></mrow></math>",
  );
  assert_element(
    &html,
    "mtable",
    &[],
    "<mtable rowspacing=\"0pt\"><mtr><mtd><mo stretchy=\"false\">\u{2192}</mo></mtd></mtr><mtr><mtd><mrow><mi>x</mi><mo>\u{2208}</mo><mi>U</mi></mrow></mtd></mtr></mtable>",
  );
}

/// The witness class itself: memoir's `\scriptsize` is that robust `\@setfontsize` command. memoir.cls
/// raw-loads with the one `Warning:misdefined:\@iffirstamp` (memoir.cls:5454) it always gives.
#[test]
fn memoir_substack_keeps_math_italic() {
  let (stderr, html) = convert_html(
    "\\documentclass{memoir}\n\\usepackage{amsmath}\n\\begin{document}\n\
     \\[ \\lim_{\\substack{\\to \\\\ x \\in U}} F|_U \\]\n\\end{document}\n",
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  assert!(
    stderr.contains("Warning:misdefined:\\@iffirstamp"),
    "{stderr}"
  );
  assert_element(
    &html,
    "mtable",
    &[],
    "<mtable rowspacing=\"0pt\"><mtr><mtd><mo stretchy=\"false\">\u{2192}</mo></mtd></mtr><mtr><mtd><mrow><mi>x</mi><mo>\u{2208}</mo><mi>U</mi></mrow></mtd></mtr></mtable>",
  );
}

/// amsopn.sty:98-108 defines `\varinjlim`, `\varprojlim`, `\varliminf` and `\varlimsup` as
/// `\mathop{…}\nmlimits@` with `\nmlimits@` = `\displaylimits` (:109): their limits go below or above in
/// display style and beside inline. Perl gives them no `scriptpos`, so a limit was always an `msub` or
/// `msup` at the operator's side (OXIDIZED_DESIGN_DIVERGENCES #598). The `XMDual` the operator is made of
/// carries `mid` on its content branch, which the parser's script attachment reads as Perl's `NewScript`
/// does (MathParser.pm:1602-1614). The `nonamelimits` option is not honoured.
#[test]
fn varlim_operators_take_display_limits_below() {
  let (stderr, html) = convert_html(
    "\\documentclass{article}\n\\usepackage{amsmath}\n\\begin{document}\n\
     \\[ \\varprojlim_r A \\quad \\varinjlim_{n\\to\\infty} B \\quad \\varliminf_n C \\quad \\varlimsup^n D \\]\n\
     $\\varprojlim_r A \\varinjlim_r B \\varliminf_n C \\varlimsup^n D$\n\\end{document}\n",
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &html,
    "math",
    &["id=\"S0.Ex1.m1\""],
    r##"<math id="S0.Ex1.m1" class="ltx_Math" alttext="\varprojlim_{r}A\quad\varinjlim_{n\to\infty}B\quad\varliminf_{n}C\quad\varlimsup^{n}D" display="block"><mrow><mrow><munder><munder accentunder="true"><mi>lim</mi><mo>←</mo></munder><mi>r</mi></munder><mo lspace="0.167em">⁡</mo><mi>A</mi></mrow><mspace style="width:1.167em;" width="1.167em"></mspace><mrow><munder><munder accentunder="true"><mi>lim</mi><mo>→</mo></munder><mrow><mi>n</mi><mo stretchy="false">→</mo><mi mathvariant="normal">∞</mi></mrow></munder><mo lspace="0.167em">⁡</mo><mi>B</mi></mrow><mspace style="width:1.167em;" width="1.167em"></mspace><mrow><munder><munder accentunder="true"><mrow class="ltx_underline" style="border-bottom:0.04em solid currentColor;padding-bottom:0.12em;margin-bottom:0.04em"><mi>lim</mi></mrow><mo mathsize="0" stretchy="false">¯</mo></munder><mi>n</mi></munder><mo lspace="0.167em">⁡</mo><mi>C</mi></mrow><mspace style="width:1.167em;" width="1.167em"></mspace><mrow><mover><mover accent="true"><mrow class="ltx_overline" style="border-top:0.04em solid currentColor;padding-top:0.12em;margin-top:0.04em"><mi>lim</mi></mrow><mo mathsize="0" stretchy="false">¯</mo></mover><mi>n</mi></mover><mo lspace="0.167em">⁡</mo><mi>D</mi></mrow></mrow></math>"##,
  );
  assert_element(
    &html,
    "math",
    &["id=\"p1.m1\""],
    r##"<math id="p1.m1" class="ltx_Math" alttext="\varprojlim_{r}A\varinjlim_{r}B\varliminf_{n}C\varlimsup^{n}D" display="inline"><mrow><msub><munder accentunder="true"><mi>lim</mi><mo>←</mo></munder><mi>r</mi></msub><mo lspace="0.167em">⁡</mo><mrow><mi>A</mi><mo lspace="0.167em">⁢</mo><mrow><msub><munder accentunder="true"><mi>lim</mi><mo>→</mo></munder><mi>r</mi></msub><mo lspace="0.167em">⁡</mo><mrow><mi>B</mi><mo lspace="0.167em">⁢</mo><mrow><msub><munder accentunder="true"><mrow class="ltx_underline" style="border-bottom:0.04em solid currentColor;padding-bottom:0.12em;margin-bottom:0.04em"><mi>lim</mi></mrow><mo mathsize="0" stretchy="false">¯</mo></munder><mi>n</mi></msub><mo lspace="0.167em">⁡</mo><mrow><mi>C</mi><mo lspace="0.167em">⁢</mo><mrow><msup><mover accent="true"><mrow class="ltx_overline" style="border-top:0.04em solid currentColor;padding-top:0.12em;margin-top:0.04em"><mi>lim</mi></mrow><mo mathsize="0" stretchy="false">¯</mo></mover><mi>n</mi></msup><mo lspace="0.167em">⁡</mo><mi>D</mi></mrow></mrow></mrow></mrow></mrow></mrow></mrow></math>"##,
  );
}

/// `\widehat` is U+0302 over its base (OXIDIZED_DESIGN_DIVERGENCES #599): the only circumflex the math
/// fonts have a horizontal construction for (U+005E stays a small caret), so it spans `AB` and `\bigoplus` as TeX's
/// wide hat does (`\widehat{\bigoplus}`, `\widehat{\mathfrak{gl}(1)}`).
/// `\hat` stays the narrow `^`, `\widetilde` the `~`.
#[test]
fn widehat_stretches_with_u0302() {
  let (stderr, html) = convert_html(
    "\\documentclass{article}\n\\usepackage{amsmath,amssymb}\n\\begin{document}\n\
     $\\widehat{AB}\\ \\hat{AB}\\ \\widetilde{AB}$\n\
     \\[ \\widehat{\\bigoplus}\\ \\widehat{\\mathfrak{gl}(1)} \\]\n\\end{document}\n",
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(&html, "math", &["id=\"p1.m1\""], &r##"<math id="p1.m1" class="ltx_Math" alttext="\widehat{AB}\ \hat{AB}\ \widetilde{AB}" display="inline"><mrow><mover accent="true"><mrow><mi>A</mi><mo>⁢</mo><mi>B</mi></mrow><mo>@U0302@</mo></mover><mo lspace="0.500em">⁢</mo><mover accent="true"><mrow><mi>A</mi><mo>⁢</mo><mi>B</mi></mrow><mo>^</mo></mover><mo>⁢</mo><mover accent="true"><mrow><mi>A</mi><mo>⁢</mo><mi>B</mi></mrow><mo>~</mo></mover></mrow></math>"##.replace("@U0302@", "\u{0302}"));
  assert_element(&html, "math", &["id=\"S0.Ex1.m1\""], &r##"<math id="S0.Ex1.m1" class="ltx_Math" alttext="\widehat{\bigoplus}\ \widehat{\mathfrak{gl}(1)}" display="block"><mrow><mover accent="true"><mo movablelimits="false">⨁</mo><mo>@U0302@</mo></mover><mo lspace="0.667em">⁢</mo><mover accent="true"><mrow><mi>𝔤</mi><mo>⁢</mo><mi>𝔩</mi><mo>⁢</mo><mrow><mo stretchy="false">(</mo><mn>1</mn><mo stretchy="false">)</mo></mrow></mrow><mo>@U0302@</mo></mover></mrow></math>"##.replace("@U0302@", "\u{0302}"));
}

/// Perl's `Subscript` is `aSubscript (PUNCT(?) aSubscript)(s?)` (MathGrammar:85-92): `g*` in `L_{g*}`
/// is two items of one script, joined by an invisible comma, and the formula parses. Ours had no
/// rule for an expression closed by a bare `*`, `\ast`, `\cdot` or `\times`, so every pushforward
/// `f_*`-with-a-letter was left unparsed (`L_{g *}\xi` after a `\qquad`), and that fallback had lost the
/// `\qquad` as well. The `lspace="0.222em"` on the invisible comma is Perl's own `adjust_spacing` of its
/// `XMTok`, kept for parity; TeX sets no space there.
#[test]
fn script_ending_in_a_star_parses() {
  let (stderr, html) = convert_html(
    "\\documentclass{article}\n\\usepackage{amsmath,amssymb,mathtools}\n\\begin{document}\n\
     \\[ L \\colon G \\times \\mathfrak{g} \\xrightarrow{\\sim} TG \\qquad (g, \\xi) \\mapsto L_{g *} \\xi. \\]\n\
     $L_{g\\ast}\\xi$ $f^{g\\cdot}$\n\\end{document}\n",
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(!html.contains("ltx_math_unparsed"), "{html}");
  assert_element(
    &html,
    "math",
    &["id=\"S0.Ex1.m1\""],
    r##"<math id="S0.Ex1.m1" class="ltx_Math" alttext="L\colon G\times\mathfrak{g}\xrightarrow{\sim}TG\qquad(g,\xi)\mapsto L_{g*}\xi." display="block"><mrow><mrow><mrow><mi>L</mi><mo lspace="0.278em" rspace="0.278em">:</mo><mrow><mrow><mi>G</mi><mo lspace="0.222em" rspace="0.222em">×</mo><mi>𝔤</mi></mrow><mover accent="true"><mo>→</mo><mo mathsize="0.700em">∼</mo></mover><mrow><mi>T</mi><mo>⁢</mo><mi>G</mi></mrow></mrow></mrow><mspace style="width:2em;" width="2em"></mspace><mrow><mrow><mo stretchy="false">(</mo><mi>g</mi><mo>,</mo><mi>ξ</mi><mo stretchy="false">)</mo></mrow><mo stretchy="false">↦</mo><mrow><msub><mi>L</mi><mrow><mi>g</mi><mo lspace="0.222em">⁣</mo><mo>∗</mo></mrow></msub><mo>⁢</mo><mi>ξ</mi></mrow></mrow></mrow><mo lspace="0em">.</mo></mrow></math>"##,
  );
  assert_element(
    &html,
    "math",
    &["id=\"p1.m1\""],
    r##"<math id="p1.m1" class="ltx_Math" alttext="L_{g\ast}\xi" display="inline"><mrow><msub><mi>L</mi><mrow><mi>g</mi><mo lspace="0.222em">⁣</mo><mo>∗</mo></mrow></msub><mo>⁢</mo><mi>ξ</mi></mrow></math>"##,
  );
  assert_element(
    &html,
    "math",
    &["id=\"p1.m2\""],
    r##"<math id="p1.m2" class="ltx_Math" alttext="f^{g\cdot}" display="inline"><msup><mi>f</mi><mrow><mi>g</mi><mo lspace="0.222em">⁣</mo><mo>⋅</mo></mrow></msup></math>"##,
  );
}

/// Perl runs its kludge on every failed parse (MathParser.pm:322, :374), and it keeps the wide
/// spaces `filter_hints` made into punctuation. Ours ran it only for formulae with fences and read
/// the hints once, unlinking them, so a formula left unparsed with no parenthesis kept its scripts as
/// markers presented over an empty base and lost its `\qquad`: `a_i\qquad=\to b` (two relations in a
/// row: unparsed in Perl too, whose XML has `<XMHint name="qquad" role="PUNCT" width="20pt"/>`).
#[test]
fn unparsed_formula_keeps_its_scripts_and_wide_spaces() {
  let (stderr, html) = convert_html(
    "\\documentclass{article}\n\\usepackage{amsmath}\n\\begin{document}\n\
     \\[ a_i^j \\qquad = \\to b \\]\n\\[ a + \\text{and $c_i \\qquad = \\to b$} \\]\n\\end{document}\n",
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 2, "{stderr}");
  assert_eq!(
    stderr.matches("Warning:unparsed_math:").count(),
    2,
    "{stderr}"
  );
  assert!(stderr.contains("Warning:unparsed_math:"), "{stderr}");
  assert_element(
    &html,
    "math",
    &["id=\"S0.Ex1.m1\""],
    r##"<math id="S0.Ex1.m1" class="ltx_math_unparsed" alttext="a_{i}^{j}\qquad=\to b" display="block"><mrow><msubsup><mi>a</mi><mi>i</mi><mi>j</mi></msubsup><mspace style="width:2em;" width="2em"></mspace><mo rspace="0em">=</mo><mo lspace="0em" stretchy="false">→</mo><mi>b</mi></mrow></math>"##,
  );
  // a failed formula inside a parsed one (an `\text` argument) is kludged where it fails, before the outer copy
  assert_element(
    &html,
    "math",
    &["id=\"S0.Ex2.m1\""],
    r##"<math id="S0.Ex2.m1" class="ltx_Math" alttext="a+\text{and $c_{i}\qquad=\to b$}" display="block"><mrow><mi>a</mi><mo>+</mo><mrow><mtext>and </mtext><mrow><msub><mi>c</mi><mi>i</mi></msub><mspace style="width:2em;" width="2em"></mspace><mo rspace="0em">=</mo><mo lspace="0em" stretchy="false">→</mo><mi>b</mi></mrow></mrow></mrow></math>"##,
  );
}
