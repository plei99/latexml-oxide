//! Binding for `luatexja-adjust.sty` — LuaTeX-ja's line-end adjustment (`\ltjenableadjust`, `\ltjdisableadjust`, the
//! ghost characters).
//!
//! Perl LaTeXML ships no `luatexja-adjust.sty.ltxml` binding and, at the default `notex` setting, does not raw-load the
//! real file; under `--preload=latexml.sty` (no `rawstyles`) the package was skipped with `Warning:missing_file`.
//! Line breaking is not the XML's, so the commands only read their operands: `\ltjenableadjust[<keys>]` (luatexja-
//! adjust.sty:105-116), `\ltjdisableadjust` (:111), and `\ltjghostjachar`, `\ltjghostbeforejachar` and
//! `\ltjghostafterjachar` (:149-151), which do nothing. The package requires `luatexja` (native, `luatexja_sty.rs`).
//! Where raw TeX Live packages are read (`raw_tex_styles_enabled`) the genuine file loads instead, as before this binding.
use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Where raw TeX Live packages are read (`--includestyles`, the `rawstyles` protocol) the genuine file loads, as it did before
  // this binding existed; the stand-ins below are the native protocol's (`raw_tex_styles_enabled`).
  if raw_tex_styles_enabled() {
    InputDefinitions!("luatexja-adjust", noltxml => true, extension => Some(Cow::Borrowed("sty")));
    return Ok(());
  }
  RequirePackage!("luatexja");
  RawTeX!(r"\protected\def\ltjenableadjust{\@ifnextchar[\lx@ltjadjust@keys{}}
\def\lx@ltjadjust@keys[#1]{}
\protected\def\ltjdisableadjust{}
\protected\def\ltjghostjachar{}\protected\def\ltjghostbeforejachar{}\protected\def\ltjghostafterjachar{}");
});
