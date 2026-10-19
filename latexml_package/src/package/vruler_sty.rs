//! vruler.sty (v2.3, 1996, Z. Jiang) — vertical line-number rulers in the page margin.
//!
//! No Perl binding exists; native binding, OXIDIZED_DESIGN #588(p). `\setvruler[scale][init][step]
//! [digits][mode][odd shift][even shift][vshift][height]` (vruler.sty:139-180, its nine optional
//! arguments defaulted through `\setdefault`, :345-350) patches the output routine (`\set@vruler`
//! parses `\@outputpage` and wraps it, :146-173) so that every page shipped out carries a box of
//! numbers in its margin (`\SET`, `\makevruler`, :119-138); `\unsetvruler` restores the output
//! routine (:175-176). The ruler numbers the page's height, not the source's lines, and exists only
//! at shipout: an HTML conversion has no pages and no output routine, so the rulers are not drawn,
//! and `\setvruler` reads its arguments and changes nothing (the raw patch fails on this engine's
//! `\@outputpage`). The rest is the package's: the registers, `\rulercount` (the number the next
//! ruler starts from, set when a ruler is built), `\fillzeros[digits]{n}` (:96-104, real text when
//! typeset), `\makevruler`, the macro stack `\push`/`\pop`/`\popnil` (:221-236) and `\setdefault`,
//! which wraps a command so that each of its first `n` bracket arguments, empty or missing, takes a
//! default (:250-350).
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // vruler.sty:76-78: a second load is refused.
  if is_defined_token(&T_CS!("\\VrulerDefined")) {
    RawTeX!(r"\message{NOT reloaded}");
    return Ok(());
  }
  // vruler.sty:79-117
  RawTeX!(r"\xdef\VrulerDefined{}
\newdimen\temp@dim\newdimen\b@xheight\newdimen\lineh@ight
\newbox\outputb@x\newbox\bufferbox\newbox\tempb@x
\newcount\refn@\newcount\tot@l\newcount\rulercount
\newcount\temp@@\newcount\temp@
\def\fillzeros[#1]#2{\temp@@=#2\relax\ifnum\temp@@<0\temp@@=-\temp@@\fi
\temp@=1 %
\loop\ifnum\temp@@<10 \else\divide\temp@@ by 10 \advance\temp@ by 1 \fi
\ifnum\temp@@=10\relax\temp@@=11\relax\fi\ifnum\temp@@>10 \repeat
\ifnum#2<0\advance\temp@1\relax-\fi
\loop\ifnum\temp@<#1\relax0\advance\temp@1\relax\fi\ifnum\temp@<#1 \repeat
\temp@@=#2\relax\ifnum\temp@@<0\temp@@=-\temp@@\fi\relax\the\temp@@}
\newtoks\toksone\newtoks\tokstwo\newtoks\toksthree\newtoks\toksfour\newtoks\toksfive
\newif\ifRuler@Started\Ruler@Startedfalse");
  // vruler.sty:119-131 `\makevruler`: the ruler box is a page-margin object; building it sets the
  // count it starts from, as the package's loop does (`\global\rulercount=#2`).
  RawTeX!(r"\def\makevruler[#1][#2][#3][#4][#5]{\global\rulercount=#2\relax}");
  // vruler.sty:146-173, :175-176: the output-routine patch and its removal.
  RawTeX!(r"\def\set@vruler[#1][#2][#3][#4][#5][#6][#7][#8][#9]{}
\def\unsetvruler{\Ruler@Startedfalse}
\let\setvruler\set@vruler");
  // vruler.sty:212-236: the macro stack.
  RawTeX!(r"\newcount\STKcount\global\STKcount=0\relax
\long\def\push#1{\global\advance\STKcount1\relax\expandafter\gdef\csname STK\the\STKcount\string~\endcsname{#1}}
\def\popnil{\expandafter\let\expandafter\temp@macro\csname STK\the\STKcount\string~\endcsname
\ifnum\STKcount>0\global\expandafter\let\csname STK\the\STKcount\string~\endcsname=\undefined@
\global\advance\STKcount-1\else\def\temp@macro{}\global\STKcount=0\fi\relax}
\def\pop{\popnil\temp@macro}");

  // vruler.sty:250-350 `\setdefault{cs}{n}{d1}…{dn}`: `\cs` reads up to `n` bracket arguments
  // (`\futurelet`, so no space is skipped before a `[`); a missing or empty one is its default; the
  // command saved as `\<cs>@ raw` (once) is called with all `n`.
  DefPrimitive!("\\setdefault{}{Number}", sub[(name, count)] {
    let name = name.untex().trim().to_string();
    let n = count.value_of().clamp(0, 9) as usize;
    let mut defaults = Vec::with_capacity(n);
    for _ in 0..n {
      defaults.push(read_arg(ExpansionLevel::Off)?.unlist());
    }
    let cs = T_CS!(&format!("\\{name}"));
    let raw = T_CS!(&format!("\\{name}@ raw"));
    if !is_defined_token(&raw)
      && let Some(meaning) = lookup_meaning(&cs) {
        assign_meaning(&raw, meaning, None);
      }
    def_macro(cs, None, ExpansionBody::Closure(Rc::new(move |_args| {
      let mut out = vec![raw];
      let mut reading = true;
      for default in &defaults {
        let given = if reading && if_next(T_OTHER!("["))? {
          read_token()?;
          read_until(&Tokens!(T_OTHER!("]")))?.unwrap_or_default().unlist()
        } else {
          reading = false;
          Vec::new()
        };
        out.push(T_OTHER!("["));
        out.extend(if given.iter().all(|t| t.get_catcode() == Catcode::SPACE) { default.clone() } else { given });
        out.push(T_OTHER!("]"));
      }
      Ok(Tokens::new(out))
    })), None)?;
  });
  // vruler.sty:351
  RawTeX!(r"\setdefault{setvruler}{9}{12pt}{1}{1}{4}{1}{0pt}{0pt}{0pt}{\textheight}");
});
