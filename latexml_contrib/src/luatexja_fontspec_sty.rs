//! Binding for `luatexja-fontspec.sty` — fontspec for LuaTeX-ja: `\setmainjfont`, `\setsansjfont`, `\setmonojfont`,
//! `\jfontspec`, the `\newjfontfamily` / `\newjfontface` definers and `\defaultjfontfeatures` /
//! `\addjfontfeatures`, which select the Japanese (kanji) font beside the Latin one.
//!
//! Perl LaTeXML ships no `luatexja-fontspec.sty.ltxml` binding and, at the default `notex` setting, does not raw-load
//! the real file; under `--preload=latexml.sty` (no `rawstyles`) the package was skipped with `Warning:missing_file`,
//! `\setmainjfont{FandolSong}` was `Error:undefined` and the font name printed as page text. A font choice has no
//! effect on the XML, so each command only reads the arguments its signature takes (luatexja-fontspec-29e.sty:367-505,
//! all `O{} m O{}` or `m O{} m O{}` or `t+ o m` or `m`), exactly as the bound `fontspec` does for its Latin twins
//! (fontspec_sty.rs), and the real file's expl3 patching of fontspec's internals (:37-360, :482-507) is not run. As
//! the file does (luatexja-fontspec.sty:7, :30) it requires `l3keys2e`, `luatexja` (native, `luatexja_sty.rs`) and
//! `fontspec`. Its options are read as luatexja-fontspec.sty:19-27 reads them: `match` makes `\jttdefault` the
//! `\gtdefault` (luatexja-fontspec-29e.sty:11-14), `scale=` sets `\Cjascale`, which nothing here uses, and `pass=` and any
//! other option go to fontspec. The definers define their control sequence as a protected font switch that only reselects the
//! font (:406-412 define `\cs` as `\kanjifamily{…}\selectfont`; there is no font to select), so `{\cs text}` works.
//! The three font setters end in `\normalfont` (luatexja-fontspec-29e.sty:382, :393, :401), `\jfontspec` and the feature
//! commands in `\ignorespaces` (:370, :465, :504).
//! Where raw TeX Live packages are read (`raw_tex_styles_enabled`) the genuine file loads instead, as before this binding.
use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Where raw TeX Live packages are read (`--includestyles`, the `rawstyles` protocol) the genuine file loads, as it did before
  // this binding existed; the stand-ins below are the native protocol's (`raw_tex_styles_enabled`).
  if raw_tex_styles_enabled() {
    InputDefinitions!("luatexja-fontspec", noltxml => true, extension => Some(Cow::Borrowed("sty")));
    return Ok(());
  }
  RequirePackage!("l3keys2e");
  RequirePackage!("luatexja");
  // luatexja-fontspec.sty:19-28 the package's own keys: `match` (:21; luatexja-fontspec-29e.sty:11-14 make
  // `\jttdefault` the `\gtdefault`), `scale=<n>` (:25, `\Cjascale`) and `pass=<options>` (:23, passed to fontspec); any other option
  // is passed to fontspec (:26).
  RawTeX!(r"\def\lx@ltjfs@option#1{\lx@ltjfs@split#1==\@nil{#1}}
\def\lx@ltjfs@split#1=#2=#3\@nil#4{%
  \def\lx@ltjfs@key{#1}\def\lx@ltjfs@a{match}\def\lx@ltjfs@b{scale}\def\lx@ltjfs@c{pass}%
  \ifx\lx@ltjfs@key\lx@ltjfs@a\def\jttdefault{\gtdefault}\else\ifx\lx@ltjfs@key\lx@ltjfs@b\gdef\Cjascale{#2}\else
    \ifx\lx@ltjfs@key\lx@ltjfs@c\PassOptionsToPackage{#2}{fontspec}\else\PassOptionsToPackage{#4}{fontspec}\fi
  \fi\fi}");
  DeclareOption!(None, {
    Digest!("\\expandafter\\lx@ltjfs@option\\expandafter{\\CurrentOption}")?;
  });
  ProcessOptions!();
  RequirePackage!("fontspec");
  // luatexja-fontspec.sty:10-17 the default kanji scale is `\zw / 16` at 16pt, i.e. 1.
  if !is_defined("\\Cjascale") {
    DefMacro!("\\Cjascale", "1");
  }
  // luatexja-fontspec-29e.sty:367-401 `\jfontspec` and `\set{main,sans,mono}jfont`, all `{ O{} m O{} }`.
  DefMacro!("\\jfontspec[]{}[]", "\\selectfont\\ignorespaces");
  DefMacro!("\\setmainjfont[]{}[]", "\\normalfont");
  DefMacro!("\\setsansjfont[]{}[]", "\\normalfont");
  DefMacro!("\\setmonojfont[]{}[]", "\\normalfont");
  // :416-458 the family and face definers, all `{ m O{} m O{} }`: `\cs`, features, font name, features. Like the
  // Latin definers (fontspec_sty.rs `\lx@fontspec@definer`) they define `\cs` as a protected `\selectfont`.
  DefMacro!("\\lx@luatexja@jdefiner DefToken []{}[]", sub[(cs, _pre, _font, _post)] {
    def_macro(
      cs,
      None,
      Tokens::new(vec![T_CS!("\\selectfont")]),
      Some(ExpandableOptions { protected: true, ..Default::default() }),
    )?;
    Ok(Tokens::default())
  });
  for definer in [
    "\\newjfontfamily",
    "\\renewjfontfamily",
    "\\setjfontfamily",
    "\\providejfontfamily",
    "\\newjfontface",
    "\\renewjfontface",
    "\\setjfontface",
    "\\providejfontface",
  ] {
    Let!(&T_CS!(definer), "\\lx@luatexja@jdefiner");
  }
  // :460-465 `\defaultjfontfeatures` is `{ t+ o m }`; :488-505 `\addjfontfeatures` is `{ m }`, `\addjfontfeature`
  // its alias (:507).
  DefMacro!("\\defaultjfontfeatures OptionalMatch:+ []{}", "\\ignorespaces");
  DefMacro!("\\addjfontfeatures{}", "\\ignorespaces");
  DefMacro!("\\addjfontfeature{}", "\\ignorespaces");
});
