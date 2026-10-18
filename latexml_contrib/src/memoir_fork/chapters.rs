//! Chapter heads (`\chapterstyle`), `\appendix` and the front/main/back matter.
//!
//! LaTeXML's sectioning builds a chapter's head itself, so memoir's `\@makechapterhead` (memoir.cls:3092) and the
//! chapter-style hooks it calls (`\printchaptername`, `\chapternamenum`, `\printchapternum`, `\printchaptertitle`)
//! never run: every chapter was headed "Chapter 1 Title" whatever the style, appendices "Chapter A", and the style
//! had no effect at all. Here the style (memoir.cls:3176-3640, `\makechapterstyle`) decides what the tag before the
//! title reads, through the tag hooks `\fnum@chapter` / `\fnum@appendix` (Perl `Base_Utility.pool.ltxml:1041`,
//! `\lx@fnum@@`), and it is carried onto the unit as the class `ltx_chapterstyle_<name>`, which `ltx-memoir.css`
//! lays out for the styles the author's notes use (`ell`, `demo2`).
use latexml_package::prelude::*;

/// What a built-in chapter style prints between the head's start and the title, as the tag: memoir.cls prints
/// `\printchaptername \chapternamenum \printchapternum` (:3098), the name being `\@chapapp` (`Chapter`, `Appendix`
/// after `\appendix`) and the number `\thechapter` or a word.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct HeadTag {
  /// `\printchaptername` prints `\@chapapp` (the default style, `companion`, `thatcher`, `veelo` ...).
  name:   bool,
  number: NumberForm,
  /// The punctuation `\printchapternum` sets around the number.
  around: Around,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum NumberForm {
  /// `\thechapter` (`ell`, `article`, `section`, `komalike` ... and `culver`, whose style renews `\thechapter` as
  /// `\Roman{chapter}`, :3316).
  Arabic,
  /// `\numtoName{\c@chapter}` whatever the unit (`demo`, :3261).
  Word,
  /// `\ifanappendix \thechapter \else \numtoName{\c@chapter}\fi` (`demo2`, `demo3`, `brotherton`, `dowding`).
  WordUnlessAppendix,
  /// `\printchapternum` is empty (`bringhurst`, :3283-3293).
  Nothing,
}

/// The characters a style prints next to the number, which the tag can carry (the rules, skips and fonts it sets
/// cannot be).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Around {
  Nothing,
  /// `\thechapter.\ ` (`verville`, :3555): "1. Title".
  Period,
  /// `\rule{1em}{0.4pt} \thechapter\ \rule{1em}{0.4pt}` (`dash`, :3331-3332): the two rules read as dashes, "- 1 -".
  Dashes,
}

