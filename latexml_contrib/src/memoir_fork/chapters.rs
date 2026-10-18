//! Chapter heads (`\chapterstyle`), `\appendix` and the front/main/back matter.
//!
//! LaTeXML's sectioning builds a chapter's head itself, so memoir's `\@makechapterhead` (memoir.cls:3092) and the
//! chapter-style hooks it calls (`\printchaptername`, `\chapternamenum`, `\printchapternum`, `\printchaptertitle`)
//! never run: every chapter was headed "Chapter 1 Title" whatever the style, appendices "Chapter A", and the style
//! had no effect at all. Here the style (memoir.cls:3170-3700, `\makechapterstyle`) decides what the tag before the
//! title reads, through the tag hooks `\fnum@chapter` / `\fnum@appendix` (Perl `Base_Utility.pool.ltxml:1041`,
//! `\lx@fnum@@`), and it is carried onto the unit as the class `ltx_chapterstyle_<name>`, which `ltx-memoir.css`
//! lays out for the styles the author's notes use (`ell`, `demo2`).
use latexml_package::prelude::*;

/// What a built-in chapter style prints between the head's start and the title, as the tag: memoir.cls prints
/// `\printchaptername \chapternamenum \printchapternum` (:3099), the name being `\@chapapp` (`Chapter`, `Appendix`
/// after `\appendix`) and the number `\thechapter` or a word.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct HeadTag {
  /// `\printchaptername` prints `\@chapapp` (the default style, `companion`, `thatcher`, `veelo` ...).
  name:   bool,
  number: NumberForm,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum NumberForm {
  /// `\thechapter` (`ell`, `article`, `section`, `komalike` ...).
  Arabic,
  /// `\numtoName{\c@chapter}` whatever the unit (`demo`, :3261).
  Word,
  /// `\ifanappendix \thechapter \else \numtoName{\c@chapter}\fi` (`demo2`, `demo3`, `brotherton`, `dowding`).
  WordUnlessAppendix,
  /// `\printchapternum` is empty (`bringhurst`, :3283-3293).
  Nothing,
}

/// The tag of every `\makechapterstyle` in memoir.cls (:3179-3680), read off its `\printchaptername`,
/// `\chapternamenum` and `\printchapternum`. The look (fonts, rules, alignment, skips) is not a tag: it is the
/// class `ltx_chapterstyle_<name>` and `ltx-memoir.css`. A style the document defines itself with `\makechapterstyle`
/// is not in this table: it keeps the default name and number, as LaTeXML always did, because its hooks are
/// arbitrary TeX that cannot be told apart from their effect.
fn head_tag(style: &str) -> HeadTag {
  use NumberForm::*;
  let (name, number) = match style {
    // :3196-3708 `\printchaptername` and `\chapternamenum` emptied, the number `\thechapter` (`ell` :3390).
    "section" | "article" | "reparticle" | "hangnum" | "ell" | "pedersen" | "southall"
    | "verville" | "crosshead" | "komalike" | "tandh" | "wilsondob" | "culver" | "dash" => {
      (false, Arabic)
    },
    // :3258-3266 `\printchaptername` is just `\centering`; the number is the word, appendices included.
    "demo" => (false, Word),
    // :3340-3355, :3357-3375 `\printchaptername` is `\centering` only.
    "demo2" | "demo3" => (false, WordUnlessAppendix),
    // :3295-3299 name kept, number the word; `dowding` :3580-3590 the same.
    "brotherton" | "dowding" => (true, WordUnlessAppendix),
    "bringhurst" => (false, Nothing),
    // `default` and the styles that change only the look or place the name and number (`companion`, `bianchi`,
    // `chappell`, `ger`, `lyhne`, `madsen`, `thatcher`, `veelo`, `ntglike`).
    _ => (true, Arabic),
  };
  HeadTag { name, number }
}

