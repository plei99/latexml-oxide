//! Page layout, section-numbering and ToC depth, and the ToC entry commands (`\cft…`).
//!
//! memoir's own definitions of these are TeX (`memoir.cls`); they run raw before this file does. What follows replaces
//! each with a Rust equivalent so that the part of its effect that HTML can show reaches the document:
//!
//! * `\setsecnumdepth`, `\settocdepth` and friends set the `secnumdepth` / `tocdepth` counters the native sectioning
//!   and `\tableofcontents` read. memoir writes `\changetocdepth` into the `.toc` through `\addtocontents`, which
//!   LaTeXML makes a no-op, so before this a preamble `\settocdepth{subsection}` did nothing;
//! * the `\cft<level>font` commands reach the ToC entries through `\format@toctitle@font@<level>` (the hook Perl
//!   `Base_Utility.pool.ltxml:1107-1126` calls for a ToC title), so a ToC entry is typeset in the font the author gave
//!   it. Page numbers, leaders and indents have no HTML counterpart: the other `\cft…` commands are storage only;
//! * the page-geometry arithmetic is a port of memoir's, so `\textwidth` and the margins read back what pdfLaTeX
//!   computes. Nothing in the HTML depends on it, but the document's own lengths (`\the\textwidth`, `\linewidth`
//!   scaled graphics) do.
use latexml_package::prelude::*;

/// TeX source tokenized as a binding's own macro body (`@` a letter).
fn tex(source: impl AsRef<str>) -> Tokens {
  mouth::tokenize_internal(TeXString::assembled(source.as_ref().to_string()))
}

/// memoir's division names and their counter values (`\@setclcnt`, memoir.cls:7544-7565; `\settocdepth@`,
/// memoir.cls:7631-7656 spells the same table).
fn division_level(name: &str) -> Option<i64> {
  Some(match name {
    "none" => -10,
    "book" => -2,
    "part" => -1,
    "chapter" => 0,
    "section" => 1,
    "subsection" => 2,
    "subsubsection" => 3,
    "paragraph" => 4,
    "subparagraph" => 5,
    "all" => 50,
    _ => return None,
  })
}

/// `\@memerror{<message>}{<help>}`: memoir's `\ClassError{memoir}` (memoir.cls:83), raised through the same macro the
/// raw class uses so the log line is the one pdfLaTeX's `Class memoir Error` becomes here.
fn memerror(message: &str, help: &str) -> Tokens {
  tex(format!("\\@memerror{{{message}}}{{{help}}}"))
}

/// The unknown-division-name error of `\@setclcnt` (memoir.cls:7556-7563) and `\settocdepth@` (memoir.cls:7650-7655).
fn unknown_division(name: &str, help: &str) -> Tokens {
  memerror(&format!("Unknown document division name ({name})"), help)
}

/// `\@setclcnt{name}{counter}`: set `counter` (globally, as `\setcounter` does) to the level of division `name`.
fn set_division_counter(name: &str, counter: &str) -> Result<Tokens> {
  match division_level(name) {
    Some(level) => {
      SetCounter!(counter, Number::new(level));
      Ok(Tokens::new(Vec::new()))
    },
    None => Ok(unknown_division(
      name,
      "I'll ignore it. Type <return> and I'll continue.\\MessageBreak \
       If you haven't mistyped the name then use \\protect\\setcounter instead.",
    )),
  }
}

/// `\@setclcnt{name}{secnumdepth}\@setclcnt{name}{maxsecnumdepth}` (memoir.cls:7744-7753): two calls, so an unknown
/// name raises the error twice, as pdfLaTeX does.
fn set_both_division_counters(name: &str) -> Result<Tokens> {
  let mut tokens = set_division_counter(name, "secnumdepth")?.unlist();
  tokens.extend(set_division_counter(name, "maxsecnumdepth")?.unlist());
  Ok(Tokens::new(tokens))
}

// --- Page layout ---------------------------------------------------------------------------------------------------

