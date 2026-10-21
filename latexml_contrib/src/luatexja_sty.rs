//! Binding for `luatexja.sty` — LuaTeX-ja, Japanese typesetting for LuaTeX. Native stand-ins for its user commands:
//! `\ltjsetparameter`, `\ltjgetparameter`, `\jfont`/`\tfont`, `\ltjdefcharrange`, `\ltjjachar`, `\inhibitglue`,
//! `\yoko`/`\tate`, the kanji font commands (`\kanjifamily`, `\mcfamily`, `\textmc`, `\DeclareKanjiFamily`, …) and
//! the `\mcdefault`-style names `luatexja-fontspec` and `luatexja-preset` read.
//!
//! Perl LaTeXML ships no `luatexja.sty.ltxml` binding and, at the default `notex` setting, does not raw-load the real
//! file; under `--preload=latexml.sty` (no `rawstyles`) the package was skipped with `Warning:missing_file`, and with
//! it `luatexja-fontspec`'s `\setmainjfont`. Under the native protocol it is not raw-loaded: luatexja.sty:55 loads `luatexja-core`
//! and `luatexja-compat`, which load `lltjfont.sty`, a patch of NFSS itself: it redefines `\fontencoding` (lltjfont.sty:
//! 573), `\fontseries` (:813), `\fontshape` (:822) and `\usefont` (:863) over kanji/roman font variants that this engine
//! cannot carry out, so a raw load made `\textbf`, `\textit`, `\textsl`, `\scshape`, `\maketitle`'s bold title,
//! `\caption{…\textbf{c}}`, `\item[\textbf{b}]` and description terms lose their font (no error), `\fontencoding{T1}` and
//! `\usefont` print their operand, and `\jfont\cs=…` printed the font spec (luatexja-core.sty:235-239 reads it with
//! `\directlua`). Japanese text needs nothing from the package here (the engine is Unicode), and a font choice has no
//! effect on the XML, so every command reads the operands its definition takes and produces no output:
//! - `\ltjsetparameter{…}` and `\ltjglobalsetparameter{…}` (luatexja-core.sty:459-471) end in `\ignorespaces`;
//!   `\ltjdefcharrange`, `\ltjdeclarealtfont` and `\ltjclearaltfont` (:295, :557, :581) read their arguments and end in
//!   `\ignorespaces`; `\ltjsetkanjiskip` and `\ltjsetxkanjiskip` (:420-421) read a glue.
//! - `\ltjgetparameter{key}` (:484), with the second argument of the array parameters (:473-483, and `kansujichar`,
//!   luatexja-compat.sty), expands to `\z@`, as the `luatex` persona's stand-in does (latexml_sty/mod.rs; jlreq.cls:551-554, :1240
//!   read it as a number and as a skip): the Lua side keeps the values and there are none here. It is `0` under `\number`,
//!   `\the` and in `\ifnum`, a zero skip in an assignment, and, bare in text, a register that reads on.
//! - `\jfont\cs=<font spec>`, `\tfont`, `\globaljfont`, `\globaltfont` (:235-239) read the control sequence, an optional
//!   `=`, the file name and `at <dimen>` or `scaled <number>` as `\font` does (tex_fonts.rs) and define `\cs` as a
//!   protected empty font switch.
//! - `\ltjjachar`/`\ltjalchar<number>` (:529-532) are `\leavevmode\char`; `\ltjsetwd`/`\ltjsetht`/`\ltjsetdp` and
//!   `\ltjgetwd`/`\ltjgetht`/`\ltjgetdp` (:765-773) are `\wd`/`\ht`/`\dp` and `\the` of them; `\ltjsetmathletter` and
//!   `\ltjunsetmathletter` (:535-540) read a number; `\inhibitglue`, `\disinhibitglue`, `\ltjfakeparbegin`,
//!   `\ltjfakeboxbdd`, `\insertkanjiskip`, `\insertxkanjiskip`, `\yoko`, `\tate`, `\dtou` and `\utod` (:286-291,
//!   :587-598) do nothing; `\zw`, `\zh` (:257-260) are the 10pt of the persona's stand-ins (latexml_sty/mod.rs) and
//!   `\jQ`, `\jH` (:131-132) a 0.25mm dimension; `\kansuji<number>` (luatexja-compat.sty) is the kanji digits.
//! - lltjdefs.sty and lltjfont.sty: the font-family names (`\mcdefault`, `\gtdefault`, `\jttdefault`,
//!   `\kanjiencodingdefault`, …), `\kanjifamily`, `\kanjiseries`, `\kanjishape`, `\kanjiencoding`, `\mcfamily`,
//!   `\gtfamily`, `\textmc`, `\textgt`, `\mc`, `\gt`, `\romanfamily` (= `\fontfamily`, lltjfont.sty:617-619) and its
//!   siblings, the `\Declare…Kanji…` and `\…RelationFont` declarations and `\ystrut`/`\tstrut` (= `\strut`).
//!
//! The package option `disablejfam` (luatexja.sty:48) sets `\ifltj@disablejfam`; other options are ignored (:49).
//! Not provided: the code conversions `\euc`, `\jis`, `\sjis`, `\kuten` (luatexja-compat.sty; they need the tables),
//! `\ucs` is `\char`, and everything under `luatexja.*` in Lua. OXIDIZED_DESIGN #614.
//!
//! Where raw TeX Live packages are read (`--includestyles`, the `rawstyles` protocol; `raw_tex_styles_enabled`) the genuine
//! file loads, as it did before this binding: `jlreq.cls` and the classes built on luatexja use its internals (`\Cht`,
//! `\ltj@setpar@global`, the `ltj` keys; `perfect_kernel_gemini::kksymbols_fontfamily_no_text_leak`,
//! `perfect_kernel_batch56::unwound_titlepage_rehomes_its_paragraphs`), which the stand-ins do not define, and the font
//! loss above stays there (not ruled on).
use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Where raw TeX Live packages are read (`--includestyles`, the `rawstyles` protocol) the genuine file loads, as it did before
  // this binding existed; the stand-ins below are the native protocol's (`raw_tex_styles_enabled`).
  if raw_tex_styles_enabled() {
    InputDefinitions!("luatexja", noltxml => true, extension => Some(Cow::Borrowed("sty")));
    return Ok(());
  }
  RawTeX!(r"\ifx\ifltj@disablejfam\@undefined\newif\ifltj@disablejfam\fi");
  DeclareOption!("disablejfam", {
    RawTeX!(r"\ltj@disablejfamtrue");
  });
  DeclareOption!(None, {});
  ProcessOptions!();
  RawTeX!(r"\def\LuaTeXjaversion{20261004.0}\let\LuaTeXjaAvailable=t
\ifx\ltj@dimen@zw\@undefined\newdimen\ltj@dimen@zw \ltj@dimen@zw=10pt\fi
\ifx\ltj@dimen@zh\@undefined\newdimen\ltj@dimen@zh \ltj@dimen@zh=10pt\fi
\ifx\zw\@undefined\def\zw{\ltj@dimen@zw}\fi
\ifx\zh\@undefined\def\zh{\ltj@dimen@zh}\fi
\newdimen\ltj@dimen@jQ \ltj@dimen@jQ=0.25mm \let\jQ=\ltj@dimen@jQ \let\jH=\jQ
\newcount\ltjlineendcomment
\ifdefined\hour\else\csname newcount\endcsname\hour\fi
\ifdefined\minute\else\csname newcount\endcsname\minute\fi");
  // luatexja-core.sty:459-471, :295-330, :557-585: operands read, then `\ignorespaces`.
  RawTeX!(r"\protected\def\ltjsetparameter#1{\ignorespaces}
\protected\def\ltjglobalsetparameter#1{\ignorespaces}
\protected\def\ltjdefcharrange#1#2{\ignorespaces}
\protected\def\ltjdeclarealtfont#1#2#3{\ignorespaces}
\protected\def\ltjclearaltfont#1{\ignorespaces}");
  DefPrimitive!("\\ltjsetkanjiskip Glue", None);
  DefPrimitive!("\\ltjsetxkanjiskip Glue", None);
  // luatexja-core.sty:473-503: the array parameters take a second operand. The value is `0`.
  RawTeX!(r"\def\lx@ltj@array@param#1{\expandafter\let\csname ltj@@array@param/#1\endcsname=t}
\lx@ltj@array@param{prebreakpenalty}\lx@ltj@array@param{postbreakpenalty}\lx@ltj@array@param{jaxspmode}
\lx@ltj@array@param{alxspmode}\lx@ltj@array@param{kcatcode}\lx@ltj@array@param{jacharrange}
\lx@ltj@array@param{chartorange}\lx@ltj@array@param{boxdir}\lx@ltj@array@param{kansujichar}
\def\lx@ltj@getparam@one#1{\z@}
\def\lx@ltj@getparam@two#1#2{\z@}
\def\ltjgetparameter#1{\ifcsname ltj@@array@param/#1\endcsname
  \expandafter\lx@ltj@getparam@two\else\expandafter\lx@ltj@getparam@one\fi{#1}}");
  // luatexja-core.sty:235-239 `\jfont\cs=<font spec>` (read like `\font`: tex_fonts.rs, tex.web §1257).
  DefPrimitive!("\\lx@ltj@fontdef RedefinableToken SkipSpaces SkipKeyword:= SkipSpaces TeXFileName",
  sub[(cs, _name)] {
    if read_keyword(&["at"])?.is_some() {
      read_dimension()?;
    } else if read_keyword(&["scaled"])?.is_some() {
      read_number()?;
    }
    skip_spaces()?;
    if !is_definition_locked(&cs) {
      let scope = if get_prefix("global") { Scope::Global } else { Scope::Local };
      def_macro(
        cs,
        None,
        Tokens::default(),
        Some(ExpandableOptions { protected: true, scope: Some(scope), ..Default::default() }),
      )?;
    }
    after_assignment();
  });
  RawTeX!(r"\protected\def\jfont{\lx@ltj@fontdef}\protected\def\tfont{\lx@ltj@fontdef}
\protected\def\globaljfont{\global\lx@ltj@fontdef}\protected\def\globaltfont{\global\lx@ltj@fontdef}");
  // luatexja-core.sty:286-291, :587-598: nothing to do without the Lua side.
  RawTeX!(r"\protected\def\inhibitglue{}\protected\def\disinhibitglue{}\protected\def\ltjfakeparbegin{}
\protected\def\ltjfakeboxbdd{}\protected\def\insertkanjiskip{}\protected\def\insertxkanjiskip{}
\protected\def\yoko{}\protected\def\tate{}\protected\def\dtou{}\protected\def\utod{}
\protected\def\<{\ifvmode\leavevmode\fi}");
  // luatexja-core.sty:529-540, :765-773, luatexja-compat.sty.
  RawTeX!(r"\protected\def\ltjjachar{\leavevmode\char}\protected\def\ltjalchar{\leavevmode\char}
\let\ltjsetwd\wd \let\ltjsetht\ht \let\ltjsetdp\dp
\def\ltjgetwd{\the\wd}\def\ltjgetht{\the\ht}\def\ltjgetdp{\the\dp}
\def\ucs{\char}");
  DefPrimitive!("\\ltjsetmathletter Number", None);
  DefPrimitive!("\\ltjunsetmathletter Number", None);
  DefMacro!("\\kansuji Number", sub[(number)] {
    let digits: String = number.to_string().chars().map(|c| match c {
      '0' => '〇', '1' => '一', '2' => '二', '3' => '三', '4' => '四',
      '5' => '五', '6' => '六', '7' => '七', '8' => '八', '9' => '九',
      other => other,
    }).collect();
    Tokens::new(Explode!(digits))
  });
  // lltjdefs.sty:29-35, :71-74, lltjfont.sty:545-1010: the kanji font names and switches.
  RawTeX!(r"\def\mcdefault{mc}\def\gtdefault{gt}\def\jttdefault{\mcdefault}
\def\kanjiencodingdefault{JY3}\def\kanjifamilydefault{\mcdefault}
\def\kanjiseriesdefault{\mddefault}\def\kanjishapedefault{\shapedefault}
\def\kanjifamily#1{}\def\kanjiseries#1{}\def\kanjishape#1{}\def\kanjiencoding#1{}
\def\mcfamily{}\def\gtfamily{}
\DeclareTextFontCommand{\textmc}{\mcfamily}\DeclareTextFontCommand{\textgt}{\gtfamily}
\DeclareOldFontCommand{\mc}{\normalfont\mcfamily}{}\DeclareOldFontCommand{\gt}{\normalfont\gtfamily}{}
\let\romanfamily\fontfamily \let\romanseries\fontseries \let\romanshape\fontshape
\let\romanencoding\fontencoding
\let\ystrut\strut \let\tstrut\strut \let\dstrut\strut \let\zstrut\strut
\def\adjustbaseline{}\def\userelfont{}
\def\DeclareKanjiFamily#1#2#3{}\def\DeclareKanjiEncodingDefaults#1#2{}
\def\DeclareKanjiSubstitution#1#2#3#4{}\def\DeclareErrorKanjiFont#1#2#3#4#5{}
\def\DeclareYokoKanjiEncoding#1#2#3{}\def\DeclareTateKanjiEncoding#1#2#3{}
\def\DeclareKanjiEncoding#1#2#3{}\def\KanjiEncodingPair#1#2{}
\def\DeclareRelationFont#1#2#3#4#5#6#7#8{}\def\SetRelationFont#1#2#3#4#5#6#7#8{}
\def\DeclareAlternateKanjiFont#1#2#3#4#5#6#7#8#9{\ignorespaces}\def\ClearAlternateKanjiFont#1#2#3#4{}");
});