/// The tokens `\fnum@chapter` / `\fnum@appendix` expand to for the style, as the kernel composes them (`\lx@the@@`
/// is `\the<counter>` of the unit's type; `\lx@refnum@compose` puts the name and a space before it).
fn tag_tokens(style: &str, unit: &str) -> Tokens {
  let tag = head_tag(style);
  let arabic = format!("\\lx@the@@{{{unit}}}");
  let word = "\\numtoName{\\c@chapter}";
  let word_unless_appendix = format!("\\ifanappendix {arabic}\\else {word}\\fi");
  let number = match tag.number {
    NumberForm::Arabic => arabic.clone(),
    NumberForm::Word => word.to_string(),
    NumberForm::WordUnlessAppendix => word_unless_appendix,
    NumberForm::Nothing => String::new(),
  };
  let tex = match (tag.name, tag.number) {
    (_, NumberForm::Nothing) | (false, _) => number,
    // The name and a number that may be a word: not `\lx@refnum@compose`, which tests the number's first token.
    (true, NumberForm::Arabic) => format!("\\lx@refnum@compose{{\\@chapapp}}{{{number}}}"),
    (true, _) => format!("\\@chapapp\\space {number}"),
  };
  TokenizeInternal!(TeXString::assembled(tex))
}

/// The style name as a class-name fragment (`\chapterstyle{my style}` stays a single token of the class).
fn class_fragment(style: &str) -> String {
  style
    .chars()
    .map(|c| {
      if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
        c
      } else {
        '_'
      }
    })
    .collect()
}

/// The style in force now: `\lx@memoir@chapterstyle`, which `\chapterstyle` re-defines in its own group, as the style
/// it runs re-defines its `\printchapternum` and the others.
fn current_style() -> String {
  do_expand(T_CS!("\\lx@memoir@chapterstyle"))
    .map(|t| t.to_string())
    .unwrap_or_else(|_| "default".to_string())
}