/// memoir's `\nametest{#}{*}` (memoir.cls:87-96): the argument is the one token `*`.
fn is_star(arg: &Tokens) -> bool {
  let star = T_OTHER!("*");
  let toks = arg.clone().unlist();
  toks.len() == 1 && toks[0] == star
}

/// The value of a TeX `<dimen>` written as `tokens`, read the way `\setlength` reads its argument.
fn dimen_of(tokens: Tokens) -> Result<i64> {
  Ok(
    reading_from_mouth(Mouth::new("", None)?, move || {
      unread(tokens);
      read_dimension()
    })?
    .value_of(),
  )
}

/// The value of a dimension register, in sp.
fn dimen_reg(cs: &str) -> i64 { lookup_dimension(cs).map(|d| d.value_of()).unwrap_or(0) }

/// Assign a dimension register, with TeX's `max_dimen` clamp on the arithmetic (tex.web §1236).
fn set_dimen_reg(cs: &str, sp: i64) -> Result<()> {
  const MAX_DIMEN: i64 = 0o7777777777;
  let rv: RegisterValue = Dimension::new(sp.clamp(-MAX_DIMEN, MAX_DIMEN)).into();
  assign_register(cs, rv, None, Vec::new())
}

/// `<factor><register>` as TeX reads `\setlength{<reg>}{#5<reg>}` (memoir.cls:763-866): `base` goes into the
/// register the ratio multiplies, so TeX's own fixed-point product (tex.web §455) decides the rounding.
fn ratio_of(ratio: &Tokens, base_reg: &str, base: i64) -> Result<i64> {
  set_dimen_reg(base_reg, base)?;
  let mut toks = ratio.clone().unlist();
  toks.push(T_CS!(base_reg));
  dimen_of(Tokens::new(toks))
}

/// memoir's `\setfillsize{T}{C}{L}{R}{r}` (memoir.cls:763-866): with a total `T`, split it into `L + C + R`, where
/// the unknown ones are `*`, `r` is the ratio `R/L` when only one of the two is given, and `C` is the central block.
/// Results go to `\@tempdima` (`L`), `\@tempdimb` (`R`) and `\@tempdimc` (`C`), as memoir leaves them. Returns false
/// for the ambiguous combination (memoir.cls:771-776, :859-864), which memoir reports and sets to zero.
#[allow(clippy::many_single_char_names)]
fn set_fill_size(
  total: &Tokens,
  c: &Tokens,
  l: &Tokens,
  r: &Tokens,
  ratio: &Tokens,
) -> Result<bool> {
  let (a, b, cc);
  let mut ok = true;
  const A: &str = "\\@tempdima";
  const B: &str = "\\@tempdimb";
  if is_star(c) {
    if is_star(l) {
      if is_star(r) {
        ok = false;
        (a, b, cc) = (0, 0, 0);
      } else {
        // C = *, L = *, R given
        b = dimen_of(r.clone())?;
        a = if is_star(ratio) {
          b
        } else {
          ratio_of(ratio, B, b)?
        };
        cc = dimen_of(total.clone())? - a - b;
      }
    } else if is_star(r) {
      // C = *, L given, R = *
      a = dimen_of(l.clone())?;
      b = if is_star(ratio) {
        a
      } else {
        ratio_of(ratio, A, a)?
      };
      cc = dimen_of(total.clone())? - a - b;
    } else {
      a = dimen_of(l.clone())?;
      b = dimen_of(r.clone())?;
      cc = dimen_of(total.clone())? - a - b;
    }
  } else if is_star(l) {
    if is_star(r) {
      cc = dimen_of(c.clone())?;
      if is_star(ratio) {
        // the margins share what C leaves: `\@tempdima = 0.5\@tempdima` (memoir.cls:827)
        let rest = dimen_of(total.clone())? - cc;
        a = ratio_of(&tex("0.5"), A, rest)?;
        b = a;
      } else {
        // memoir.cls:829-840: L = (T - C)/(1 + r), written in sp so that the quotient truncates as TeX's
        // `\divide` does
        let rest = dimen_of(total.clone())? - cc;
        let thousand_r = ratio_of(ratio, A, 1000)?;
        let count = thousand_r + 1000;
        let quotient = if count == 0 { 0 } else { rest / count };
        a = quotient * 1000;
        b = rest - a;
      }
    } else {
      // C given, L = *, R given
      cc = dimen_of(c.clone())?;
      b = dimen_of(r.clone())?;
      a = dimen_of(total.clone())? - cc - b;
    }
  } else if is_star(r) {
    // C given, L given, R = *
    cc = dimen_of(c.clone())?;
    a = dimen_of(l.clone())?;
    b = dimen_of(total.clone())? - cc - a;
  } else {
    ok = false;
    (a, b) = (0, 0);
    cc = dimen_of(c.clone())?;
  }
  set_dimen_reg(A, a)?;
  set_dimen_reg(B, b)?;
  set_dimen_reg("\\@tempdimc", cc)?;
  Ok(ok)
}

