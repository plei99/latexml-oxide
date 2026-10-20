//! Binding for `beamerthemeDarmstadt.sty` — beamer's Darmstadt theme, reached by `\usetheme{Darmstadt}`.
//!
//! Perl LaTeXML ships no `beamerthemeDarmstadt.sty.ltxml` binding and, at the default `notex` setting, does not
//! raw-load the real file; under `--preload=latexml.sty` (no `rawstyles`) the package was skipped with
//! `Warning:missing_file`. The beamer binding routes `\usetheme{X}` to `\usepackage[opts]{beamerthemeX}`
//! (`\beamer@calltheme`, beamer_cls.rs) so this file is requested by that name. Raw-loads the genuine TL theme; the
//! outer/inner/colour themes it names (`\useoutertheme{smoothbars}` …) read raw through its own `\usepackage`s
//! while the wrapped file is read.
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("beamerthemeDarmstadt", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
