//! stackengine.sty (v4.11, 2021/07/22) — stacks of objects over and under an anchor.
//!
//! No Perl binding exists; native binding, OXIDIZED_DESIGN #588(b). The package builds every stack
//! with `\stackengine{gap}{anchor}{item}{O|U}{l|c|r}{quiet}{useanchorwidth}{S|L}`
//! (stackengine.sty:203-255): the anchor sits on the baseline and the item is raised over it (`O`)
//! or lowered under it (`U`) by a gap measured from the boxes (`S`) or baseline to baseline (`L`),
//! the two aligned left, centre or right, each item typeset in `\stack@delim` — `$…$` after
//! `\stackMath`, text after `\stackText` (:181-185). Here the stack is its meaning instead of its
//! kerns: with math items, the item is an over- or under-accent of the anchor (`<mover>`/`<munder>`,
//! as `\overset`/`\underset` are built, but in text style, as the package's fresh `$…$` is); with
//! text items, the two are the rows of an inline block, `\shortstack`'s shape, attached at the
//! anchor's row (the bottom row for `O`, the top for `U`) and aligned as asked. The multi-row stacks
//! (`\Shortstack`, `\Longstack`, the under-stacks, `\Centerstack`, `\Vectorstack`, :263-305, :393-410)
//! split their rows at `\SEP@char` as `\readlist*` does and are the same blocks, or a one-column
//! array of the math items (the fenced `\…Vectorstack`s a fenced column vector). The gaps and
//! `useanchorwidth` only place the boxes. `\stackinset` (:368-391) overlays its inset on the
//! anchor: the two are the rows of an `ltx_stackinset` block, which LaTeXML.css lays over each other,
//! the inset at the top, centre or bottom and left, centre or right of the anchor (the offsets are
//! not applied). `\savestack`, the defaults and the mode switches are the package's own definitions.
//! Witnesses (errors in the default configuration, before → after): 2610.13477 (1 → 0), 2610.20214 (1 → 0), 2610.22059 (`\reallywidetilde`).
use crate::prelude::*;

/// Split `rows` at the separator tokens (top level only), trimming spaces around each row, as
/// `\setsepchar{\SEP@char}\readlist*` does (stackengine.sty:125-129); a space separator drops the
/// empty rows its trimming leaves.
fn split_rows(rows: &Tokens, sep: &[Token]) -> Vec<Vec<Token>> {
  let toks = rows.clone().unlist();
  let space_sep = sep.iter().all(|t| t.get_catcode() == Catcode::SPACE);
  let mut out: Vec<Vec<Token>> = vec![Vec::new()];
  let mut depth = 0usize;
  let mut i = 0;
  while i < toks.len() {
    let t = &toks[i];
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ => {},
    }
    let at_sep = depth == 0
      && !sep.is_empty()
      && if space_sep {
        t.get_catcode() == Catcode::SPACE
      } else {
        toks.len() - i >= sep.len() && toks[i..i + sep.len()] == sep[..]
      };
    if at_sep {
      out.push(Vec::new());
      i += if space_sep { 1 } else { sep.len() };
      continue;
    }
    if let Some(row) = out.last_mut() {
      row.push(*t);
    }
    i += 1;
  }
  let trimmed: Vec<Vec<Token>> = out
    .into_iter()
    .map(|row| {
      let start = row
        .iter()
        .position(|t| t.get_catcode() != Catcode::SPACE)
        .unwrap_or(row.len());
      let end = row
        .iter()
        .rposition(|t| t.get_catcode() != Catcode::SPACE)
        .map_or(start, |e| e + 1);
      row[start..end.max(start)].to_vec()
    })
    .collect();
  if space_sep {
    trimmed.into_iter().filter(|r| !r.is_empty()).collect()
  } else {
    trimmed
  }
}

