//! nccmath.sty (ncctools v1.3, 2006/01/20) — medium-size math, display-style arrays and amsmath
//! extensions.
//!
//! No Perl binding exists; native binding, OXIDIZED_DESIGN #588(w). nccmath requires amsmath
//! (nccmath.sty:31) and adds: `\medmath{x}`, `x` as an ordinary atom in display style at a medium size
//! (80% of the font size, 83% above 11.5pt, rounded to whole points, :60-88, :167-170), with `\medop`,
//! `\medint`, `\mfrac`, `\mbinom` and the `medsize`/`mmatrix` environments built on it (:171-183);
//! `\nr` (a row break, :40-54), `\mrel` (stacked relations, :55-57), `\underrel` (:58);
//! `darray`/`\dmulticolumn` (an array of display-style cells, :289-311); the `fleqn`/`ceqn` toggles
//! (:365-374); the shorthands `\eq`, `\eqalign`, `\eqs` (:375-388); `\intertext[skip]` and
//! `\useshortskip` (:246-267); an `eqnarray` whose middle column is in display style and which takes
//! amsmath's `\tag` and `\notag` (:389-427); and under `mediummath` medium `\frac`, `\binom` and big
//! operators in display and text style (:429-471). Here the medium size is the font size the math
//! carries (`mathsize`), the display style its math style, and the arrays, tags and environments are
//! the converter's own. The skips (`\intertext`'s, `\nr`'s extra 0.5ex, `\useshortskip`, the
//! `\NCC@ignorepar` paragraph handling, `\medintcorr`) only space the displays. A local `fleqn` or
//! `ceqn` group changes the indentation of the displays in it; the converter aligns displays per
//! document (`ltx_fleqn`, fleqn_sty.rs), so the group keeps its flag only.
use crate::prelude::*;

/// Prefix every `l`/`c`/`r` column of an array preamble with `>{\displaystyle}` (nccmath.sty:289-303:
/// all `darray` cells are display style), copying `@{…}`/`!{…}` groups and the counts of `*{n}{…}`.
fn display_columns(spec: &[Token]) -> Vec<Token> {
  let mut out = Vec::new();
  let mut i = 0;
  let group_end = |toks: &[Token], start: usize| -> usize {
    let mut depth = 0usize;
    let mut j = start;
    while j < toks.len() {
      match toks[j].get_catcode() {
        Catcode::BEGIN => depth += 1,
        Catcode::END => {
          depth = depth.saturating_sub(1);
          if depth == 0 {
            return j;
          }
        },
        _ => {},
      }
      j += 1;
    }
    toks.len().saturating_sub(1)
  };
  while i < spec.len() {
    let t = &spec[i];
    let text = t.with_str(|s| s.to_string());
    match (t.get_catcode(), text.as_str()) {
      (Catcode::OTHER, "@" | "!") | (Catcode::LETTER, "p" | "m" | "b") => {
        out.push(*t);
        if i + 1 < spec.len() && spec[i + 1].get_catcode() == Catcode::BEGIN {
          let end = group_end(spec, i + 1);
          out.extend_from_slice(&spec[i + 1..=end]);
          i = end;
        }
      },
      (Catcode::OTHER, "*") => {
        out.push(*t);
        if i + 1 < spec.len() && spec[i + 1].get_catcode() == Catcode::BEGIN {
          let end = group_end(spec, i + 1);
          out.extend_from_slice(&spec[i + 1..=end]);
          i = end;
          if i + 1 < spec.len() && spec[i + 1].get_catcode() == Catcode::BEGIN {
            let end = group_end(spec, i + 1);
            out.push(T_BEGIN!());
            out.extend(display_columns(&spec[i + 2..end]));
            out.push(T_END!());
            i = end;
          }
        }
      },
      (Catcode::LETTER, "l" | "c" | "r") => {
        out.extend(mouth::tokenize_internal(r">{\displaystyle}").unlist());
        out.push(*t);
      },
      _ => out.push(*t),
    }
    i += 1;
  }
  out
}

