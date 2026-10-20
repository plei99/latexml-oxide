//! Binding for `fourier-orns.sty` — the Fourier ornaments (`\warning`, `\lefthand`, `\decoone`, `\aldine`,
//! `\eurologo`, …), each a slot of the `futs` font through `\FourierOrns`
//! (`\fontencoding{U}\fontfamily{futs}\selectfont`, fourier-orns.sty:29); fourier.sty:61 requires it with `noOTF`.
//!
//! Perl LaTeXML ships no `fourier-orns.sty.ltxml` binding and, at the default `notex` setting, does not raw-load the
//! real file; under `--preload=latexml.sty` (no `rawstyles`) the package was skipped with `Warning:missing_file`.
//! Raw-loads the genuine TL file, then makes `\FourierOrns` also switch LaTeXML's font encoding to `futs`
//! (`\lx@fontencoding`, which loads `futs_fontmap.rs`), so the ornaments are their Unicode glyphs, not the slot
//! letters. The encoding is scoped to `\FourierOrns`: `futs` is also Fourier's Utopia text family (t1futs.fd,
//! fourier.sty:58), and a family-level encoding would decode its T1 body text through the ornament map.
//!
//! Options are the file's: a direct `\usepackage{fourier-orns}` under a TU persona (`\iftutex`, xetex/luatex) takes the
//! package's own fontspec branch (fourier-orns.sty:16-27, `FourierOrns-Regular.otf`) and is left alone; `noOTF`, which
//! fourier.sty passes, selects the `futs` branch this binding decodes.
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("fourier-orns", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  // fourier-orns.sty:29 (the non-TU branch) defines `\FourierOrns` without a LaTeXML font encoding.
  RawTeX!(
    r"\ifFourier@TU\else\renewcommand*{\FourierOrns}{\fontencoding{U}\fontfamily{futs}\selectfont\lx@fontencoding{futs}}\fi"
  );
});