#[rustfmt::skip]
LoadDefinitions!({
  // stackengine.sty:94-96
  RequirePackage!("etoolbox");
  RequirePackage!("calc");
  // stackengine.sty:95: listofitems (its `\readlist`), which the multi-row stacks here split natively.
  InputDefinitions!("listofitems", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  // stackengine.sty:98-113, :115-124, :137-191: registers, boxes, options and defaults, as the
  // package defines them.
  RawTeX!(r"\newcounter{@stackindex}\newlength\@boxshift\newlength\stack@tmplength\newlength\temp@stkl
\newlength\@stackedboxwidth\newsavebox\@addedbox\newsavebox\@anchorbox\newsavebox\@insetbox
\newsavebox\se@backgroundbox\newsavebox\stackedbox\newsavebox{\@centerbox}
\begingroup\lccode`\|=`\\\lowercase{\endgroup\def\removebs#1{\if#1|\else#1\fi}}
\newcommand{\stack@macro@name}[1]{\expandafter\removebs\string#1}
\newcommand\setstackEOL[1]{\ifstrempty{#1}{\def\SEP@char{ }}{\def\SEP@char{#1}}}
\newif\ifstackengine@oldsyntax\newif\ifstackengine@usestackEOL");
  DeclareOption!("oldsyntax", { RawTeX!(r"\stackengine@oldsyntaxtrue"); });
  DeclareOption!("usestackEOL", { RawTeX!(r"\stackengine@usestackEOLtrue"); });
  ProcessOptions!();
  RawTeX!(r"\ifstackengine@oldsyntax
\newlength\Sstackgap\newlength\Lstackgap
\newcommand\stackgap{\if S\stacktype\the\Sstackgap\else\the\Lstackgap\fi}
\setlength{\Sstackgap}{3pt}\setlength{\Lstackgap}{\normalbaselineskip}
\else
\newcommand{\setstackgap}[2]{\@namedef{#1stackgap}{#2}}
\newcommand\stackgap{\@nameuse{\if S\stacktype S\else L\fi stackgap}\relax}
\setstackgap{S}{3pt}\setstackgap{L}{\normalbaselineskip}
\fi
\ifstackengine@usestackEOL\setstackEOL{\\}\else\setstackEOL{ }\fi
\def\useanchorwidth{F}\def\quietstack{F}\def\stackalignment{c}\def\stacktype{S}
\newcommand\stackMath{\gdef\stack@delim{$}}\newcommand\stackText{\gdef\stack@delim{}}
\newcommand\lstackMath{\def\stack@delim{$}}\newcommand\lstackText{\def\stack@delim{}}
\stackText
\newcommand\strutlongstacks[1]{\def\@strutlongstacks{#1}}\strutlongstacks{F}
\newcommand\strutshortanchors[1]{\def\@usestackstrut{#1}}\strutshortanchors{T}
\newcommand\ensurestackMath[1]{\let\sv@stackmode\stack@delim\stackMath#1\global\let\stack@delim\sv@stackmode}
\def\lx@stack@dollar{$}");

  // The two-item stack in math: the item over (under) the anchor, in text style.
  DefConstructor!("\\lx@stack@over Undigested {}{}",
    "<ltx:XMApp><ltx:XMWrap role='OVERACCENT'>#3</ltx:XMWrap><ltx:XMArg>#2</ltx:XMArg></ltx:XMApp>",
    reversion => "#1");
  DefConstructor!("\\lx@stack@under Undigested {}{}",
    "<ltx:XMApp><ltx:XMWrap role='UNDERACCENT'>#3</ltx:XMWrap><ltx:XMArg>#2</ltx:XMArg></ltx:XMApp>",
    reversion => "#1");
  // Text rows: `\shortstack`'s inline block (latex_constructs/sect13.rs) with the alignment and the
  // row on the baseline given.
  DefConstructor!("\\lx@stack@rows{}{}{}",
    "<ltx:inline-block align='#align' vattach='#vattach' class='#class'><ltx:p>#3</ltx:p></ltx:inline-block>",
    bounded => true,
    sizer => "#3",
    before_digest => {
      Let!("\\\\", "\\@shortstack@cr");
      Let!("\\lx@newline", "\\@shortstack@cr");
      bgroup();
    },
    after_digest => sub[_whatsit] { egroup()?; },
    properties => sub[args] {
      let align = match args[0].as_ref().map(|a| a.to_string()).as_deref() {
        Some("l") => "left", Some("r") => "right", _ => "",
      };
      let attach = args[1].as_ref().map(|a| a.to_string()).unwrap_or_default();
      let (vattach, class) = match attach.as_str() {
        "top" | "bottom" | "middle" => (attach.as_str(), String::new()),
        // `inset-<v>`: an overlay, its inset row placed t/c/b and l/c/r on the anchor (LaTeXML.css).
        other => match other.strip_prefix("inset-") {
          Some(v) => {
            let h = match align { "left" => "l", "right" => "r", _ => "c" };
            ("", format!("ltx_stackinset ltx_stackinset_{v} ltx_stackinset_{h}"))
          },
          None => ("", String::new()),
        },
      };
      Ok(stored_map!("align" => align, "vattach" => vattach.to_string(), "class" => class))
    },
    mode => "restricted_horizontal");

  // `\lx@stack@two{written}{anchor}{item}{O|U}{align}`: math items (`\stack@delim` is `$`) make an
  // accent application, entering math from text as the package's `$…$` items do; text items, the
  // two rows, attached at the anchor's.
  RawTeX!(r"\def\lx@stack@two#1#2#3#4#5{\ifx\stack@delim\lx@stack@dollar\expandafter\@firstoftwo\else\expandafter\@secondoftwo\fi
  {\ensuremath{\if U#4\expandafter\lx@stack@under\else\expandafter\lx@stack@over\fi{#1}{{\textstyle#2}}{{\textstyle#3}}}}
  {\if U#4\lx@stack@rows{#5}{top}{#2\\#3}\else\lx@stack@rows{#5}{bottom}{#3\\#2}\fi}}");

  // stackengine.sty:203-255: the stack, or (quiet `T`) the stack stored in `\stackedbox`.
  RawTeX!(r"\newcommand*\stackengine[8]{\if T#6\sbox{\stackedbox}{\lx@stack@two{\stackengine{#1}{#2}{#3}{#4}{#5}{#6}{#7}{#8}}{#2}{#3}{#4}{#5}}\else\lx@stack@two{\stackengine{#1}{#2}{#3}{#4}{#5}{#6}{#7}{#8}}{#2}{#3}{#4}{#5}\fi}
\newcommand*\stackunder[3][\stackgap]{\stackengine{#1}{#2}{#3}{U}{\stackalignment}{\quietstack}{\useanchorwidth}{\stacktype}}
\newcommand*\stackon[3][\stackgap]{\stackengine{#1}{#2}{#3}{O}{\stackalignment}{\quietstack}{\useanchorwidth}{\stacktype}}
\newcommand*\abovebaseline[2][\stackgap]{\stackengine{#1}{}{#2}{O}{\stackalignment}{\quietstack}{F}{\stacktype}}
\newcommand*\belowbaseline[2][\stackgap]{\stackengine{#1}{}{#2}{U}{\stackalignment}{\quietstack}{F}{\stacktype}}");

  // stackengine.sty:263-305, :393-410: the multi-row stacks.
  // `\lx@stack@many@{separator}{math T|F}{align}{rows}{O|U|C}{fence}`
  DefMacro!("\\lx@stack@many@{}{}{}{}{}{}", sub[(sep, math, align, rows, kind, fence)] {
    let rows = split_rows(&rows, &sep.unlist());
    let math = math.untex().trim() == "T";
    let kind = kind.untex();
    let vattach = match kind.trim() { "O" => "bottom", "U" => "top", _ => "middle" };
    let align = align.untex();
    let align = match align.trim() { "l" | "r" => align.trim().to_string(), _ => "c".to_string() };
    let mut out = Vec::new();
    if math {
      let fence = fence.untex();
      let (open, close) = match fence.trim() {
        "paren" => ("\\left(", "\\right)"), "bracket" => ("\\left[", "\\right]"),
        "brace" => ("\\left\\{", "\\right\\}"), "vert" => ("\\left|", "\\right|"), _ => ("", ""),
      };
      let pos = match vattach { "bottom" => "[b]", "top" => "[t]", _ => "" };
      out.extend(mouth::tokenize_internal(TeXString::assembled(
        format!("\\ensuremath{{{open}\\begin{{array}}{pos}{{{align}}}"))).unlist());
      for (k, row) in rows.into_iter().enumerate() {
        if k > 0 { out.push(T_CS!("\\\\")); }
        out.extend(row);
      }
      out.extend(mouth::tokenize_internal(TeXString::assembled(format!("\\end{{array}}{close}}}"))).unlist());
    } else {
      out.extend(mouth::tokenize_internal(TeXString::assembled(
        format!("\\lx@stack@rows{{{align}}}{{{vattach}}}{{"))).unlist());
      for (k, row) in rows.into_iter().enumerate() {
        if k > 0 { out.push(T_CS!("\\\\")); }
        out.extend(row);
      }
      out.push(T_END!());
    }
    Ok(Tokens::new(out))
  });
  RawTeX!(r"\def\lx@stack@many#1#2#3#4{\ifx\stack@delim\lx@stack@dollar\expandafter\@firstoftwo\else\expandafter\@secondoftwo\fi{\lx@stack@many@@{T}}{\lx@stack@many@@{F}}{#1}{#2}{#3}{#4}}
\def\lx@stack@many@@#1{\expandafter\lx@stack@many@\expandafter{\SEP@char}{#1}}
\newcommand*\Shortstack[2][\stackalignment]{\lx@stack@many{#1}{#2}{O}{}}
\newcommand*\Longstack[2][\stackalignment]{\lx@stack@many{#1}{#2}{O}{}}
\newcommand*\Shortunderstack[2][\stackalignment]{\lx@stack@many{#1}{#2}{U}{}}
\newcommand*\Longunderstack[2][\stackalignment]{\lx@stack@many{#1}{#2}{U}{}}
\newcommand\Centerstack[2][\stackalignment]{\lx@stack@many{#1}{#2}{C}{}}
\newcommand\Vectorstack[2][\stackalignment]{\ensurestackMath{\lx@stack@many{#1}{#2}{C}{}}}
\newcommand\parenVectorstack[2][\stackalignment]{\ensurestackMath{\lx@stack@many{#1}{#2}{C}{paren}}}
\newcommand\bracketVectorstack[2][\stackalignment]{\ensurestackMath{\lx@stack@many{#1}{#2}{C}{bracket}}}
\newcommand\braceVectorstack[2][\stackalignment]{\ensurestackMath{\lx@stack@many{#1}{#2}{C}{brace}}}
\newcommand\vertVectorstack[2][\stackalignment]{\ensurestackMath{\lx@stack@many{#1}{#2}{C}{vert}}}");

  // stackengine.sty:328-342 `\stackanchor[gap]{top}{bottom}`: two rows centred on the baseline.
  RawTeX!(r"\newcommand*\stackanchor[3][\stackgap]{\if T\quietstack\sbox{\stackedbox}{\lx@stack@pair{#2}{#3}}\else\lx@stack@pair{#2}{#3}\fi}
\def\lx@stack@pair#1#2{\ifx\stack@delim\lx@stack@dollar\ensuremath{\begin{array}{\stackalignment}#1\\#2\end{array}}\else\lx@stack@rows{\stackalignment}{middle}{#1\\#2}\fi}");
  // stackengine.sty:308-326: the laps are zero-width items a baseline (`L` gap) above or below;
  // `\hsmash` a zero-width centred item.
  RawTeX!(r"\newcommand*\@stacklap[4]{\makebox[0pt][\if l#2r\else\if r#2l\else c\fi\fi]{\raisebox{\if U#4-\fi#1}{\stack@delim#3\stack@delim}}}
\newcommand*\tllap[2][\Lstackgap]{\@stacklap{#1}{l}{#2}{O}}\newcommand*\tclap[2][\Lstackgap]{\@stacklap{#1}{c}{#2}{O}}
\newcommand*\trlap[2][\Lstackgap]{\@stacklap{#1}{r}{#2}{O}}\newcommand*\bllap[2][\Lstackgap]{\@stacklap{#1}{l}{#2}{U}}
\newcommand*\bclap[2][\Lstackgap]{\@stacklap{#1}{c}{#2}{U}}\newcommand*\brlap[2][\Lstackgap]{\@stacklap{#1}{r}{#2}{U}}
\newcommand*\toplap[3][\Lstackgap]{\@stacklap{#1}{#2}{#3}{O}}\newcommand*\bottomlap[3][\Lstackgap]{\@stacklap{#1}{#2}{#3}{U}}
\newcommand*\hsmash[1]{\makebox[0pt]{\stack@delim#1\stack@delim}}");
  // stackengine.sty:360-361: `\addstackgap` pads its content vertically only.
  RawTeX!(r"\newcommand*\addstackgap[2][\Sstackgap]{\stack@delim#2\stack@delim}");
  // stackengine.sty:350-358, as the package defines it.
  RawTeX!(r"\newcommand*\savestack[2]{\xdef\sv@name{\stack@macro@name{#1}}\@ifundefined{\sv@name content}{\expandafter\newsavebox\expandafter{\csname\sv@name content\endcsname}}{}\expandafter\sbox\csname\sv@name content\endcsname{#2}\expandafter\def\expandafter#1\expandafter{\expandafter\usebox\expandafter{\csname\sv@name content\endcsname}}}");
  // stackengine.sty:363-391: the overlay, kept as two rows (inset over anchor).
  RawTeX!(r"\newcommand*\stackinset[6]{\lx@stack@rows{#1}{inset-#3}{\stack@delim#5\stack@delim\\\stack@delim#6\stack@delim}}
\newcommand*\topinset[4]{\stackinset{\stackalignment}{#4}{t}{#3}{#1}{#2}}
\newcommand*\bottominset[4]{\stackinset{\stackalignment}{#4}{b}{#3}{#1}{#2}}");
});