#[rustfmt::skip]
LoadDefinitions!({
  // nccmath.sty:26-31: `mediummath`, the other options amsmath's.
  RawTeX!(r"\newif\ifNCC@mediummath");
  DeclareOption!("mediummath", { RawTeX!(r"\NCC@mediummathtrue"); });
  DeclareOption!(None, { RawTeX!(r"\PassOptionsToPackage{\CurrentOption}{amsmath}"); });
  ProcessOptions!();
  RequirePackage!("amsmath");
  // nccmath builds the `darray` preamble itself (nccmath.sty:289-303); here it is an array preamble
  // with `>{\displaystyle}` columns, array.sty's syntax.
  RequirePackage!("array");

  // nccmath.sty:40-58
  RawTeX!(r"\def\nr{\\}
\newcommand{\mrel}[1]{\mathrel{\begin{matrix}#1\end{matrix}}}
\newcommand{\underrel}[2]{\mathrel{\mathop{#1}\limits_{#2}}}");

  // nccmath.sty:60-88, :167-170: the medium size, display style.
  DefConstructor!("\\medmath{}", "#1",
    bounded => true, require_math => true,
    font => sub[_whatsit] {
      let size = lookup_font().and_then(|f| f.get_size()).unwrap_or(10.0);
      let ratio = if size > 11.5 { 0.83 } else { 0.8 };
      let medium = (size * ratio + 0.5).floor().max(5.0);
      Ok(Font { size: Some(medium), mathstyle: Some("display".into()), ..Font::default() })
    });
  DefEnvironment!("{medsize}", "#body",
    require_math => true,
    font => sub[_whatsit] {
      let size = lookup_font().and_then(|f| f.get_size()).unwrap_or(10.0);
      let ratio = if size > 11.5 { 0.83 } else { 0.8 };
      let medium = (size * ratio + 0.5).floor().max(5.0);
      Ok(Font { size: Some(medium), mathstyle: Some("text".into()), ..Font::default() })
    });
  // nccmath.sty:171-183
  RawTeX!(r"\newcommand{\medop}[1]{\DOTSB\mathop{\medmath{#1}}\slimits@}
\newcommand{\medintcorr}[1]{}
\newcommand{\medint}[1]{\DOTSI\mathop{\medmath{#1}}\ilimits@}
\newcommand{\mfrac}[2]{\medmath{\frac{#1}{#2}}}
\newcommand{\mbinom}[2]{\medmath{\binom{#1}{#2}}}
\newenvironment{mmatrix}{\begin{medsize}\begin{matrix}}{\end{matrix}\end{medsize}}");

  // nccmath.sty:246-267: amsmath's `\intertext` with an optional skip; `\useshortskip`.
  DefMacro!("\\@ams@intertext[]{}", "\\lx@hidden@crcr\\noalign{\\@@ams@intertext{#2}}");
  DefConstructor!("\\intertext[]{}", "<ltx:p class='ltx_intertext'>#2</ltx:p>",
    mode => "internal_vertical");
  DefMacro!("\\useshortskip", None);

  // nccmath.sty:289-311
  DefMacro!("\\lx@ncc@darray[]{}", sub[(pos, spec)] {
    let mut out = mouth::tokenize_internal(r"\begin{array}").unlist();
    let pos = pos.map(|p| p.unlist()).unwrap_or_default();
    if !pos.is_empty() {
      out.push(T_OTHER!("["));
      out.extend(pos);
      out.push(T_OTHER!("]"));
    }
    out.push(T_BEGIN!());
    out.extend(display_columns(&spec.unlist()));
    out.push(T_END!());
    Ok(Tokens::new(out))
  });
  RawTeX!(r"\newenvironment{darray}{\lx@ncc@darray}{\end{array}}
\newcommand{\dmulticolumn}[3]{\multicolumn{#1}{#2}{\displaystyle #3}}");

  // nccmath.sty:365-374
  RawTeX!(r"\@ifundefined{if@fleqn}{\newif\if@fleqn}{}
\@ifundefined{@mathmargin}{\newskip\@mathmargin}{}
\newenvironment{fleqn}[1][0pt]{\@fleqntrue\setlength\@mathmargin{#1}}{}
\newenvironment{ceqn}{\@fleqnfalse}{}");

  // nccmath.sty:375-388
  RawTeX!(r"\newcommand{\eq}{\@ifstar{\lx@ncc@eq{equation*}}{\lx@ncc@eq{equation}}}
\long\def\lx@ncc@eq#1#2{\begin{#1}#2\end{#1}}
\def\eqalign{\@ifstar{\lx@ncc@eqalign{equation*}}{\lx@ncc@eqalign{equation}}}
\def\lx@ncc@eqalign#1{\@ifnextchar[{\lx@ncc@eqalign@{#1}}{\lx@ncc@eqalign@{#1}[rcl]}}
\long\def\lx@ncc@eqalign@#1[#2]#3{\begin{#1}\begin{darray}{#2}#3\end{darray}\end{#1}}
\newcommand{\eqs}{\@ifstar{\lx@ncc@eqs{eqnarray*}}{\lx@ncc@eqs{eqnarray}}}
\def\lx@ncc@eqs#1{\@ifnextchar[{\lx@ncc@eqs@{#1}}{\lx@ncc@eqs@{#1}[]}}
\long\def\lx@ncc@eqs@#1[#2]#3{\begin{#1}#3\end{#1}}");

  // nccmath.sty:429-471 `mediummath`: medium fractions, binomials and big operators in display and
  // text style; the script styles keep the plain ones.
  RawTeX!(r"\ifNCC@mediummath
\let\NCC@frac\frac\let\NCC@binom\binom
\renewcommand{\frac}[2]{\mathchoice{\mfrac{#1}{#2}}{\mfrac{#1}{#2}}{\NCC@frac{#1}{#2}}{\NCC@frac{#1}{#2}}}
\renewcommand{\binom}[2]{\mathchoice{\mbinom{#1}{#2}}{\mbinom{#1}{#2}}{\NCC@binom{#1}{#2}}{\NCC@binom{#1}{#2}}}
\def\lx@ncc@medop#1{\expandafter\let\csname NCC@#1\expandafter\endcsname\csname #1\endcsname
  \expandafter\edef\csname #1\endcsname{\noexpand\medop{\expandafter\noexpand\csname NCC@#1\endcsname}}}
\@for\lx@ncc@op:=sum,prod,coprod,bigcap,bigcup,bigvee,bigwedge,biguplus,bigotimes,bigoplus,bigodot,bigsqcup\do{\expandafter\lx@ncc@medop\expandafter{\lx@ncc@op}}
\def\lx@ncc@medint#1{\expandafter\let\csname NCC@#1\expandafter\endcsname\csname #1\endcsname
  \expandafter\edef\csname #1\endcsname{\noexpand\medint{\expandafter\noexpand\csname NCC@#1\endcsname}}}
\@for\lx@ncc@op:=int,oint\do{\expandafter\lx@ncc@medint\expandafter{\lx@ncc@op}}
\fi");
});
