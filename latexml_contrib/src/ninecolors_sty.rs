//! Binding for `ninecolors.sty` — numbered colour shades with checked contrast (`red6`, …), defined over xcolor.
//!
//! Perl LaTeXML ships no `ninecolors.sty.ltxml` binding and, at the default `notex` setting, does not raw-load the
//! real file; under `--preload=latexml.sty` (no `rawstyles`) the package was skipped with `Warning:missing_file`.
//! Without the file `\textcolor{red6}{x}` is `Error:unexpected:red6 Can't find color named 'red6'`. Raw-loads the
//! genuine TL file; xcolor's `\definecolor`s register the names.
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("ninecolors", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
