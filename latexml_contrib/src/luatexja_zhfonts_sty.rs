//! Binding for `luatexja-zhfonts.sty` — LuaTeX-ja's Chinese font sets (`\usepackage[Fandol]{luatexja-zhfonts}`).
//!
//! Perl LaTeXML ships no `luatexja-zhfonts.sty.ltxml` binding and, at the default `notex` setting, does not raw-load the
//! real file; under `--preload=latexml.sty` (no `rawstyles`) the package was skipped with `Warning:missing_file`.
//! Each option (`SinoType`, `ZhongYi`, `Adobe`, `Founder`, `HanYi-J`, `HanYi-F`, `cwttf`, `Fandol`;
//! luatexja-zhfonts.sty:7-76) sets the main and sans Japanese fonts and defines the faces `\kai` and `\fang` with
//! `\newjfontface`. The font names have no effect on the XML, so each option defines `\kai` and `\fang` as the font
//! switches `luatexja-fontspec` makes (luatexja_fontspec_sty.rs); an option the file does not know is the error it is
//! there. The package requires `luatexja-fontspec` (:4).
//! Where raw TeX Live packages are read (`raw_tex_styles_enabled`) the genuine file loads instead, as before this binding.
use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Where raw TeX Live packages are read (`--includestyles`, the `rawstyles` protocol) the genuine file loads, as it did before
  // this binding existed; the stand-ins below are the native protocol's (`raw_tex_styles_enabled`).
  if raw_tex_styles_enabled() {
    InputDefinitions!("luatexja-zhfonts", noltxml => true, extension => Some(Cow::Borrowed("sty")));
    return Ok(());
  }
  RequirePackage!("luatexja-fontspec");
  for option in ["SinoType", "ZhongYi", "Adobe", "Founder", "HanYi-J", "HanYi-F", "cwttf", "Fandol"] {
    DeclareOption!(option, {
      RawTeX!(r"\newjfontface\kai{}\newjfontface\fang{}");
    });
  }
  ProcessOptions!();
});
