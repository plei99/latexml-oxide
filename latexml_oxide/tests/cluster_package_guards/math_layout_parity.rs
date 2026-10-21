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

