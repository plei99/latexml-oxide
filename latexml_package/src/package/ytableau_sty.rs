//! ytableau.sty — Young tableaux and diagrams, loaded raw; only the box of a cell is bound.
//!
//! ytableau draws each box (`\fcolorbox@YT`, ytableau.sty:272-296) as LaTeX's `\@frameb@x` does: a `\vbox` with the
//! content, then two `\vrule`s and two `\hrule`s placed over it with negative kerns. HTML has no such overlap, so the
//! rules came out beside the content. A box is its cell's border instead, as LaTeXML renders `\fbox` with a frame:
//! top and bottom unless ytableau's horizontal rules are off (`\hrule@none@YT`), left and right unless its vertical
//! ones are (`\vrule@none@YT`, `\ytableausetup{nobaseline}`'s skew fill), and the content in `\colorbox` for a filled
//! box (`*(red)`). `\none` cells (`\omit\none@YT`) get no box, as in the PDF. KNOWN_PERL_ERRORS #661.
use crate::prelude::*;

LoadDefinitions!({
  InputDefinitions!("ytableau", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  // The current cell's border sides (`l`, `r`, `t`, `b`), as a `|` column or an `\hline` would add them.
  DefPrimitive!("\\lx@ytableau@border{}", sub[(sides)] {
    let sides = sides.to_string();
    if let Some(alignment) = lookup_alignment()
      && let Some(data) = alignment.alignment_cell()
      && let Some(cell) = data.borrow_mut().current_column()
    {
      cell.border.push_str(&sides);
    }
    Ok(())
  });

  // A box is `\boxdim@YT` square in TeX (`\startbox@@YT`'s `\hbox to \boxdim@YT`, `\endbox@YT`'s `\vbox to`); an empty
  // cell (`\ydiagram`) has nothing else to give it a size in HTML. The entry is centred (`\hss` both sides, `\vss`
  // above and below), and the box keeps its size in math text, whose line height would stretch it.
  DefConstructor!(
    "\\lx@ytableau@box Dimension {}",
    "<ltx:inline-block width='#1' height='#1' \
      cssstyle='display:inline-flex;align-items:center;justify-content:center;text-align:center;vertical-align:top;overflow:hidden'>#2</ltx:inline-block>"
  );

  RawTeX!(
    r"\def\fcolorbox@YT#1#2{%
    \ifx\hrule@YT\hrule@none@YT\else\lx@ytableau@border{tb}\fi
    \ifx\vrule@YT\vrule@none@YT\else\lx@ytableau@border{lr}\fi
    \compare@@YT{#1}{clear}%
    \ifeq@YT\lx@ytableau@box\boxdim@YT{#2}\else\colorbox{#1}{\lx@ytableau@box\boxdim@YT{#2}}\fi}"
  );

  // The end of a tableau (ytableau.sty:205-234) less its overlay: ytableau sets the tableau's box to width 0 and
  // follows it with a kern of its width and `\fboxrule`, so that `\ydiagram*` draws the next tableau over it. HTML
  // keeps a box's own width, so that kern doubled the space after every diagram. What is left of it, the
  // `\fboxrule`, is narrower than any space, so it is a zero-width space (`\lx@ytableau@rule`): text after the tabular,
  // which keeps a diagram in a formula raw markup in its `m:mtext`, top-aligned as in TeX, rather than an XMArray (a
  // lone tabular, TeX_Math.pool.ltxml:281-310), which MathML centres on the axis. A `\ydiagram*` overlay now follows
  // its tableau instead.
  DefConstructor!("\\lx@ytableau@rule", "\u{200B}");
  RawTeX!(
    r"\def\endytableau{%
     \crcr\egroup
    \iffalse{\fi}%
   \setbox\tableaux@YT=\hbox{\box\thistableau@YT\unhbox\tableaux@YT\lx@ytableau@rule}%
   \ifstar@YT
    {\ydiagram}%
    {%
     \leavevmode
     \compare@@YT{\centering@YT}{center}%
     \ifeq@YT
      \hbox{$\vcenter{\box\tableaux@YT}$}%
     \else\compare@@YT{\centering@YT}{bottom}%
     \ifeq@YT
      \hbox{\raise\dp\tableaux@YT\box\tableaux@YT}%
     \fi
      \box\tableaux@YT
     \fi
    }%
  }"
  );
});
