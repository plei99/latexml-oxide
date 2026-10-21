//! Binding for `luatexja-preset.sty` — LuaTeX-ja's Japanese font presets (`\usepackage[haranoaji]{luatexja-preset}`,
//! `\ltjnewpreset`, `\ltjapplypreset`).
//!
//! Perl LaTeXML ships no `luatexja-preset.sty.ltxml` binding and, at the default `notex` setting, does not raw-load
//! the real file; under `--preload=latexml.sty` (no `rawstyles`) the package was skipped with `Warning:missing_file`.
//! It is not raw-loaded here: the file requires `luatexja` (luatexja-preset.sty:5; native, `luatexja_sty.rs`) and builds
//! the kanji font families with expl3 over `\DeclareKanjiFamily` and fontspec, none of which has an effect on the XML.
//! The binding reads every option (the presets, `bold`, `deluxe`, `expert`, `nocheck`, `scale=`, `match`, …; keys the file
//! defines with `.code:n`/`.bool_set:N`, :36-105), requires `luatexja-fontspec` (:641) unless `nfssonly` is given
//! (:76), and defines `\ltjnewpreset` (starred or not, `{name}{keys}`, :736) and `\ltjapplypreset{keys}` (:766), which
//! ends in `\normalfont`, as the file does.
//! Where raw TeX Live packages are read (`raw_tex_styles_enabled`) the genuine file loads instead, as before this binding.
use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Where raw TeX Live packages are read (`--includestyles`, the `rawstyles` protocol) the genuine file loads, as it did before
  // this binding existed; the stand-ins below are the native protocol's (`raw_tex_styles_enabled`).
  if raw_tex_styles_enabled() {
    InputDefinitions!("luatexja-preset", noltxml => true, extension => Some(Cow::Borrowed("sty")));
    return Ok(());
  }
  RequirePackage!("luatexja");
  RawTeX!(r"\newif\ifltj@preset@nfssonly");
  DeclareOption!("nfssonly", {
    RawTeX!(r"\ltj@preset@nfssonlytrue");
  });
  DeclareOption!(None, {});
  ProcessOptions!();
  RawTeX!(r"\ifltj@preset@nfssonly\else\RequirePackage{luatexja-fontspec}\fi
\def\ltjnewpreset{\@ifstar{\lx@ltjpreset@new}{\lx@ltjpreset@new}}
\def\lx@ltjpreset@new#1#2{}
\def\ltjapplypreset#1{\normalfont}");
});