LoadDefinitions!({
  RequireResource!("ltx-memoir.css");

  // memoir.cls:3169-3171 `\chapterstyle{<name>}` is `\@nameuse{chs@<name>}`, which runs the definitions the style
  // was made of (`\makechapterstyle{<name>}{...}`, :3167). Those still run, through the class's own macro, so a
  // document that calls `\printchaptername` or another hook (or redefines one) finds it defined; the style's name is
  // recorded after it, since a style begins with `\chapterstyle{default}` (:3187) and the outermost name must win.
  // An undefined name leaves the previous style (and its name) alone, as memoir's `\@nameuse` leaves its macros.
  Let!("\\lx@memoir@raw@chapterstyle", "\\chapterstyle");
  DefMacro!(
    "\\chapterstyle{}",
    r"\lx@memoir@raw@chapterstyle{#1}\@ifundefined{chs@#1}{}{\edef\lx@memoir@chapterstyle{#1}}"
  );
  // The style current at load: memoir.cls:12166 runs `\chapterstyle{article}` for the `article` option (:580-582,
  // `\ifartopt`) before this file exists, and :3181 `\chapterstyle{default}` otherwise.
  RawTeX!(r"\def\lx@memoir@chapterstyle{default}\ifartopt\def\lx@memoir@chapterstyle{article}\fi");

  // The tag of a numbered chapter / appendix (`\lx@fnum@@{chapter}` calls `\csname fnum@chapter\endcsname{}`; the
  // group it passes is inert). Cross-references take `\lx@therefnum@@`/`\lx@typerefnum@@` and keep the plain number.
  DefMacro!("\\fnum@chapter", sub[_args] { Ok(tag_tokens(&current_style(), "chapter")) });
  DefMacro!("\\fnum@appendix", sub[_args] { Ok(tag_tokens(&current_style(), "appendix")) });

  // The class on the unit. The kernel puts `\lx@startsection@afterhook{<type>}{<inlist>}` right after a heading
  // (sect04.rs `after_heading_hook`), inside the unit it opened, read at digestion: so the style it names is the one
  // in force when the heading was set and not the one left at the end of the document.
  DefMacro!("\\lx@startsection@afterhook{}{}", sub[(stype, _inlist)] {
    let stype = stype.to_string();
    if stype == "chapter" || stype == "appendix" {
      Ok(TokenizeInternal!(TeXString::assembled(format!("\\lx@memoir@chapterclass{{{}}}", class_fragment(&current_style())))))
    } else {
      Ok(Tokens!())
    }
  });
  DefConstructor!("\\lx@memoir@chapterclass{}", sub[document, args] {
    let class = format!("ltx_chapterstyle_{}", args[0].as_ref().map(|a| a.to_string()).unwrap_or_default());
    let mut unit = document.get_element();
    while let Some(node) = unit.as_mut()
      && !with(document::get_node_qname(node), |q| matches!(q, "ltx:chapter" | "ltx:appendix"))
    {
      unit = node.get_parent();
    }
    if let Some(mut unit) = unit {
      document.add_class(&mut unit, &class)?;
    }
  });

  // memoir.cls:4295-4302. LaTeXML's appendix semantics are the book binding's (book_cls.rs `\@appendix`): the
  // kernel starts the appendices, so the chapters that follow become `ltx:appendix` units lettered by the chapter
  // counter. memoir's own state follows: counters back to 0, `\@chapapp` the name (`\thechapter` is `\Alph{chapter}`
  // already, set by the kernel for the appendices), and `\ifanappendix`, which the styles and :3061 test.
  DefPrimitive!("\\@appendix", {
    start_appendices("chapter");
  });
  DefMacro!(
    "\\appendix",
    r"\par\@appendix\setcounter{chapter}{0}\setcounter{section}{0}\gdef\@chapapp{\appendixname}\anappendixtrue"
  );

  // memoir.cls:2721-2790 front, main and back matter. memoir has no `no_number_sections` switch (book_cls.rs):
  // the numbers go through `secnumdepth`, which `\frontmatter` and `\backmatter` set to -10 and `\mainmatter` back
  // to `maxsecnumdepth` (the :12127 `\maxsecnumdepth{section}`), and the unit's number is `\if@mainmatter` at :3042.
  // The figure and table counters are per chapter in the main matter only, unless the `article` option (`\ifartopt`).
  DefMacro!("\\frontmatter", r"\@ifstar{\@smemfront}{\@memfront}");
  DefMacro!(
    "\\@memfront@floats",
    r"\counterwithout{figure}{chapter}\counterwithout{table}{chapter}"
  );
  DefMacro!(
    "\\@smemfront",
    r"\cleardoublepage\@mainmatterfalse\setcounter{secnumdepth}{-10}\@memfront@floats"
  );
  DefMacro!("\\@memfront", r"\@smemfront\pagenumbering{roman}");

  DefMacro!("\\mainmatter", r"\@ifstar{\@smemmain}{\@memmain}");
  DefMacro!(
    "\\@memmain@floats",
    r"\counterwithin{figure}{chapter}\counterwithin{table}{chapter}"
  );
  DefMacro!(
    "\\@smemmain",
    r"\@mainmattertrue\setcounter{secnumdepth}{\value{maxsecnumdepth}}\ifartopt\if@twoside\cleardoublepage\else\clearpage\fi\else\cleardoublepage\@memmain@floats\fi"
  );
  DefMacro!("\\@memmain", r"\@smemmain\pagenumbering{arabic}");

  DefMacro!(
    "\\@memback@floats",
    r"\counterwithout{figure}{chapter}\counterwithout{table}{chapter}\setcounter{figure}{0}\setcounter{table}{0}"
  );
  DefMacro!(
    "\\backmatter",
    r"\ifartopt\clearpage\else\if@openright\cleardoublepage\else\clearpage\fi\fi\@mainmatterfalse\setcounter{secnumdepth}{-10}\ifartopt\else\@memback@floats\fi"
  );
});