/// The tag of every `\makechapterstyle` in memoir.cls (:3179-3640), read off its `\printchaptername`,
/// `\chapternamenum` and `\printchapternum`. The look (fonts, rules, alignment, skips) is not a tag: it is the
/// class `ltx_chapterstyle_<name>` and `ltx-memoir.css`. A style the document defines itself with `\makechapterstyle`
/// is not in this table: it keeps the default name and number, as LaTeXML always did, because its hooks are
/// arbitrary TeX that cannot be told apart from their effect.
///
/// Known residual, from the characters a tag cannot restyle: a style that changes the case of the name or of the title
/// (`thatcher` sets "chapter 1" in lowercase small capitals and the title in capitals, `veelo` the name in capitals,
/// `bringhurst` the title in lowercase small capitals) keeps the case the author wrote; and `pedersen`, `lyhne`,
/// `madsen`, `veelo` put the number in the margin or scale it with `\resizebox`, which is the look and not the tag.
fn head_tag(style: &str) -> HeadTag {
  use NumberForm::*;
  let (name, number, around) = match style {
    // `\printchaptername` emptied (and `\chapternamenum`, which is a `\space` after `\@chs@def@ult` otherwise, :3119,
    // for `verville` and `crosshead`), the number `\thechapter` (`section` :3196, `ell` :3390).
    "section" | "article" | "reparticle" | "hangnum" | "ell" | "pedersen" | "southall"
    | "crosshead" | "komalike" | "tandh" | "wilsondob" | "culver" => {
      (false, Arabic, Around::Nothing)
    },
    // `verville` (:3549): the name emptied, `\thechapter.\ ` (:3555).
    "verville" => (false, Arabic, Around::Period),
    // `dash` (:3323): the name emptied, a rule, `\thechapter\ ` and a rule (:3331-3332).
    "dash" => (false, Arabic, Around::Dashes),
    // `demo` (:3258): `\printchaptername` is just `\centering`; the number is the word, appendices included (:3261).
    "demo" => (false, Word, Around::Nothing),
    // `demo2` (:3340), `demo3` (:3357): `\printchaptername` is `\centering` only, :3344 and :3362 the number.
    "demo2" | "demo3" => (false, WordUnlessAppendix, Around::Nothing),
    // :3295-3298 name kept, number the word; `dowding` (:3580, :3586-3587) the same.
    "brotherton" | "dowding" => (true, WordUnlessAppendix, Around::Nothing),
    "bringhurst" => (false, Nothing, Around::Nothing),
    // `default` and the styles that change only the look or place the name and number (`companion`, `bianchi`,
    // `chappell`, `ger`, `lyhne`, `madsen`, `thatcher`, `veelo`, `ntglike`).
    _ => (true, Arabic, Around::Nothing),
  };
  HeadTag { name, number, around }
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
  let number = match tag.around {
    Around::Nothing => number,
    Around::Period => format!("{number}."),
    Around::Dashes => format!("\\textendash\\space {number}\\space\\textendash"),
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
  // memoir.cls:3177 `\chapterstyle{<name>}` is `\@nameuse{chs@<name>}`, which runs the definitions the style was
  // made of (`\makechapterstyle{<name>}{...}`, :3176, which prepends the defaults of `\@chs@def@ult`, :3116). Those
  // still run, through the class's own macro, so a document that calls `\printchaptername` or another hook (or
  // redefines one) finds it defined; the style's name is recorded after it, since most styles begin with
  // `\chapterstyle{default}` (:3194 is the default itself) and the outermost name must win.
  // An undefined name leaves the previous style (and its name) alone, as memoir's `\@nameuse` leaves its macros.
  Let!("\\lx@memoir@raw@chapterstyle", "\\chapterstyle");
  DefMacro!(
    "\\chapterstyle{}",
    r"\lx@memoir@raw@chapterstyle{#1}\@ifundefined{chs@#1}{}{\edef\lx@memoir@chapterstyle{#1}}"
  );
  // The style current at load: memoir.cls:12167 runs `\chapterstyle{article}` for the `article` option (:580-582,
  // `\ifartopt`) before this file exists, and :3194 `\chapterstyle{default}` otherwise.
  RawTeX!(r"\def\lx@memoir@chapterstyle{default}\ifartopt\def\lx@memoir@chapterstyle{article}\fi");

  // The tag of a numbered chapter / appendix (`\lx@fnum@@{chapter}` calls `\csname fnum@chapter\endcsname{}`; the
  // group it passes is inert). Cross-references take `\lx@therefnum@@`/`\lx@typerefnum@@` and keep the plain number.
  DefMacro!("\\fnum@chapter", sub[_args] { Ok(tag_tokens(&current_style(), "chapter")) });
  DefMacro!("\\fnum@appendix", sub[_args] { Ok(tag_tokens(&current_style(), "appendix")) });

  // The class on the unit. The kernel puts `\lx@startsection@afterhook{<type>}{<inlist>}` right after a heading
  // (sect04.rs `after_heading_hook`), inside the unit it opened, read at digestion: so the style it names is the one
  // in force when the heading was set and not the one left at the end of the document. A hook another binding has
  // defined before this one (svmult's `\motto`, a class's chapter author) is kept and called after the class.
  let previous_hook = has_meaning(&T_CS!("\\lx@startsection@afterhook"));
  if previous_hook {
    Let!(
      "\\lx@memoir@previous@afterhook",
      "\\lx@startsection@afterhook"
    );
  }
  DefMacro!("\\lx@startsection@afterhook{}{}", sub[(stype, inlist)] {
    let stype = stype.to_string();
    let inlist = inlist.to_string();
    let mut tex = String::new();
    if stype == "chapter" || stype == "appendix" {
      tex.push_str(&format!("\\lx@memoir@chapterclass{{{}}}", class_fragment(&current_style())));
    }
    if previous_hook {
      tex.push_str(&format!("\\lx@memoir@previous@afterhook{{{stype}}}{{{inlist}}}"));
    }
    Ok(TokenizeInternal!(TeXString::assembled(tex)))
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

  // memoir.cls:2723 `\frontmatter`, :2738 `\mainmatter`, :2766 `\backmatter`. memoir has no `no_number_sections`
  // switch (book_cls.rs): the numbers go through `secnumdepth`, which `\frontmatter` and `\backmatter` set to -10 and
  // `\mainmatter` back to `maxsecnumdepth` (the :12128 `\maxsecnumdepth{section}`), and the unit's number is
  // `\if@mainmatter` at :3034.
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
  // A chapter after `\backmatter` is a chapter again, not an appendix (:3071 writes its ToC entry as
  // `chapter`; the `appendix` kind is the numbered chapter under `\ifanappendix`, :3061-3063): the kernel's appendices end, as
  // `appendix.sty` ends them (`end_appendices`).
  DefPrimitive!("\\lx@memoir@end@appendices", {
    end_appendices();
  });
  DefMacro!(
    "\\backmatter",
    r"\lx@memoir@end@appendices\ifartopt\clearpage\else\if@openright\cleardoublepage\else\clearpage\fi\fi\@mainmatterfalse\setcounter{secnumdepth}{-10}\ifartopt\else\@memback@floats\fi"
  );
});
