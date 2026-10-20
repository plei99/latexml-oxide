//! Binding for `extdash.sty` (ncctools) — breakable dashes: `\Hyphdash`, `\Endash`, `\Emdash`, `\Halfspace`, and
//! with the `shortcuts` option `\-` followed by `-`, `--`, `/`, `,` (extdash.sty:79-110).
//!
//! Perl LaTeXML ships no `extdash.sty.ltxml` binding and, at the default `notex` setting, does not raw-load the
//! real file; under `--preload=latexml.sty` (no `rawstyles`) the package was skipped with `Warning:missing_file`.
//! Without the file the commands are undefined control sequences. Raw-loads the genuine TL file; the em dash comes
//! out as U+2014 between the package's half-spaces.
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("extdash", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
