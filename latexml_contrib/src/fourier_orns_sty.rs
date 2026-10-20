//! Binding for `fourier-orns.sty` — the Fourier ornaments (`\warning`, `\lefthand`, `\decoone`, `\aldine`,
//! `\eurologo`, …), each a slot of an ornament font through `\FourierOrns` — `futs` under `noOTF`
//! (fourier-orns.sty:29, which fourier.sty:61 passes), a fontspec family of `FourierOrns-Regular.otf` under a TU
//! engine (:16-27).
//!
//! Perl LaTeXML ships no `fourier-orns.sty.ltxml` binding and, at the default `notex` setting, does not raw-load the
//! real file; under `--preload=latexml.sty` (no `rawstyles`) the package was skipped with `Warning:missing_file`.
//! Raw-loads the genuine TL file, then redefines `\FourierOrns` — which only the package's own commands use
//! (fourier-orns.sty:30, 33-72, 80-82) — as a one-argument macro for the slot: it sets the font and then
//! LaTeXML's font encoding to `futs` (`\lx@fontencoding`, which loads `futs_fontmap.rs`) for that one character, so
//! the ornaments are their Unicode glyphs, not the slot letters, in both branches (the OTF has the ornaments at
//! the same ASCII slots). The encoding covers the slot only: `futs` is also Fourier's Utopia text family (t1futs.fd,
//! fourier.sty:58), and a scope-wide encoding would decode text after a `\normalfont` through the ornament map.
//! `\FourierOrns` is therefore no longer a free-standing font switch. Under a TU engine `\grappe`, `\leftgrappe` and
//! `\rightgrappe` (:80-82, `\char"2766`, `"2619`, `"2767`: Unicode code points the `futs` table does not cover) are
//! redefined to use the original fontspec switch directly. Options are the file's: a direct
//! `\usepackage{fourier-orns}` under a TU persona takes the fontspec branch; fourier.sty's `noOTF` the `futs` one. OXIDIZED_DESIGN #591.
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("fourier-orns", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  RawTeX!(
    r##"\ifFourier@TU
  \ifdefined\FourierOrns
    \let\lx@FourierOrns@otf\FourierOrns
    \renewcommand*{\FourierOrns}[1]{{\lx@FourierOrns@otf\lx@fontencoding{futs}#1}}
    \renewcommand*{\grappe}{\texorpdfstring{{\lx@FourierOrns@otf\char"2766}}{\textdagger}}
    \renewcommand*{\leftgrappe}{\texorpdfstring{{\lx@FourierOrns@otf\char"2619}}{\textdagger}}
    \renewcommand*{\rightgrappe}{\texorpdfstring{{\lx@FourierOrns@otf\char"2767}}{\textdagger}}
  \fi
\else
  \renewcommand*{\FourierOrns}[1]{{\fontencoding{U}\fontfamily{futs}\selectfont\lx@fontencoding{futs}#1}}
\fi"##
  );
});