/// The ambiguous-combination error of `\setfillsize` (memoir.cls:771-776).
fn ambiguous_fill_size() -> Tokens {
  memerror(
    "The combination of argument values is ambiguous.\\MessageBreak The lengths will be set to zero",
    "\\@ehd",
  )
}

/// How a line count rounds in `\checkthelayout` (memoir.cls:1039-1061): `\textheight` becomes `n` baselines plus a
/// `\topskip`, where `n` is the quotient of what `rounded` returns.
fn fit_text_height(algorithm: &str) -> Result<()> {
  let text_height = dimen_reg("\\textheight");
  let baseline = dimen_of(tex("\\baselineskip"))?;
  let top_skip = dimen_of(tex("\\topskip"))?;
  let measured = match algorithm {
    // \m@mclassicht
    "classic" => text_height,
    // \m@mlinesht
    "lines" => text_height - baseline,
    // \m@mnearestht: `\advance\@tempdima 0.5\baselineskip`
    "nearest" => text_height - top_skip + dimen_of(tex("0.5\\baselineskip"))?,
    _ => return Ok(()),
  };
  let lines = if baseline == 0 {
    0
  } else {
    measured / baseline
  };
  set_dimen_reg("\\@tempdima", measured)?;
  assign_register("\\@tempcnta", Number::new(lines).into(), None, Vec::new())?;
  set_dimen_reg("\\textheight", lines * baseline + top_skip)
}

