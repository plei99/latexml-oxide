//! old-arrows.sty (v2.0, 2017/01/05) — Computer Modern's pre-1992 arrowheads, and `\var…`
//! names for the other design.
//!
//! No Perl binding exists; native binding, OXIDIZED_DESIGN #588(z). The package redeclares the
//! arrows in the `oasy` font (`default`, `old`) and adds `\var…` arrows in the other one (`new`,
//! `old`). The two designs differ only in the arrowhead's drawing: Unicode and MathML have one
//! character for each arrow, so every arrow keeps its meaning and each `\var…` name is the arrow
//! it draws. Raw, its redefinitions break the composite arrows into pieces (`\longrightarrow` is
//! `\relbarra\joinrelaz\rightarrow`, old-arrows.sty:115-118) and turn mathtools' `\xhookrightarrow`
//! and friends into `\ext@arrow` (:131-149), which this converter only approximates; the binding
//! keeps the semantic arrows and defines the commands the package adds.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // old-arrows.sty:20-38: `new` and `old` exclude each other; `default` is executed first.
  RawTeX!(r"\newif\ifoldarrows@default
\newif\ifoldarrows@new
\newif\ifoldarrows@old
\def\oldarrows@opterr{\PackageError{old-arrows}{`new' and `old' options are incompatible}{I'm sorry, you cannot use the `new' and `old' options simultaneously! Ciao ;) \MessageBreak}}");
  DeclareOption!("default", { RawTeX!(r"\oldarrows@defaulttrue"); });
  DeclareOption!("new", { RawTeX!(r"\ifoldarrows@old\oldarrows@opterr\fi\oldarrows@newtrue"); });
  DeclareOption!("old", { RawTeX!(r"\ifoldarrows@new\oldarrows@opterr\fi\oldarrows@defaultfalse\oldarrows@oldtrue"); });
  execute_options(&["default"])?;
  ProcessOptions!();

  // The building blocks the package declares (:103-110, :276-281): `\meno` is the arrow's bar
  // (a minus in the arrow font), `\relbarra`/`\joinrelaz` the package's `\relbar`/`\joinrel`.
  DefMath!("\\meno", "\u{2212}", role => "ADDOP", meaning => "minus");
  Let!("\\relbarra", "\\relbar");
  Let!("\\joinrelaz", "\\joinrel");

  // Long hooked and harpoon arrows (default :152-163, old :476-487): no long forms in Unicode, so
  // each is its arrow's character under its own name.
  DefMath!("\\longhookrightarrow", "\u{21AA}", role => "ARROW");
  DefMath!("\\longhookleftarrow", "\u{21A9}", role => "ARROW");
  DefMath!("\\longrightharpoonup", "\u{21C0}", role => "ARROW");
  DefMath!("\\longrightharpoondown", "\u{21C1}", role => "ARROW");
  DefMath!("\\longleftharpoonup", "\u{21BC}", role => "ARROW");
  DefMath!("\\longleftharpoondown", "\u{21BD}", role => "ARROW");

  // A mapsfrom arrow with labels (:241-243, :335-338).
  DefMath!("\\lx@oldarrows@mapsfrom", "\u{21A4}", role => "ARROW", stretchy => true, alias => "\\mapsfrom");
  // The `\var…` family (new :165-243, old :260-470): the same arrows in the other arrowhead
  // design, so each is the arrow itself. The amsmath-, mathtools- and stmaryrd-dependent ones
  // exist only when that package is loaded, as the package tests (:224-243, :297-338).
  RawTeX!(r"\def\lx@oldarrows@var{\let\varleftarrow\leftarrow \let\vargets\varleftarrow
\let\varrightarrow\rightarrow \let\varto\varrightarrow
\let\varleftrightarrow\leftrightarrow
\let\varnearrow\nearrow \let\varsearrow\searrow \let\varnwarrow\nwarrow \let\varswarrow\swarrow
\let\varmapstochar\mapstochar \let\varmapsto\mapsto
\let\varhookrightarrow\hookrightarrow \let\varhookleftarrow\hookleftarrow
\let\varuparrow\uparrow \let\vardownarrow\downarrow \let\varupdownarrow\updownarrow
\let\varoverrightarrow\overrightarrow \let\varoverleftarrow\overleftarrow
\let\varlongrightarrow\longrightarrow \let\varlongleftarrow\longleftarrow
\let\varlongmapsto\longmapsto \let\varlongleftrightarrow\longleftrightarrow
\let\varlonghookrightarrow\longhookrightarrow \let\varlonghookleftarrow\longhookleftarrow
\@ifpackageloaded{amsmath}{\let\varoverleftrightarrow\overleftrightarrow
  \let\varunderrightarrow\underrightarrow \let\varunderleftarrow\underleftarrow
  \let\varunderleftrightarrow\underleftrightarrow
  \let\varxrightarrow\xrightarrow \let\varxleftarrow\xleftarrow
  \def\varvarinjlim{\varinjlim}\def\varvarprojlim{\varprojlim}}{}%
\@ifpackageloaded{mathtools}{\let\varxleftrightarrow\xleftrightarrow
  \let\varxhookleftarrow\xhookleftarrow \let\varxhookrightarrow\xhookrightarrow
  \let\varxmapsto\xmapsto}{}%
\@ifpackageloaded{stmaryrd}{\def\varxmapsfrom{\lx@long@arrow{\varxmapsfrom}{\lx@oldarrows@mapsfrom}}%
  \ifoldarrows@old\def\xmapsfrom{\lx@long@arrow{\xmapsfrom}{\lx@oldarrows@mapsfrom}}\fi}{}}
\ifoldarrows@new\lx@oldarrows@var\fi
\ifoldarrows@old\lx@oldarrows@var\fi");
  // old-arrows.sty:488-491
  RawTeX!(r"\@ifpackageloaded{stmaryrd}{\let\varmapsfrom\mapsfrom\let\varlongmapsfrom\longmapsfrom}{}");
});
