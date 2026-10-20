//! Binding for `fourier-orns.sty` — the Fourier ornaments (`\warning`, `\lefthand`, `\decoone`, `\aldine`,
//! `\eurologo`, …), each a slot of an ornament font through `\FourierOrns` — `futs` under `noOTF`
//! (fourier-orns.sty:29, which fourier.sty:61 passes), a fontspec family of `FourierOrns-Regular.otf` under a TU
//! engine (:16-27). `\FourierOrns` is also a font switch for user text (fourier-orns-doc.tex:118-121, 150-151
//! `{\FourierOrns E 1 F}`, :112 `{\Large\FourierOrns W}`).
//!
//! Perl LaTeXML ships no `fourier-orns.sty.ltxml` binding and, at the default `notex` setting, does not raw-load the
//! real file; under `--preload=latexml.sty` (no `rawstyles`) the package was skipped with `Warning:missing_file`.
//! Raw-loads the genuine TL file, then makes `\FourierOrns` also switch LaTeXML's font encoding to `futs`
//! (`\lx@fontencoding`, which loads `futs_fontmap.rs`), so the ornaments are their Unicode glyphs, not the slot
//! letters, in both branches (the OTF has the ornaments at the same ASCII slots). The `futs` family is also
//! Fourier's Utopia text family (t1futs.fd, fourier.sty:58), so the encoding is switched only by `\FourierOrns`,
//! never by the family. The engine's `\normalfont` does not reset the encoding (latex.ltx:14306 does), so text after
//! a `\normalfont` inside an ornament group stays in the ornament encoding (SYNC_STATUS.md). Under a TU engine
//! `\grappe`, `\leftgrappe` and `\rightgrappe` (:80-82, `\char"2766`, `"2619`, `"2767`: Unicode code points the
//! `futs` table does not cover) are redefined to use the original fontspec switch directly. Options are the file's:
//! a direct `\usepackage{fourier-orns}` under a TU persona takes the fontspec branch; fourier.sty's `noOTF` the
//! `futs` one. Loaded by fourier-otf.sty (:17 skips the definition, fourier-otf.sty:109-110 makes it later) the
//! wrap does not apply; that package has no binding here. OXIDIZED_DESIGN #591.
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("fourier-orns", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  RawTeX!(
    r##"\ifFourier@TU
  \ifdefined\FourierOrns
    \let\lx@FourierOrns@otf\FourierOrns
    \renewcommand*{\FourierOrns}{\lx@FourierOrns@otf\lx@fontencoding{futs}}
    \renewcommand*{\grappe}{\texorpdfstring{{\lx@FourierOrns@otf\char"2766}}{\textdagger}}
    \renewcommand*{\leftgrappe}{\texorpdfstring{{\lx@FourierOrns@otf\char"2619}}{\textdagger}}
    \renewcommand*{\rightgrappe}{\texorpdfstring{{\lx@FourierOrns@otf\char"2767}}{\textdagger}}
  \fi
\else
  \renewcommand*{\FourierOrns}{\fontencoding{U}\fontfamily{futs}\selectfont\lx@fontencoding{futs}}
\fi"##
  );
});