/// `\checkthelayout[<algorithm>]` (memoir.cls:1096-1206): the warnings for non-positive lengths, the text height
/// fitted to whole lines, the margin recomputed from the paper, and the errors for a layout that overflows its
/// paper. Returns the `\@memwarn` / `\@memerror` calls that report what failed.
fn check_the_layout(algorithm: &str) -> Result<Tokens> {
  let mut report = String::new();
  // `\@memnegtest` / `\@memznegtest` (memoir.cls:1030-1037)
  let warn = |report: &mut String, name: &str, what: &str| {
    report.push_str(&format!("\\@memwarn{{\\protect\\{name}\\space is {what}}}"));
  };
  for name in ["trimedge", "trimtop"] {
    if dimen_reg(&format!("\\{name}")) < 0 {
      warn(&mut report, name, "negative");
    }
  }
  for name in ["stockwidth", "paperwidth", "textwidth"] {
    if dimen_reg(&format!("\\{name}")) <= 0 {
      warn(&mut report, name, "zero or negative");
    }
  }
  for name in ["spinemargin", "foremargin"] {
    if dimen_reg(&format!("\\{name}")) < 0 {
      warn(&mut report, name, "negative");
    }
  }
  for name in [
    "marginparsep",
    "marginparwidth",
    "stockheight",
    "paperheight",
    "textheight",
  ] {
    if dimen_reg(&format!("\\{name}")) <= 0 {
      warn(&mut report, name, "zero or negative");
    }
  }
  for name in [
    "uppermargin",
    "lowermargin",
    "headheight",
    "headsep",
    "footskip",
  ] {
    if dimen_reg(&format!("\\{name}")) < 0 {
      warn(&mut report, name, "negative");
    }
  }
  match algorithm {
    "classic" | "lines" | "nearest" => fit_text_height(algorithm)?,
    "fixed" => {},
    _ => report.push_str(
      "\\@memerror{Optional argument is not one of: classic, fixed, lines, or nearest. \
       I will assume the default}{\\@ehc}",
    ),
  }
  // `\setulmargins{\uppermargin}{*}{*}`: the lower margin is what the paper leaves
  let star = tex("*");
  let upper = tex("\\uppermargin");
  set_fill_size(
    &tex("\\paperheight"),
    &tex("\\textheight"),
    &upper,
    &star,
    &star,
  )?;
  set_dimen_reg("\\textheight", dimen_reg("\\@tempdimc"))?;
  set_dimen_reg("\\uppermargin", dimen_reg("\\@tempdima"))?;
  set_dimen_reg("\\lowermargin", dimen_reg("\\@tempdimb"))?;
  // The overflow checks (memoir.cls:1139-1206): each `\@tempdima` is what is left of a whole after its parts; more
  // than a point short is an error.
  let tolerance = -65536;
  let d = |name: &str| dimen_reg(&format!("\\{name}"));
  let shown = |sp: i64| Dimension::new(sp).to_string();
  let mut overflow = |left: i64, parts: &[&str], whole: &str| {
    if left < tolerance {
      let listed = parts
        .iter()
        .map(|p| format!("\\protect\\{p}\\space ({})", shown(d(p))))
        .collect::<Vec<_>>()
        .join(" and/or ");
      report.push_str(&format!(
        "\\@memerror{{{listed} are too large for \\protect\\{whole}\\space ({}) by {}}}{{\\@ehd}}",
        shown(d(whole)),
        shown(-left)
      ));
    }
  };
  overflow(
    d("stockwidth") - d("trimedge") - d("paperwidth"),
    &["paperwidth", "trimedge"],
    "stockwidth",
  );
  overflow(
    d("paperwidth") - d("foremargin") - d("textwidth") - d("spinemargin"),
    &["spinemargin", "textwidth", "foremargin"],
    "paperwidth",
  );
  overflow(
    d("stockheight") - d("trimtop") - d("paperheight"),
    &["paperheight", "trimtop"],
    "stockheight",
  );
  overflow(
    d("paperheight") - d("uppermargin") - d("textheight") - d("lowermargin"),
    &["uppermargin", "textheight", "lowermargin"],
    "paperheight",
  );
  overflow(
    d("uppermargin") - d("headheight") - d("headsep"),
    &["headheight", "headsep"],
    "uppermargin",
  );
  // the footskip test compares with zero, not the point of slack (memoir.cls:1198-1206)
  let footskip_left = d("lowermargin") - d("footskip");
  if footskip_left < 0 {
    report.push_str(&format!(
      "\\@memerror{{\\protect\\footskip\\space ({}) is too large for \\protect\\lowermargin\\space ({}) by {}}}{{\\@ehd}}",
      shown(d("footskip")),
      shown(d("lowermargin")),
      shown(-footskip_left)
    ));
  }
  set_dimen_reg("\\@tempdimb", tolerance)?;
  Ok(tex(report))
}

/// `\fixthelayout` (memoir.cls:1208-1227): hand the layout to LaTeX's own parameters — `\topmargin`, `\oddsidemargin`
/// and `\evensidemargin` count from the one-inch origin. Returns the tail that needs memoir's header and
/// margin-note machinery (`\fixheaderwidths`, the `\setmarginnotes` flag), which is not geometry.
fn fix_the_layout() -> Result<Tokens> {
  let one_inch = dimen_of(tex("1in"))?;
  let d = |name: &str| dimen_reg(&format!("\\{name}"));
  set_dimen_reg(
    "\\topmargin",
    d("trimtop") + d("uppermargin") - d("headsep") - d("headheight") - one_inch,
  )?;
  set_dimen_reg(
    "\\oddsidemargin",
    d("stockwidth") - d("trimedge") - d("paperwidth") + d("spinemargin") - one_inch,
  )?;
  set_dimen_reg(
    "\\evensidemargin",
    d("trimedge") + d("foremargin") - one_inch,
  )?;
  // `\mem@settopoint` (memoir.cls:609-610) drops the fraction of a point, and memoir's `fullptlayout` option makes it
  // `\@gobble`: memoir's own macro decides, run before the header widths are fixed (:1222-1224).
  Ok(tex(
    "\\mem@settopoint\\textwidth\\mem@settopoint\\oddsidemargin\\mem@settopoint\\evensidemargin\
     \\fixheaderwidths\\memifmacroused\\setmarginnotes{}{\\mem@autoadjust@marginparwidth}\\mem@reset@used@macros",
  ))
}

