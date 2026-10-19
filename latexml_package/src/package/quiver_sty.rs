//! quiver.sty (2026/07/27) — the TikZ styles of diagrams exported by the quiver editor.
//!
//! No Perl binding exists; native binding, OXIDIZED_DESIGN #588(a). quiver.sty loads tikz-cd and
//! amssymb, the TikZ libraries calc, decorations.pathmorphing, spath3 and nfold (quiver.sty:21-30),
//! and defines the styles its exports name: `curve={height=…,pos=…}` (a Bézier `to path` bent by a
//! height, :32-40), `between={a}{b}` (the arrow kept between two fractions of its length with spath3,
//! :42-44), the tails `tail reversed`, `2tail`, `2tail reversed`, `coil`, `coil'`, the head
//! `multimap` and `no body` (:46-56). The binding loads the same packages and libraries and defines the
//! same styles, so a diagram is the tikz-cd diagram the editor exported, drawn by this converter's pgf
//! SVG driver.
//! Witnesses (errors in the default configuration, before → after): 2610.14931 (5 → 0), 2610.17060 (7 → 0), 2610.20932 (6 → 0), 2610.19203 (5 → 0).
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // quiver.sty:21-30
  RequirePackage!("tikz-cd");
  RequirePackage!("amssymb");
  RawTeX!(r"\usetikzlibrary{calc}\usetikzlibrary{decorations.pathmorphing}\usetikzlibrary{spath3}\usetikzlibrary{nfold}");
  // quiver.sty:32-56
  RawTeX!(r"\tikzset{curve/.style={settings={#1},to path={(\tikztostart)
    .. controls ($(\tikztostart)!\pv{pos}!(\tikztotarget)!\pv{height}!270:(\tikztotarget)$)
    and ($(\tikztostart)!1-\pv{pos}!(\tikztotarget)!\pv{height}!270:(\tikztotarget)$)
    .. (\tikztotarget)\tikztonodes}},
    settings/.code={\tikzset{quiver/.cd,#1}
        \def\pv##1{\pgfkeysvalueof{/tikz/quiver/##1}}},
    quiver/.cd,pos/.initial=0.35,height/.initial=0}
\tikzset{between/.style n args={2}{/tikz/execute at end to={
    \tikzset{spath/split at keep middle={current}{#1}{#2}}
}}}
\tikzset{
    tail reversed/.code={\pgfsetarrowsstart{tikzcd to}},
    2tail/.code={\pgfsetarrowsstart{Implies[reversed]}},
    2tail reversed/.code={\pgfsetarrowsstart{Implies}},
    coil/.code={\pgfsetarrowsstart{Glyph[glyph math command=looparrowleft]}},
    coil'/.code={\pgfsetarrowsstart{Glyph[glyph math command=looparrowleft, swap]}},
    multimap/.code={\pgfsetarrowsend{Glyph[glyph math command=multimap]}},
    no body/.style={/tikz/dash pattern=on 0 off 1mm},
}");
});
