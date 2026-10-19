//! mathalpha.sty (v1.145, 2025/01/17) and its alias mathalfa.sty — choosing the faces of the
//! calligraphic, script, fraktur and blackboard math alphabets.
//!
//! No Perl binding exists; native binding, OXIDIZED_DESIGN #588(j). The options `cal=`, `scr=`,
//! `frak=`, `bb=` each pick a face for one alphabet and (re)declare its command (mathalpha.sty:68-1100):
//! `cal=` redeclares `\mathcal`, `scr=` is the only path that defines `\mathscr`, `frak=` `\mathfrak`,
//! `bb=` `\mathbb`, each with its bold companion (`\mathbfcal`, `\mathbfscr`, `\mathbffrak`,
//! `\mathbfbb`) when the face has a bold weight; with no option nothing is declared. `bfcal`, `bfscr`,
//! `bffrak`, `bfbb` make the alphabet itself bold and its `\mathbf…` the same (:1102-1105); `oldbold`
//! adds `\mathbbb`, `\mathbfrak`, `\mathbcal`, `\mathbscr` (:1107). What a face changes in a
//! conversion is the alphabet's style: Unicode has one script alphabet (U+1D49C, bold U+1D4D0), whose
//! two designs are the chancery and roundhand variants (StandardizedVariants U+FE00/U+FE01), and the
//! font model carries them as the `caligraphic` and `script` families (`\mathcal`'s and `\mathscr`'s
//! in the kernel, sect07.rs; the MathML stage keeps the chancery one apart as
//! `ltx_font_mathcaligraphic`). So each script face is set in the family of its design — the chancery
//! faces (cm, euler, lucida, mma, pxtx, txupr, zapfc, stix, stixplain, stixtwoplain) as
//! `caligraphic`, the roundhand ones as `script`, whichever command it is chosen for (`cal=boondox`
//! makes `\mathcal` roundhand, `scr=euler` makes `\mathscr` chancery). Fraktur and double-struck
//! faces are the `fraktur` and `blackboard` families, bold by weight. The scaling options only scale
//! the fonts.
//! Witnesses (errors in the default configuration, before → after): 2610.22074 (5 → 2), 2610.10804 and 2610.22161 (`mathalfa`, 1 → 0).
use crate::prelude::*;

const SCRIPT_FACES: &[&str] = &[
  "cm",
  "euler",
  "rsfso",
  "rsfs",
  "lucida",
  "mathpi",
  "mma",
  "mt",
  "mtc",
  "zapfc",
  "esstix",
  "boondox",
  "boondoxo",
  "dutchcal",
  "pxtx",
  "bickham",
  "bickhams",
  "stix",
  "txupr",
  "boondoxupr",
  "kp",
  "stixplain",
  "stixfancy",
  "stixtwoplain",
  "stixtwofancy",
];
/// The chancery designs among the script faces (mathalpha-doc.tex:340-440, the font names).
const CHANCERY_FACES: &[&str] = &[
  "cm",
  "euler",
  "lucida",
  "mma",
  "pxtx",
  "txupr",
  "zapfc",
  "stix",
  "stixplain",
  "stixtwoplain",
];
/// Script faces with no bold weight (mathalpha.sty:482-561).
const SCRIPT_NO_BOLD: &[&str] = &["rsfso", "rsfs", "mathpi", "mtc", "zapfc", "esstix"];
const FRAK_FACES: &[&str] = &[
  "euler", "lucida", "mathpi", "mma", "mt", "esstix", "boondox", "pxtx", "stixtwo",
];
const FRAK_NO_BOLD: &[&str] = &["lucida", "mathpi", "esstix"];
const BB_FACES: &[&str] = &[
  "ams",
  "lucida",
  "mathpi",
  "mma",
  "mt",
  "mth",
  "pazo",
  "fourier",
  "esstix",
  "boondox",
  "px",
  "tx",
  "txof",
  "libus",
  "dsserif",
  "bboldxlight",
  "bboldx",
  "dsfontserif",
  "dsfontsans",
  "stixtwo",
  "stix",
  "ncmbbr",
  "ncmbbk",
];
const BB_NO_BOLD: &[&str] = &[
  "ams",
  "mathpi",
  "pazo",
  "fourier",
  "esstix",
  "tx",
  "libus",
  "dsfontserif",
  "dsfontsans",
  "stixtwo",
];

/// `\mathalpha@boldError` (mathalpha.sty:103-104).
fn bold_error(face: &str, key: &str) -> Result<()> {
  raw_tex(&format!(
    "\\PackageError{{mathalpha}}{{You specified `{key}={face},b{key}', but `{face}' has no bold\\space {key}}}{{}}"
  ))
}

/// `\let` `\<cs>` to the alphabet `\lx@mathalpha@<family>@<series>` with the reversion `\<cs>{…}`.
fn set_alphabet(cs: &str, family: &str, bold: bool) -> Result<()> {
  let series = if bold { "bold" } else { "medium" };
  let body = format!("\\lx@mathalpha@{family}@{series}{{\\{cs}}}");
  def_macro(
    T_CS!(&format!("\\{cs}")),
    parse_parameters("{}", &T_CS!(&format!("\\{cs}")), true)?,
    ExpansionBody::Tokens(mouth::tokenize_internal(TeXString::assembled(format!(
      "{body}{{#1}}"
    )))),
    Some(ExpandableOptions {
      scope: Some(Scope::Global),
      robust: true,
      ..Default::default()
    }),
  )
}