// --- ToC entries ---------------------------------------------------------------------------------------------------

/// What `\cft<level>font` contributes to a ToC entry's title: its tokens without the glue and breaks the author put
/// there to place the entry on a page (`\renewcommand*{\cftsectionfont}{\hfill\bfseries}`), which would otherwise
/// become a space or a fill in the title. memoir typesets the font in a `\leftskip` paragraph, where `\hfill` pushes
/// the title to the right margin; HTML has no such line to push on.
fn toc_entry_font(level: &str) -> Result<Tokens> {
  const LAYOUT_ONLY: &[&str] = &[
    "\\hfill",
    "\\hfil",
    "\\hss",
    "\\hfilneg",
    "\\vfill",
    "\\vfil",
    "\\vss",
    "\\par",
    "\\newline",
    "\\\\",
    "\\break",
    "\\linebreak",
    "\\nobreak",
    "\\newpage",
    "\\clearpage",
    "\\leavevmode",
    "\\null",
    "\\indent",
    "\\noindent",
    "\\quad",
    "\\qquad",
    "\\enskip",
    "\\enspace",
    "\\dotfill",
    "\\hrulefill",
    "\\relax",
  ];
  let cs = T_CS!(format!("\\cft{level}font"));
  let Some(defn) = lookup_definition(&cs)? else {
    return Ok(Tokens::new(Vec::new()));
  };
  let Some(ExpansionBody::Tokens(body)) = defn.get_expansion() else {
    // a closure or a macro with parameters: its call is all there is to give
    return Ok(Tokens::new(vec![cs]));
  };
  if defn
    .get_parameters()
    .is_some_and(|p| !p.get_parameters().is_empty())
  {
    return Ok(Tokens::new(vec![cs]));
  }
  Ok(Tokens::new(
    body
      .clone()
      .unlist()
      .into_iter()
      .filter(|t| {
        t.get_catcode() != Catcode::SPACE
          && !(t.get_catcode() == Catcode::CS && LAYOUT_ONLY.contains(&t.to_string().as_str()))
      })
      .collect(),
  ))
}

/// How a level's `\cft<level>formatpnum` boxes the page number and its `\cft<level>fillnum` fills the line: the book
/// and part levels differ from chapters and the `\newlistentry` levels (memoir.cls:7303-7312 for book,
/// :7374-7381 for part, :7446-7452 for chapter, :7231-7239 for the rest).
#[derive(Clone, Copy, PartialEq)]
enum CftKind {
  Book,
  Part,
  Chapter,
  Entry,
}

/// memoir's defaults for one level's `\cft…` commands: the font (and page-number font), the dot separator. Book and
/// part are large and bold with no dots (memoir.cls:7294-7300, :7364-7373), chapters bold with no dots (:7436-7442),
/// every `\newlistentry` level plain with dots (:7220-7225).
struct CftLevel {
  name:   &'static str,
  kind:   CftKind,
  font:   &'static str,
  dotsep: &'static str,
}

