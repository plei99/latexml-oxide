//! Binding for `beamerthememetropolis.sty` — the Metropolis beamer theme, reached by `\usetheme{metropolis}`.
//!
//! Perl LaTeXML ships no `beamerthememetropolis.sty.ltxml` binding and, at the default `notex` setting, does not
//! raw-load the real file; under `--preload=latexml.sty` (no `rawstyles`) the package was skipped with
//! `Warning:missing_file`. The beamer binding routes `\usetheme{X}` to `\usepackage[opts]{beamerthemeX}`
//! (`\beamer@calltheme`, beamer_cls.rs) so this file is requested by that name. Raw-loads the genuine TL theme; its
//! `\setbeamercolor` and `\definecolor` calls register the colours documents use, and its nested `\RequirePackage`s
//! (pgfopts, pgfplotsthemetol) read raw. The package's own `Package beamerthememetropolis Warning: You need to
//! compile with XeLaTeX or LuaLaTeX to use the Fira fonts` is kept: it is the real file's warning.
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("beamerthememetropolis", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