#[rustfmt::skip]
LoadDefinitions!({
  // mathalpha.sty:5-6
  RequirePackage!("xkeyval");
  // One constructor per family and weight: `#1` the command written (the reversion), `#2` the text.
  DefConstructor!("\\lx@mathalpha@caligraphic@medium Undigested {}", "#2",
    bounded => true, require_math => true, reversion => "#1{#2}",
    font => {family => "caligraphic", series => "medium", shape => "upright"});
  DefConstructor!("\\lx@mathalpha@caligraphic@bold Undigested {}", "#2",
    bounded => true, require_math => true, reversion => "#1{#2}",
    font => {family => "caligraphic", series => "bold", shape => "upright"});
  DefConstructor!("\\lx@mathalpha@script@medium Undigested {}", "#2",
    bounded => true, require_math => true, reversion => "#1{#2}",
    font => {family => "script", series => "medium", shape => "upright"});
  DefConstructor!("\\lx@mathalpha@script@bold Undigested {}", "#2",
    bounded => true, require_math => true, reversion => "#1{#2}",
    font => {family => "script", series => "bold", shape => "upright"});
  DefConstructor!("\\lx@mathalpha@fraktur@medium Undigested {}", "#2",
    bounded => true, require_math => true, reversion => "#1{#2}",
    font => {family => "fraktur", series => "medium", shape => "upright"});
  DefConstructor!("\\lx@mathalpha@fraktur@bold Undigested {}", "#2",
    bounded => true, require_math => true, reversion => "#1{#2}",
    font => {family => "fraktur", series => "bold", shape => "upright"});
  DefConstructor!("\\lx@mathalpha@blackboard@medium Undigested {}", "#2",
    bounded => true, require_math => true, reversion => "#1{#2}",
    font => {family => "blackboard", series => "medium", shape => "upright"});
  DefConstructor!("\\lx@mathalpha@blackboard@bold Undigested {}", "#2",
    bounded => true, require_math => true, reversion => "#1{#2}",
    font => {family => "blackboard", series => "bold", shape => "upright"});

  // mathalpha.sty:25-74: the options, as xkeyval reads them (values trimmed; the choices are matched
  // without case, `\define@choicekey*`).
  RawTeX!(r"\def\mathalpha@cal{}\def\mathalpha@scr{}\def\mathalpha@frak{}\def\mathalpha@bb{}
\def\mathalpha@bfcal{}\def\mathalpha@bfscr{}\def\mathalpha@bffrak{}\def\mathalpha@bfbb{}\def\mathalpha@oldbold{}
\DeclareOptionX{cal}{\def\mathalpha@cal{#1}}\DeclareOptionX{scr}{\def\mathalpha@scr{#1}}
\DeclareOptionX{frak}{\def\mathalpha@frak{#1}}\DeclareOptionX{bb}{\def\mathalpha@bb{#1}}
\DeclareOptionX{bfcal}{\def\mathalpha@bfcal{T}}\DeclareOptionX{bfscr}{\def\mathalpha@bfscr{T}}
\DeclareOptionX{bffrak}{\def\mathalpha@bffrak{T}}\DeclareOptionX{bfbb}{\def\mathalpha@bfbb{T}}
\DeclareOptionX{oldbold}{\def\mathalpha@oldbold{T}}
\DeclareOptionX{bbsymbols}{}\DeclareOptionX{calsymbols}{}\DeclareOptionX{fraksymbols}{}\DeclareOptionX{scrsymbols}{}
\DeclareOptionX{scaled}{}\DeclareOptionX{bbscaled}{}\DeclareOptionX{calscaled}{}\DeclareOptionX{frakscaled}{}\DeclareOptionX{scrscaled}{}
\DeclareOptionX{showoptions}{\PackageError{mathalpha}{bb=(ams, lucida, mathpi, mma, mt, mth, pazo, fourier, esstix, boondox, px, tx, txof, libus, dsserif, bboldxLight, bboldx, dsfontserif, dsfontsans, stixtwo, stix, ncmbbr, ncmbbk); cal and scr=(cm, euler, rsfso, rsfs, lucida, mathpi, mma, mt, mtc, zapfc, esstix, boondox, boondoxo, dutchcal, pxtx, bickham, bickhams, stix, txupr, boondoxupr, kp, stixplain, stixfancy, stixtwoplain, stixtwofancy); frak=(euler, lucida, mathpi, mma, mt, esstix, boondox, pxtx, stixtwo)}{}}
\ProcessOptionsX\relax");

  // mathalpha.sty:76-1107: the alphabets the options declare.
  DefPrimitive!("\\lx@mathalpha@declare", {
    let value = |name: &str| -> Result<String> {
      Ok(Expand!(T_CS!(&format!("\\{name}"))).to_string().trim().to_ascii_lowercase())
    };
    let flag = |name: &str| -> Result<bool> {
      Ok(Expand!(T_CS!(&format!("\\{name}"))).to_string().trim() == "T")
    };
    let bad = |key: &str, face: &str| -> Result<()> {
      RawTeX!(format!("\\PackageError{{xkeyval}}{{value `{face}' is not allowed}}{{The value of key `{key}' must be one of its choices.}}").as_str());
      Ok(())
    };
    let script_family = |face: &str| if CHANCERY_FACES.contains(&face) { "caligraphic" } else { "script" };
    // cal= (:482-561), scr= (:926-1097)
    for (key, cs, bfflag) in [("cal", "mathcal", "mathalpha@bfcal"), ("scr", "mathscr", "mathalpha@bfscr")] {
      let face = value(&format!("mathalpha@{key}"))?;
      let bf = flag(bfflag)?;
      if face.is_empty() {
        if bf && is_defined_token(&T_CS!(&format!("\\{cs}"))) {
          let_i(&T_CS!(&format!("\\mathbf{key}")), &T_CS!(&format!("\\{cs}")), Some(Scope::Global));
        }
        continue;
      }
      if !SCRIPT_FACES.contains(&face.as_str()) { bad(key, &face)?; continue; }
      let family = script_family(&face);
      let has_bold = !SCRIPT_NO_BOLD.contains(&face.as_str());
      if bf && !has_bold {
        bold_error(&face, key)?;
      }
      set_alphabet(cs, family, bf && has_bold)?;
      if bf && has_bold {
        let_i(&T_CS!(&format!("\\mathbf{key}")), &T_CS!(&format!("\\{cs}")), Some(Scope::Global));
      } else if has_bold {
        set_alphabet(&format!("mathbf{key}"), family, true)?;
      }
    }
    // frak= (:562-925)
    let face = value("mathalpha@frak")?;
    let bf = flag("mathalpha@bffrak")?;
    if !face.is_empty() {
      if !FRAK_FACES.contains(&face.as_str()) { bad("frak", &face)?; } else {
        let has_bold = !FRAK_NO_BOLD.contains(&face.as_str());
        // mathalpha.sty:746, :752, :783 test `\ifmathalfa@bbb` (`bfbb`) here, not `bffrak`.
        if flag("mathalpha@bfbb")? && !has_bold {
          bold_error(&face, "frak")?;
        }
        // The fraktur faces are declared at weight `m` except pxtx, so `bffrak` makes `\mathfrak` bold
        // only for `frak=pxtx` (mathalpha.sty:562-925).
        set_alphabet("mathfrak", "fraktur", bf && face == "pxtx")?;
        if bf && has_bold {
          let_i(&T_CS!("\\mathbffrak"), &T_CS!("\\mathfrak"), Some(Scope::Global));
        } else if has_bold {
          set_alphabet("mathbffrak", "fraktur", true)?;
        }
      }
    } else if bf && is_defined_token(&T_CS!("\\mathfrak")) {
      let_i(&T_CS!("\\mathbffrak"), &T_CS!("\\mathfrak"), Some(Scope::Global));
    }
    // bb= (:85-480); dsserif/bboldx are those packages' alphabets (dsserif.sty:53 adds `\mathbbb`).
    let face = value("mathalpha@bb")?;
    let bf = flag("mathalpha@bfbb")?;
    if !face.is_empty() {
      if !BB_FACES.contains(&face.as_str()) { bad("bb", &face)?; } else {
        let has_bold = !BB_NO_BOLD.contains(&face.as_str());
        if bf && !has_bold {
          bold_error(&face, "bb")?;
        }
        set_alphabet("mathbb", "blackboard", bf && has_bold)?;
        if bf && has_bold {
          let_i(&T_CS!("\\mathbfbb"), &T_CS!("\\mathbb"), Some(Scope::Global));
        } else if has_bold {
          set_alphabet("mathbfbb", "blackboard", true)?;
        }
        if face == "dsserif" {
          set_alphabet("mathbbb", "blackboard", true)?;
        }
        if face == "stix" {
          // mathalpha.sty:400-410: double-struck italic, as Unicode has it only for ⅅⅆⅇⅈⅉ.
          set_alphabet("mathbbi", "blackboard", false)?;
          set_alphabet("mathbfbbi", "blackboard", true)?;
        }
      }
    } else if bf && is_defined_token(&T_CS!("\\mathbb")) {
      let_i(&T_CS!("\\mathbfbb"), &T_CS!("\\mathbb"), Some(Scope::Global));
    }
    // :1107 `oldbold`
    if flag("mathalpha@oldbold")? {
      for (old, new) in [("mathbbb", "mathbfbb"), ("mathbfrak", "mathbffrak"), ("mathbcal", "mathbfcal"), ("mathbscr", "mathbfscr")] {
        if is_defined_token(&T_CS!(&format!("\\{new}"))) {
          let_i(&T_CS!(&format!("\\{old}")), &T_CS!(&format!("\\{new}")), Some(Scope::Global));
        }
      }
    }
  });
  RawTeX!(r"\lx@mathalpha@declare");
});