const CFT_LEVELS: &[CftLevel] = &[
  CftLevel {
    name:   "book",
    kind:   CftKind::Book,
    font:   "\\large\\bfseries",
    dotsep: "\\cftnodots",
  },
  CftLevel {
    name:   "part",
    kind:   CftKind::Part,
    font:   "\\large\\bfseries",
    dotsep: "\\cftnodots",
  },
  CftLevel {
    name:   "chapter",
    kind:   CftKind::Chapter,
    font:   "\\bfseries",
    dotsep: "\\cftnodots",
  },
  CftLevel {
    name:   "section",
    kind:   CftKind::Entry,
    font:   "\\normalfont",
    dotsep: "\\cftdotsep",
  },
  CftLevel {
    name:   "subsection",
    kind:   CftKind::Entry,
    font:   "\\normalfont",
    dotsep: "\\cftdotsep",
  },
  CftLevel {
    name:   "subsubsection",
    kind:   CftKind::Entry,
    font:   "\\normalfont",
    dotsep: "\\cftdotsep",
  },
  CftLevel {
    name:   "paragraph",
    kind:   CftKind::Entry,
    font:   "\\normalfont",
    dotsep: "\\cftdotsep",
  },
  CftLevel {
    name:   "subparagraph",
    kind:   CftKind::Entry,
    font:   "\\normalfont",
    dotsep: "\\cftdotsep",
  },
  CftLevel {
    name:   "figure",
    kind:   CftKind::Entry,
    font:   "\\normalfont",
    dotsep: "\\cftdotsep",
  },
  CftLevel {
    name:   "table",
    kind:   CftKind::Entry,
    font:   "\\normalfont",
    dotsep: "\\cftdotsep",
  },
];

/// Define `\<name>` as a macro with `params` (a prototype such as `{}` or empty) and a TeX body.
fn def_cft_macro(name: &str, params: &str, body: &str) -> Result<()> {
  let cs = T_CS!(format!("\\{name}"));
  let parameters = if params.is_empty() {
    None
  } else {
    parse_parameters(params, &cs, true)?
  };
  def_macro(cs, parameters, ExpansionBody::Tokens(tex(body)), None)
}

