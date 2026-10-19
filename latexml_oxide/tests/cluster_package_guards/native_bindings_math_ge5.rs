//! Native bindings for the packages of the arXiv mathematics sample with no Perl binding
//! (OXIDIZED_DESIGN #588): each guard converts in the default configuration, where an unbound
//! package is a missing file and its commands undefined, and asserts what only the binding produces.

fn doc(preamble: &str, body: &str) -> String {
  format!(
    "\\documentclass{{article}}\n{preamble}\n\\begin{{document}}\n{body}\n\\end{{document}}\n"
  )
}

fn clean(preamble: &str, body: &str) -> String {
  let (log, xml) = super::convert(&doc(preamble, body), false);
  assert_eq!(super::error_count(&log), 0, "{log}");
  assert_eq!(
    super::perfect_kernel_batch46::warning_count(&log),
    0,
    "{log}"
  );
  xml
}

/// `<XMTok …>text</XMTok>` tokens of the XML, in order.
fn tokens(xml: &str) -> Vec<String> {
  let re = regex::Regex::new(r"<XMTok[^>]*>([^<]*)</XMTok>").unwrap();
  re.captures_iter(xml).map(|c| c[1].to_string()).collect()
}

/// extarrows (#588f): each arrow is amsmath's labelled arrow over a stretchy operator.
#[test]
fn extarrows_label_stretchy_operators() {
  let xml = clean(
    "\\usepackage{extarrows}",
    "$A \\xlongequal{f} B \\xLongrightarrow[u]{v} C \\xLongleftrightarrow{w} D$",
  );
  assert!(
    xml.contains(r#"<XMTok meaning="equals" role="RELOP" stretchy="true">=</XMTok>"#),
    "{xml}"
  );
  assert!(xml.contains(r#"stretchy="true">⇒</XMTok>"#), "{xml}");
  assert!(xml.contains(r#"stretchy="true">⇔</XMTok>"#), "{xml}");
  assert_eq!(xml.matches("role=\"OVERACCENT\"").count(), 3, "{xml}");
  assert_eq!(xml.matches("role=\"UNDERACCENT\"").count(), 1, "{xml}");
}

/// centernot (#588c), colonequals (#588d): negated and colon relations as single characters.
#[test]
fn centernot_and_colonequals_are_relation_characters() {
  let xml = clean(
    "\\usepackage{amsmath,centernot,colonequals}",
    "$a\\centernot\\to b$ $x\\centernot\\mid y$ $x\\colonequals y$ $a\\coloncolonequals b$ $a\\ratio b$ \\colonequals",
  );
  let toks = tokens(&xml);
  // `\not`'s overlay (the MathML stage writes the precomposed U+219B, U+2224).
  for want in ["→\u{338}", "∣\u{338}", "≔", "⩴", "∶"] {
    assert!(toks.iter().any(|t| t == want), "{want} in {toks:?}");
  }
  assert!(xml.contains(r#"tex="x\colonequals y""#), "{xml}");
}

/// faktor (#588e): a quotient of raised and lowered parts, reverting to `\faktor`.
#[test]
fn faktor_is_a_quotient() {
  let xml = clean("\\usepackage{amssymb,faktor}", "$\\faktor[\\mathbb]{Z}{n}$");
  assert!(
    xml.contains(r#"meaning="quotient" name="faktor" role="MULOP">╱</XMTok>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<XMTok font="blackboard" role="UNKNOWN" yoffset="0.35em">Z</XMTok>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<XMTok font="blackboard" role="UNKNOWN" yoffset="-0.35em">n</XMTok>"#),
    "{xml}"
  );
  assert!(xml.contains(r#"tex="\faktor[\mathbb]{Z}{n}""#), "{xml}");
}

/// tensor (#588g), leftindex (#588o): staggered indices are separate script columns.
#[test]
fn tensor_and_leftindex_build_multiscripts() {
  let (log, html) = super::perfect_kernel_batch46::convert_html(&doc(
    "\\usepackage{tensor,leftindex}",
    "$\\tensor{T}{^a_b^c}$ $\\tensor*{T}{^a_b}$ $\\tensor[^a_b]{T}{_c}$ $R\\indices{^\\mu_\\nu}$ $\\leftindex^{t}_{s}{A}$",
  ));
  assert_eq!(super::error_count(&log), 0, "{log}");
  assert_eq!(
    super::perfect_kernel_batch46::warning_count(&log),
    0,
    "{log}"
  );
  // Staggered: T, (·,a), (b,·), (·,c).
  assert!(html.contains("<mmultiscripts><mi>T</mi><mrow></mrow><mi>a</mi><mi>b</mi><mrow></mrow><mrow></mrow><mi>c</mi></mmultiscripts>"), "{html}");
  // Aligned: one sub/sup pair.
  assert!(
    html.contains("<msubsup><mi>T</mi><mi>b</mi><mi>a</mi></msubsup>"),
    "{html}"
  );
  assert!(
    html.contains(
      "<mprescripts></mprescripts><mrow></mrow><mi>a</mi><mi>b</mi><mrow></mrow></mmultiscripts>"
    ),
    "{html}"
  );
  // `\indices` attaches to the preceding atom.
  assert!(
    html.contains(
      "<mmultiscripts><mi>R</mi><mrow></mrow><mi>μ</mi><mi>ν</mi><mrow></mrow></mmultiscripts>"
    ),
    "{html}"
  );
  assert!(
    html.contains(
      "<mmultiscripts><mi>A</mi><mprescripts></mprescripts><mi>s</mi><mi>t</mi></mmultiscripts>"
    ),
    "{html}"
  );
}

/// old-arrows (#588z): the arrows keep their characters; the added names are arrows.
#[test]
fn old_arrows_keep_semantic_arrows() {
  let xml = clean(
    "\\usepackage{amsmath}\\usepackage[new]{old-arrows}",
    "$a\\longrightarrow b\\varrightarrow c\\longhookrightarrow d\\varlongmapsto e$",
  );
  assert_eq!(tokens(&xml), ["a", "⟶", "b", "→", "c", "↪", "d", "⟼", "e"]);
}

/// stackengine (#588b): math stacks are accents over the anchor, text stacks rows.
#[test]
fn stackengine_stacks() {
  let xml = clean(
    "\\usepackage{stackengine}",
    "\\stackMath $a\\stackon[1pt]{=}{\\scriptscriptstyle\\Delta}b$ \\stackText \\stackunder{A}{B} \\Shortstack[l]{x y}\
     \\stackinset{r}{}{t}{}{INSET}{ANCHOR}",
  );
  assert!(
    xml.contains(r#"<XMTok fontsize="50%" name="Delta" role="OVERACCENT">Δ</XMTok>"#),
    "{xml}"
  );
  // The item's style does not leak past the stack.
  assert!(
    xml.contains(r#"<XMTok font="italic" role="UNKNOWN">b</XMTok>"#),
    "{xml}"
  );
  assert!(xml.contains(r#"<inline-block vattach="top">"#), "{xml}");
  assert!(
    xml.contains(r#"<inline-block align="left" vattach="bottom">"#),
    "{xml}"
  );
  // The inset over its anchor, in one grid cell placed by the corner classes.
  let inset = xml
    .find(
      r#"<inline-block align="right" class="ltx_stackinset ltx_stackinset_t ltx_stackinset_r">"#,
    )
    .expect(&xml);
  let rest = &xml[inset..];
  let end = rest.find("</inline-block>").unwrap();
  let (i, a) = (
    rest.find("<p>INSET</p>").unwrap(),
    rest.find("<p>ANCHOR</p>").unwrap(),
  );
  assert!(i < a && a < end, "{xml}");
}

/// mathalpha / mathalfa (#588j): each face is set in its design's family.
#[test]
fn mathalfa_faces_follow_their_design() {
  let (log, html) = super::perfect_kernel_batch46::convert_html(&doc(
    "\\usepackage{amssymb}\\usepackage[cal=boondox,scr=euler,bb=pazo]{mathalfa}",
    "$\\mathcal{A}\\mathscr{B}\\mathbfcal{C}\\mathbb{R}$",
  ));
  assert_eq!(super::error_count(&log), 0, "{log}");
  assert_eq!(
    super::perfect_kernel_batch46::warning_count(&log),
    0,
    "{log}"
  );
  assert!(
    html.contains(r#"<mi class="ltx_font_mathscript">𝒜</mi>"#),
    "{html}"
  );
  assert!(
    html.contains(r#"<mi class="ltx_font_mathcaligraphic">ℬ</mi>"#),
    "{html}"
  );
  assert!(
    html.contains(r#"<mi class="ltx_font_mathscript">𝓒</mi>"#),
    "{html}"
  );
  // pazo has no bold blackboard: `bfbb` is the package's error.
  let (log, _) = super::convert(&doc("\\usepackage[bb=pazo,bfbb]{mathalpha}", "x"), false);
  assert_eq!(super::error_count(&log), 1, "{log}");
  assert!(log.contains("but `pazo' has no bold"), "{log}");
}

/// nccmath (#588w): medium-size display-style math, display-style arrays, `\intertext[skip]`.
#[test]
fn nccmath_medium_math_and_darray() {
  let xml = clean(
    "\\usepackage{nccmath}",
    "$\\mfrac{1}{2}$\n\\begin{align} a &= b \\intertext[3pt]{so} c &= d \\end{align}\n\\[\\begin{darray}{rcl} x &=& \\frac12 \\end{darray}\\]",
  );
  assert!(xml.contains(r#"fontsize="80%""#), "{xml}");
  assert!(xml.contains(r#"<p class="ltx_intertext">so</p>"#), "{xml}");
  // `darray`'s cells are in display style (`>{\displaystyle}` on each column).
  assert!(
    xml
      .contains(r#"\begin{array}[]{&gt;{\displaystyle}r&gt;{\displaystyle}c&gt;{\displaystyle}l}"#),
    "{xml}"
  );
}

/// calligra (#588r), anyfontsize (#588v), inconsolata (#588n): the fonts the text carries.
#[test]
fn font_packages_set_their_families() {
  let xml = clean(
    "\\usepackage{calligra,anyfontsize}\\usepackage[var0]{inconsolata}",
    "{\\calligra C} \\textcalligra{D} {\\fontsize{37}{40}\\selectfont B} \\texttt{t} \\altzero",
  );
  assert!(xml.contains(r#"<text font="script">C</text>"#), "{xml}");
  assert!(xml.contains(r#"<text font="script">D</text>"#), "{xml}");
  assert!(xml.contains(r#"<text fontsize="370%">B</text>"#), "{xml}");
  assert!(
    xml.contains("<text font=\"typewriter\">0\u{FE00}</text>"),
    "{xml}"
  );
  // Without `var0` the font's zero is slashed and `\altzero` is the plain one.
  let xml = clean("\\usepackage{inconsolata}", "\\altzero");
  assert!(xml.contains("<text font=\"typewriter\">0</text>"), "{xml}");
}

/// environ (#588l): the body, trimmed, nested environments counted, arguments from the body.
#[test]
fn environ_collects_the_body() {
  let xml = clean(
    "\\usepackage{environ}\\NewEnviron{boxit}[1][X]{[#1:\\BODY]}\\NewEnviron{twice}{\\BODY\\ \\BODY}[END]",
    "\\begin{boxit}[Y] body  \\end{boxit} \\begin{boxit}in \\begin{twice}t\\end{twice} x\\end{boxit}",
  );
  assert!(xml.contains("[Y:body] [X:in t tEND x]"), "{xml}");
}

/// xifthen (#588u): the added tests in `\ifthenelse`.
#[test]
fn xifthen_tests() {
  let xml = clean(
    "\\usepackage{xifthen}\\newtest{\\ispos}[1]{\\cnttest{#1}{>}{0}}",
    "\\ifthenelse{\\isempty{}}{E}{N}\\ifthenelse{\\isempty{ x}}{E}{N}\\ifthenelse{\\isin{bc}{abcd}}{I}{-}\
     \\ifthenelse{\\cnttest{2*3}{>=}{6}}{C}{-}\\ifthenelse{\\dimtest{1cm}{<}{1in}}{D}{-}\
     \\ifthenelse{\\isnamedefined{section}\\and\\not\\isundefined{\\section}}{S}{-}\\ifthenelse{\\ispos{5}}{P}{-}",
  );
  assert!(xml.contains("ENICDSP"), "{xml}");
}

/// xstring (#588q): tests, cuts, counts and modes as xstring computes them (pdfLaTeX's results).
#[test]
fn xstring_commands() {
  let xml = clean(
    "\\usepackage{xstring}\\def\\x{abc}\\def\\w{\\x}",
    "[\\IfSubStr{a{bc}d}{bc}{T}{F}\\IfSubStr{a{bc}d}{{bc}}{T}{F}\\expandarg\\IfStrEq{\\w}{abc}{T}{F}\\fullexpandarg\
     \\IfEq{12.34}{12,34}{T}{F}\\IfEndWith{aaa}{aa}{T}{F}]\
     [\\StrBetween[3,2]{abracadabra}{a}{bra}/\\StrDel{United States}{ }/\\StrCount{aaaaaa}{aa}/\\StrPosition[3]{aaaaaa}{aa}\
     /\\exploregroups\\StrLen{ab{cd}e}\\StrSubstitute{a{ab}a}{a}{X}\\noexploregroups/\\IfStrEqCase{hal}{{arxiv}{AR}{hal}{HAL}}[O]\
     /\\IfDecimal{-007.50}{\\integerpart:\\decimalpart}{}/\\StrBefore{abcdef}{cd}[\\r]\\r]",
  );
  assert!(
    xml.contains("[FTFTF][da/UnitedStates/3/5/5XXbX/HAL/-7:50/ab]"),
    "{xml}"
  );
}

/// tensor's `\nuclide[A][Z]{X}` (tensor.sty:45-48, :191-196): mass number over atomic number before
/// the element, either optional.
#[test]
fn tensor_nuclide() {
  let (log, html) = super::perfect_kernel_batch46::convert_html(&doc(
    "\\usepackage{tensor}",
    "$\\nuclide[14][6]{C}$ $\\nuclide[235]{U}$ x",
  ));
  assert_eq!(super::error_count(&log), 0, "{log}");
  assert!(
    html.contains("<mprescripts></mprescripts><mn>6</mn><mn>14</mn></mmultiscripts>"),
    "{html}"
  );
  assert!(
    html.contains("<mprescripts></mprescripts><mphantom></mphantom><mn>235</mn></mmultiscripts>"),
    "{html}"
  );
  assert!(!html.contains("]{C}"), "{html}");
}

/// xstring numbers are TeX numbers (registers, `\value`); explore-mode cuts return the innermost group's
/// part; `\IfDecimal`'s parts on failure; `\tokenize` does not expand (pdfLaTeX's results).
#[test]
fn xstring_numbers_explore_and_side_macros() {
  let xml = clean(
    "\\usepackage{xstring}\\newcounter{i}\\setcounter{i}{2}\\def\\foo{F}",
    "[\\StrChar{abcd}{\\value{i}}/\\exploregroups\\StrLeft{ab{cd}e}{3}/\\StrSplit{ab{cd}e}{3}\\A\\B\\A:\\B/\
     \\StrCut{ab{cd}e}{d}\\C\\D\\C:\\D\\noexploregroups/\\IfDecimal{5.3x}{T}{F}\\integerpart:\\decimalpart:\\afterdecimal/\
     \\IfInteger{5.3x}{T}{F}\\afterinteger/\\IfInteger{}{T}{F}\\afterinteger/\\noexpandarg\\tokenize\\t{\\foo}\\fullexpandarg\\def\\u{\\foo}\\ifx\\t\\u Y\\else N\\fi]",
  );
  assert!(xml.contains("[b/c/c:d/c:/F5:3:x/F.x/FX/Y]"), "{xml}");
}

/// xstring reads a number argument on its own, so an eTeX expression ends with the argument, not
/// in the following text; explore-mode `\StrSplit*` keeps the group the cut falls in, and
/// `\groupID` names the groups entered (pdfLaTeX's results).
#[test]
fn xstring_isolated_numbers_and_explore_cuts() {
  let xml = clean(
    "\\usepackage{xstring}\\def\\ab{a{bc}}\\def\\d{d}\\def\\bc{bc}",
    "[\\StrLeft{abcdef}{\\numexpr 1+1}/after/\\exploregroups\\StrSplit*{a{bc}d}{3}\\A\\B\\ifx\\A\\ab Y\\else N\\fi\\ifx\\B\\d Y\\else N\\fi/\
     \\StrSplit{a{bc}d}{3}\\A\\B\\ifx\\A\\bc Y\\else N\\fi\\ifx\\B\\empty Y\\else N\\fi/\\StrPosition{{ab}x}{x}/\
     \\StrBefore{{a}{b}x}{b}[\\R]\\ifx\\R\\empty Y\\else N\\fi:\\groupID\\noexploregroups]",
  );
  assert!(xml.contains("[ab/after/YY/YY/3/Y:1,2]"), "{xml}");
  // `\IfEq`'s numbers compare as `\ifdim` does (signed); a cut at the last leaf stays in its group,
  // `\StrSplit*` then continuing outside it; a group inside a match is not entered; `\StrMid` and
  // `\StrRight`/`\StrGobbleRight` compute their cut with `\numexpr`; `\IfInteger`/`\IfDecimal` keep the
  // text after the number as it is.
  let xml = clean(
    "\\usepackage{xstring}\\def\\abc{a{bc}}\\def\\bc{bc}\\def\\ax{{a}x{c}}\\edef\\as{\\detokenize{a }}",
    "[\\IfEq{-0.5}{0.5}{T}{F}\\IfEq{-.5}{.5}{T}{F}\\IfEq{-0}{0}{T}{F}/\\exploregroups\\StrLeft{a{bc}}{3}/\
     \\StrSplit{a{bc}}{3}\\A\\B\\ifx\\A\\bc Y\\else N\\fi\\ifx\\B\\empty Y\\else N\\fi/\
     \\StrSplit*{a{bc}}{3}\\A\\B\\ifx\\A\\abc Y\\else N\\fi\\ifx\\B\\empty Y\\else N\\fi/\
     \\StrBefore[2]{{a}x{c}{a}}{{a}}[\\R]\\ifx\\R\\ax Y\\else N\\fi:\\groupID\\noexploregroups/\
     \\StrMid{abcdef}{1+1}{4}/\\StrRight{abcdef}{1+1}/\\StrGobbleRight{abcdef}{1+1}/\
     \\IfInteger{5.X}{}{}\\afterinteger/\\IfDecimal{12a }{}{}\\ifx\\afterdecimal\\as Y\\else N\\fi]",
  );
  assert!(xml.contains("[FFT/bc/YY/YY/Y:1/bcd//abcdef/.X/Y]"), "{xml}");
}

/// manyfoot (#588k): further footnote series with their own counters and marks.
#[test]
fn manyfoot_series_are_notes_of_their_own() {
  let xml = clean(
    "\\usepackage{manyfoot}\\DeclareNewFootnote{A}\\DeclareNewFootnote{B}[roman]\\SetFootnoteHook{H:}\\DeclareNewFootnote{C}[alph]",
    "x\\footnote{s} a\\footnoteA{1} b\\footnoteA{2} c\\footnoteB{3} d\\footnoteC{4} e\\Footnote{*}{5}",
  );
  assert!(xml.contains(r#"<note mark="2" role="footnoteA""#), "{xml}");
  assert!(xml.contains(r#"<note mark="i" role="footnoteB""#), "{xml}");
  assert!(xml.contains("H:4</note>"), "{xml}");
  assert!(xml.contains(r#"<note mark="*" role="footnote""#), "{xml}");
  assert!(xml.contains(r#"<note mark="1" role="footnote" "#), "{xml}");
}

/// vruler (#588p), hypdvips (#588h), abstract (#588y): print-layout packages keep their
/// commands and what they typeset.
#[test]
fn layout_packages_keep_their_commands() {
  let xml = clean(
    "\\usepackage{hyperref}\\usepackage[fullbookmarks]{hypdvips}\\usepackage{vruler,abstract}\
     \\renewcommand{\\abstractnamefont}{\\bfseries}\\setlength{\\absleftindent}{1cm}\
     \\setvruler[12bp][1][1][3][1][1.18\\textwidth][26pt][-7pt][0.99\\textheight]",
    "\\begin{abstract}Abs.\\end{abstract}\\setvruler[][\\rulercount]\\unsetvruler\\fillzeros[4]{12}\\fillzeros[3]{-5} \
     \\attachfile[text={data}]{d.csv} \\file{a/b/c.txt}{x} \\goto[dest=s]{go}\\section{S}\\hypertarget{s}{}",
  );
  assert!(
    xml.contains(r#"<abstract inlist="toc" name="Abstract""#),
    "{xml}"
  );
  assert!(xml.contains(r#"<ref idref="s">go</ref>"#), "{xml}");
  assert!(xml.contains("0012-05"), "{xml}");
  assert!(xml.contains(r#"href="d.csv">data</ref>"#), "{xml}");
  assert!(xml.contains(">c.txt</ref>"), "{xml}");
}

/// epic (#588i): epic's dots are `\@wholewidth` squares, its counts epic's (`\sqrtandstuff`: 6 dots on 20pt at
/// a 4pt gap); a dashed line is a dash pattern.
#[test]
fn epic_draws() {
  let xml = clean(
    "\\usepackage{epic}",
    "\\setlength{\\unitlength}{1pt}\\begin{picture}(100,100)\\dottedline{4}(0,0)(20,0)\\dashline{6}(0,10)(60,10)\
     \\drawline(0,0)(30,40)(60,0)\\end{picture}",
  );
  assert_eq!(xml.matches("<rect ").count(), 6, "{xml}");
  assert_eq!(xml.matches("stroke-dasharray=").count(), 1, "{xml}");
  assert!(xml.contains(r#"points="0,0 41.51,55.35 83.02,0""#), "{xml}");
}

/// eepic (#588t): round dots, splines, ellipses, arcs (clockwise on the page), fills.
#[test]
fn eepic_draws() {
  let xml = clean(
    "\\usepackage{epic,eepic}",
    "\\setlength{\\unitlength}{1pt}\\begin{picture}(100,100)\\dottedline{4}(0,0)(20,0)\\path(10,80)(30,90)\
     \\spline(60,60)(80,90)(100,60)\\put(50,50){\\ellipse{30}{20}}\\put(80,30){\\arc{20}{0}{3.1416}}\
     \\blacken\\put(20,40){\\circle{10}}\\filltype{shade}\\put(40,30){\\circle*{8}}\\end{picture}",
  );
  assert_eq!(
    xml
      .matches(r##"<circle fill="#000000" r="0.38" stroke="none""##)
      .count(),
    6,
    "{xml}"
  );
  assert_eq!(xml.matches("<bezier ").count(), 1, "{xml}");
  assert!(
    xml.contains(r#"<ellipse fill="none" rx="20.76" ry="13.84""#),
    "{xml}"
  );
  assert!(xml.contains(r#"<arc angle1="-180" angle2="0""#), "{xml}");
  assert!(
    xml.contains(r##"<circle fill="#000000" r="6.92" stroke="#000000""##),
    "{xml}"
  );
  assert!(
    xml.contains(r##"<circle fill="#808080" r="5.53" stroke="none""##),
    "{xml}"
  );
}

/// quiver (#588a), tkz-graph (#588x), dynkin-diagrams (#588s): TikZ diagrams without raw files.
#[test]
fn tikz_diagram_packages_draw() {
  let xml = clean(
    "\\usepackage{quiver,tkz-graph,dynkin-diagrams}",
    "\\[\\begin{tikzcd} A & B \\arrow[\"f\", curve={height=12pt}, from=1-1, to=1-2]\\end{tikzcd}\\]\
     \\begin{tikzpicture}\\Vertex[x=0,y=0]{a}\\Vertex[LabelOut,Lpos=90,x=2,y=0,L=$\\alpha$]{b}\\Edge[label=e](a)(b)\\end{tikzpicture}\
     \\dynkin B3 \\dynkin[fold]A{**.**.**} \\dynkin[extended]A3 \\dynkin[edge length=2*.35cm]A4",
  );
  // The tikz-cd diagram, the graph and four Dynkin diagrams (the extended one and the one with a pgfmath
  // edge length by the package's own code).
  assert_eq!(xml.matches("<picture").count(), 6, "{xml}");
  // The arrow's label, the edge's label, the outside vertex label.
  for label in [">f<", ">e<", "α"] {
    assert!(xml.contains(label), "{label}: {xml}");
  }
  // quiver's `curve={height=12pt}`: the arrow is a Bézier (`to path` with `controls`).
  assert!(
    xml.contains(r#"d="M -17.79 -10.07 C -8.59 -17.87 8.18 -17.87 16.96 -10.43""#),
    "{xml}"
  );
  // The folded A6's three grey fold bars (`\dynkin@fold`'s bars, a root radius wide), drawn natively.
  assert_eq!(
    xml
      .matches(r##"stroke="#999999" stroke-linecap="round" stroke-width="1.42pt""##)
      .count(),
    3,
    "{xml}"
  );
  // The folded A6's two indefinite edges, a third of each dotted.
  assert_eq!(
    xml.matches(r#"stroke-dasharray="0.4pt,1.0pt""#).count(),
    2,
    "{xml}"
  );
}

/// The amsmath binding's own `\xlongLeftarrow`/`\xlongRightarrow` (not amsmath.sty's; beside extarrows' arrows)
/// have their stretchy bases.
#[test]
fn amsmath_long_double_arrows_have_bases() {
  let xml = clean(
    "\\usepackage{amsmath}",
    "$a\\xlongLeftarrow{f}b\\xlongRightarrow{g}c$",
  );
  assert!(
    xml.contains(r#"stretchy="true">⇐</XMTok>"#) && xml.contains(r#"stretchy="true">⇒</XMTok>"#),
    "{xml}"
  );
}

/// A binding outranks the raw file under the raw-loading protocol too (`[rawstyles]`): centernot's raw
/// `\mathpalette`/`\rlap` composite does not appear.
#[test]
fn bindings_outrank_raw_files() {
  let (log, xml) = super::convert(
    &doc(
      "\\usepackage{centernot,faktor}",
      "$a\\centernot\\to b$ $\\faktor{G}{H}$",
    ),
    true,
  );
  assert_eq!(super::error_count(&log), 0, "{log}");
  assert!(xml.contains(r#"tex="a\mathrel{\not\to}b""#), "{xml}");
  assert!(!xml.contains("mathchoice"), "{xml}");
  assert!(xml.contains(r#"name="faktor""#), "{xml}");
}

/// A local style the default configuration does not read still has its packages loaded (Perl's
/// dependency scan); its top-level `\let` of a command one of them defines is applied (#589), so
/// `\tensor` after `\usepackage{tensor}\let\tensor\otimes` is the operator, not tensor's command
/// (witness 2610.18718, 598 errors and a runaway before). A `\let` of a command defined before the
/// scan, one in a macro body, or one of a name the file defines again, stays unread.
#[test]
fn an_unread_style_rebinds_the_commands_of_its_packages() {
  let (log, xml) = latexml::util::test::convert_files_with(
    &doc(
      "\\usepackage{mine}",
      "$A\\tensor B$ $\\vec{x}$ \\begin{proof}P\\end{proof}",
    ),
    &[(
      "mine.sty",
      "\\usepackage{tensor,amsthm}\n\\let\\tensor\\otimes\n\\let\\vec\\mathbf\n\\newcommand\\mine{\\let\\tensor\\relax}\n\
       \\let\\proof\\relax\\let\\endproof\\relax\n\\newenvironment{proof}{}{}\n",
    )],
    None,
  );
  assert_eq!(super::error_count(&log), 0, "{log}");
  assert!(log.contains("Loading dependencies for"), "{log}");
  let toks = tokens(&xml);
  // (the `\\let\\tensor\\relax` in `\\mine`'s body would leave no `⊗`).
  assert!(toks.iter().any(|t| t == "⊗"), "{toks:?}");
  // A `\let` clearing a name the file defines again (2610.19354's class: `\let\proof\relax`, then
  // `\newenvironment{proof}`) is not applied: amsthm's `proof` stays (0 errors above).
  assert!(xml.contains("<p>P∎</p>"), "{xml}");
  // `\vec` keeps the kernel's accent.
  assert!(toks.iter().any(|t| t == "→"), "{toks:?}");
}