LoadDefinitions!({
  //----------------------------------------------------------------------------------------------------------------
  // Section-numbering depth and ToC depth (memoir.cls:7544-7565, 7631-7753)
  //----------------------------------------------------------------------------------------------------------------
  DefMacro!("\\@setclcnt Expanded Expanded", sub[(name, counter)] {
    set_division_counter(&name.to_string(), &counter.to_string())
  });
  // memoir.cls:7741-7746: the limit for either counter, kept at the highest level a document may ask for
  DefMacro!("\\maxtocdepth Expanded", sub[(name)] {
    set_division_counter(&name.to_string(), "tocdepth")
  });
  DefMacro!("\\maxsecnumdepth Expanded", sub[(name)] {
    set_both_division_counters(&name.to_string())
  });
  // memoir.cls:7747-7753: in the preamble both the depth and its limit, afterwards the depth alone. Both calls run
  // whatever the first one did, so an unknown name is reported once per call (pdfLaTeX: two errors in the preamble).
  DefMacro!("\\setsecnumdepth Expanded", sub[(name)] {
    let name = name.to_string();
    if x_equals(&T_CS!("\\@nodocument"), &T_CS!("\\relax")) {
      set_division_counter(&name, "secnumdepth")
    } else {
      set_both_division_counters(&name)
    }
  });
  // memoir.cls:7739
  DefMacro!("\\changetocdepth{}", "\\setcounter{tocdepth}{#1}");
  // memoir.cls:7631-7669. `\settocdepth@` does two things for a known division name: it writes
  // `\addtocontents{toc}{\changetocdepth{<division level>}}` (section 1, subsection 2, ...: `\@setclcnt`'s table), and
  // it sets `tocdepth` at once to `\toclevel@<name>` (:7644-7647), which `\newlistentry` made one less for the
  // numbered divisions (section 0, subsection 1, :7229 and :7458-7466; chapter, part and book agree with the
  // division level). pdfLaTeX's typeset ToC is the one the next run builds from the `.toc`, where the written
  // `\changetocdepth` has set the division level before the entries, and `\l@section` lists a section when
  // `tocdepth > 0` (:7172-7173): so `\settocdepth{subsection}` shows subsections. LaTeXML builds the ToC from the
  // document and reads `tocdepth` when `\tableofcontents` is digested (latex_constructs.pool.ltxml:731-740), with
  // no `.toc` to carry the written value, so the counter is set to the division level, the one that matches the
  // typeset ToC; `\toclevel@<name>` would list one level too few. A `\settocdepth` after `\tableofcontents` cannot
  // reach the ToC that is already built. The star form only adds a `\clearpage` in memoir (:7658-7663).
  DefMacro!("\\settocdepth OptionalMatch:* Expanded", sub[(_star, name)] {
    let name = name.to_string();
    match division_level(&name) {
      Some(level) => {
        SetCounter!("tocdepth", Number::new(level));
        Ok(Tokens::new(Vec::new()))
      },
      None => Ok(unknown_division(&name, "I'll ignore it. Type <return> and I'll continue.")),
    }
  });

  //----------------------------------------------------------------------------------------------------------------
  // The lists of contents (memoir.cls:7074-7135, 12149-12157)
  //----------------------------------------------------------------------------------------------------------------
  // memoir's lists are `\newlistof` macros taking a starred form (no `Contents` entry in the ToC itself). LaTeXML's
  // lists are built in post-processing and the heading entry has no counterpart, so both forms are the kernel's.
  DefMacro!("\\tableofcontents OptionalMatch:*", "\\lx@tableofcontents");
  DefMacro!("\\listoffigures OptionalMatch:*", "\\lx@listoffigures");
  DefMacro!("\\listoftables OptionalMatch:*", "\\lx@listoftables");
  // memoir.cls:7137: the right margin of a ToC line, storage only (HTML lines have no right margin to keep clear)
  DefMacro!("\\setrmarg{}", "\\def\\@tocrmarg{#1}");

  //----------------------------------------------------------------------------------------------------------------
  // ToC entry commands, one set per level (memoir.cls:7143-7456)
  //----------------------------------------------------------------------------------------------------------------
  def_cft_macro("cftparfillskip", "", "\\parfillskip=0pt plus1fil")?; // memoir.cls:7143
  for level in CFT_LEVELS {
    let n = level.name;
    def_cft_macro(&format!("cft{n}font"), "", level.font)?;
    def_cft_macro(&format!("cft{n}pagefont"), "", level.font)?;
    def_cft_macro(&format!("cft{n}dotsep"), "", level.dotsep)?;
    def_cft_macro(
      &format!("cft{n}leader"),
      "",
      &format!("{}\\cftdotfill{{\\cft{n}dotsep}}", level.font),
    )?;
    def_cft_macro(&format!("cft{n}afterpnum"), "", "")?;
    def_cft_macro(&format!("cft{n}formatpnumhook"), "{}", "")?;
    let (pnum_box, nobreak, group) = match level.kind {
      CftKind::Book | CftKind::Part => (
        format!("\\hss {{\\cft{n}pagefont #1}}"),
        "",
        level.kind == CftKind::Part,
      ),
      CftKind::Chapter | CftKind::Entry => {
        (format!("\\hfil\\cft{n}pagefont #1"), "\\nobreak", false)
      },
    };
    def_cft_macro(
      &format!("cft{n}formatpnum"),
      "{}",
      &format!("\\cft{n}formatpnumhook{{#1}}\\hb@xt@\\@pnumwidth{{{pnum_box}}}"),
    )?;
    let page_number = if group {
      format!("{{\\cft{n}formatpnum{{#1}}}}")
    } else {
      format!("\\cft{n}formatpnum{{#1}}")
    };
    def_cft_macro(
      &format!("cft{n}fillnum"),
      "{}",
      &format!("{{\\cft{n}leader}}{nobreak}{page_number}\\cft{n}afterpnum\\par"),
    )?;
    for storage in ["presnum", "aftersnum", "aftersnumb"] {
      def_cft_macro(&format!("cft{n}{storage}"), "", "")?;
    }
    // The ToC title of a division or caption is typeset in the entry's font (Perl `Base_Utility.pool.ltxml` L1107-1126
    // asks `\format@toctitle@font@<type>` for it), and so is its number: memoir sets both in the one group,
    // `{\cftchapterfont {#1}}` with `\chapternumberline{<number>}` inside (`\l@chapapp`, memoir.cls:7406; the
    // `\newlistentry` levels the same, :7196), where Perl asks `\fnum@tocfont@<type>` for the number
    // (`\lx@fnum@toc@@`, Base_Utility.pool.ltxml:1064-1067). `appendix` entries are chapter entries (`\l@appendix`,
    // memoir.cls:7417-7419).
    let mut types = vec![n];
    if n == "chapter" {
      types.push("appendix");
    }
    for ty in types {
      DefMacro!(T_CS!(format!("\\format@toctitle@font@{ty}")), None, {
        toc_entry_font(n)
      });
      DefMacro!(T_CS!(format!("\\fnum@tocfont@{ty}")), None, {
        toc_entry_font(n)
      });
    }
  }

  //----------------------------------------------------------------------------------------------------------------
  // Page layout (memoir.cls:763-866, 901-924, 1096-1206)
  //----------------------------------------------------------------------------------------------------------------
  DefMacro!("\\setfillsize{}{}{}{}{}", sub[(total, c, l, r, ratio)] {
    let ok = set_fill_size(
      &total.into_tokens_result()?,
      &c.into_tokens_result()?,
      &l.into_tokens_result()?,
      &r.into_tokens_result()?,
      &ratio.into_tokens_result()?,
    )?;
    Ok(if ok { Tokens::new(Vec::new()) } else { ambiguous_fill_size() })
  });
  // memoir.cls:901-908: the inner and outer margins around a text block `\textwidth` wide, counted from the paper
  // without the binding
  DefMacro!("\\setlrmarginsandblock{}{}{}", sub[(l, r, ratio)] {
    let paperwidth = || dimen_reg("\\paperwidth");
    let binding = dimen_reg("\\binding");
    set_dimen_reg("\\paperwidth", paperwidth() - binding)?;
    let ok = set_fill_size(
      &tex("\\paperwidth"),
      &tex("*"),
      &l.into_tokens_result()?,
      &r.into_tokens_result()?,
      &ratio.into_tokens_result()?,
    )?;
    set_dimen_reg("\\textwidth", dimen_reg("\\@tempdimc"))?;
    set_dimen_reg("\\spinemargin", dimen_reg("\\@tempdima"))?;
    set_dimen_reg("\\foremargin", dimen_reg("\\@tempdimb"))?;
    set_dimen_reg("\\paperwidth", paperwidth() + binding)?;
    set_dimen_reg("\\spinemargin", dimen_reg("\\spinemargin") + binding)?;
    Ok(if ok { Tokens::new(Vec::new()) } else { ambiguous_fill_size() })
  });
  // memoir.cls:919-924: the upper and lower margins around a text block `\textheight` high
  DefMacro!("\\setulmarginsandblock{}{}{}", sub[(u, l, ratio)] {
    let ok = set_fill_size(
      &tex("\\paperheight"),
      &tex("*"),
      &u.into_tokens_result()?,
      &l.into_tokens_result()?,
      &ratio.into_tokens_result()?,
    )?;
    set_dimen_reg("\\textheight", dimen_reg("\\@tempdimc"))?;
    set_dimen_reg("\\uppermargin", dimen_reg("\\@tempdima"))?;
    set_dimen_reg("\\lowermargin", dimen_reg("\\@tempdimb"))?;
    Ok(if ok { Tokens::new(Vec::new()) } else { ambiguous_fill_size() })
  });
  DefMacro!("\\checkthelayout[Default:classic]", sub[(algorithm)] {
    check_the_layout(&algorithm.map_or_else(|| "classic".to_string(), |a| a.to_string()))
  });
  DefMacro!("\\fixthelayout", { fix_the_layout() });
  // memoir.cls:1372-1376, without `\typeoutlayout`, which only writes the lengths to the log
  DefMacro!(
    "\\checkandfixthelayout[Default:classic]",
    "\\checkthelayout[#1]\\fixthelayout"
  );
});
