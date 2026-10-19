//! epic.sty (Sunil Podar, 1986-1990) — enhancements to the `picture` environment.
//!
//! No Perl binding exists; native binding, OXIDIZED_DESIGN #588(i). epic draws its lines out of
//! dots, dashes and line-font characters placed with `\raise\hbox to\z@{\hskip…}` (epic.sty:162-174),
//! which a raw load would turn into text boxes, not drawing; here each command is the drawing it makes,
//! in the picture model the kernel's `\line`/`\circle` emit (`ltx:line`, `ltx:rect`, `ltx:circle`, in
//! picture coordinates, latex_constructs/sect13.rs):
//! `\dottedline[char]{gap}(x1,y1)…(xn,yn)` (:233-266) puts per segment `N+1` dots, `N` from epic's
//! `\sqrtandstuff` (:190-217) — the default dot is a filled `\@wholewidth` square (`\picsquare@bl`,
//! :118) centred on its position, as epic centres a dot with height (:249-256), any other one is `\put`
//! there; `\dashline[stretch]{dash}[dotgap](…)`
//! (:274-357) is per segment a line with epic's dash count and step as a dash pattern (dotted dashes
//! when `dotgap` is given); `\drawline[stretch](…)` (:418-522) a polyline (a negative stretch drops
//! characters: drawn dashed, 10pt dashes stretched by `2(s+50)`, as eepic draws it, eepic.sty:213-227);
//! `\jput` (:162-174) is `\put`, joining successive points inside `dottedjoin`, `dashjoin` and `drawjoin`
//! (:144-159); `\grid` (:80-110), `\matrixput` (:49-69), `\multiputlist` (:29-43), `\picsquare` (:114-115)
//! and `\putfile` (:540-559) place their lines, objects and labels with `\put`. Coordinates are numbers
//! of `\unitlength`s (registers allowed, as epic multiplies them by `\unitlength`). eepic (eepic_sty.rs)
//! changes the dots and the negative-stretch drawlines and adds its own objects through this module.
//! Witnesses (errors in the default configuration, before → after): 2610.20583 (1 → 0), 2610.20862 (1 → 0).
use latexml_core::common::numeric_ops::UNITY_F64;

use crate::prelude::*;

/// A pt length as CSS px, as the picture elements carry them.
pub(crate) fn px(pt: f64) -> String {
  let v = Dimension::new((pt * UNITY_F64).round() as i64).px_value(Some(2));
  if v == v.trunc() {
    format!("{}", v as i64)
  } else {
    let s = format!("{v:.2}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
  }
}

/// `<coord>\unitlength` evaluated as a TeX dimension, in pt: a number, a decimal or a register.
pub(crate) fn units_to_pt(coord: Tokens) -> Result<f64> {
  let mut toks = coord.unlist();
  if toks.iter().all(|t| t.get_catcode() == Catcode::SPACE) {
    return Ok(0.0);
  }
  toks.push(T_CS!("\\unitlength"));
  // Read in a mouth of its own: the dimension's scan ends with the coordinate.
  reading_from_mouth(Mouth::default(), move || -> Result<f64> {
    unread(Tokens::new(toks));
    let dim = read_dimension()?;
    while read_token()?.is_some() {}
    Ok(dim.pt_value(None))
  })
}

/// A point list `(x1,y1)(x2,y2)…` from the input, in pt; spaces and line ends between the pairs are
/// skipped, as epic's `\@ifnextchar (` recursion skips them.
pub(crate) fn read_points() -> Result<Vec<(f64, f64)>> {
  let mut points = Vec::new();
  loop {
    skip_spaces()?;
    if !if_next(T_OTHER!("("))? {
      break;
    }
    read_token()?;
    let x = read_until(&Tokens!(T_OTHER!(",")))?.unwrap_or_default();
    let y = read_until(&Tokens!(T_OTHER!(")")))?.unwrap_or_default();
    let x = units_to_pt(x)?;
    let y = units_to_pt(y)?;
    points.push((x, y));
  }
  Ok(points)
}

/// The optional `[…]` argument, if the next token opens one (no space skipped, as `\@ifnextchar[`
/// right after a control word).
pub(crate) fn read_bracket() -> Result<Option<Tokens>> {
  if if_next(T_OTHER!("["))? {
    read_token()?;
    Ok(Some(
      read_until(&Tokens!(T_OTHER!("]")))?.unwrap_or_default(),
    ))
  } else {
    Ok(None)
  }
}

/// A string kept in the state, or empty.
pub(crate) fn state_string(key: &str) -> String {
  lookup_value(key).map(|v| v.to_string()).unwrap_or_default()
}

pub(crate) fn wholewidth_pt() -> Result<f64> {
  Ok(match lookup_register("\\@wholewidth", Vec::new())? {
    Some(RegisterValue::Dimension(d)) => d.pt_value(None),
    _ => 0.4,
  })
}

/// epic's `\sqrtandstuff` (epic.sty:190-217): the number of `gap`s along a segment `(dx,dy)`.
pub(crate) fn sqrt_and_stuff(dx: f64, dy: f64, gap: f64) -> usize {
  if gap <= 0.0 {
    return 1;
  }
  let x = ((dx.abs() + gap / 2.0) / gap).floor() as i64;
  let y = ((dy.abs() + gap / 2.0) / gap).floor() as i64;
  let (min, max) = if x < y { (x, y) } else { (y, x) };
  let n = if x < 2 && y < 2 {
    x + y
  } else if max > 0 {
    max + ((457 * min * min / max + 750) / 1000)
  } else {
    0
  };
  n.max(1) as usize
}

/// One picture element: its qualified name and attributes.
pub(crate) type Element = (String, HashMap<String, String>);

pub(crate) fn element(qname: &str, attrs: &[(&str, String)]) -> Element {
  (
    qname.to_string(),
    attrs
      .iter()
      .map(|(k, v)| (k.to_string(), v.clone()))
      .collect(),
  )
}

pub(crate) fn polyline(
  points: &[(f64, f64)],
  width: f64,
  dash: Option<String>,
  fill: Option<&str>,
) -> Element {
  let pts = points
    .iter()
    .map(|(x, y)| format!("{},{}", px(*x), px(*y)))
    .collect::<Vec<_>>()
    .join(" ");
  let mut attrs = vec![
    ("points", pts),
    ("stroke", "#000000".to_string()),
    ("stroke-width", format!("{width}")),
    ("fill", fill.unwrap_or("none").to_string()),
  ];
  if let Some(d) = dash {
    attrs.push(("stroke-dasharray", d));
  }
  element("ltx:line", &attrs)
}

/// The dot positions of a dotted segment (epic.sty:243-262): `N+1` evenly spaced points.
pub(crate) fn dot_positions(p1: (f64, f64), p2: (f64, f64), gap: f64) -> Vec<(f64, f64)> {
  let (dx, dy) = (p2.0 - p1.0, p2.1 - p1.1);
  let n = sqrt_and_stuff(dx, dy, gap);
  (0..=n)
    .map(|i| {
      (
        p1.0 + dx * i as f64 / n as f64,
        p1.1 + dy * i as f64 / n as f64,
      )
    })
    .collect()
}

/// How a dot is drawn: epic's `\picsquare@bl` square, or eepic's round tpic pen.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Dot {
  Square,
  Round,
}

pub(crate) fn dot(at: (f64, f64), kind: Dot, width: f64) -> Element {
  match kind {
    // epic.sty:249-256: a dot with height is centred on its position (`\picsquare@bl` included).
    Dot::Square => element("ltx:rect", &[
      ("x", px(at.0 - width / 2.0)),
      ("y", px(at.1 - width / 2.0)),
      ("width", px(width)),
      ("height", px(width)),
      ("fill", "#000000".to_string()),
      ("stroke", "none".to_string()),
    ]),
    Dot::Round => element("ltx:circle", &[
      ("x", px(at.0)),
      ("y", px(at.1)),
      ("r", px(width / 2.0)),
      ("fill", "#000000".to_string()),
      ("stroke", "none".to_string()),
    ]),
  }
}

/// The dashes of a segment (epic.sty:280-352): the dash vector length and the step between dash starts,
/// or `None` for a segment too short to dash.
pub(crate) fn dash_pattern(dx: f64, dy: f64, dash: f64, width: f64, stretch: f64) -> (f64, f64) {
  let len = (dx * dx + dy * dy).sqrt();
  // epic.sty:290: the count from the segment's legs, as `\sqrtandstuff` estimates the length.
  let mut n = sqrt_and_stuff(dx, dy, (dash - width).max(1e-6)).max(3) as f64;
  let a = len / n;
  n -= ((2.0 * n * width + dash / 2.0) / dash).floor();
  if stretch != 0.0 && stretch >= -100.0 {
    n = (n * (100.0 + stretch) / 100.0).floor();
  }
  let k = (n / 2.0).floor();
  let step = if k > 0.0 { (len - a) / k } else { len - a };
  (a, step)
}

/// A dashed segment: solid dashes as a dash pattern on the segment's line, or (with a dot gap) the
/// dots of each dash.
pub(crate) fn dashed_segment(
  p1: (f64, f64),
  p2: (f64, f64),
  dash: f64,
  dotgap: Option<f64>,
  stretch: f64,
  width: f64,
  dot_kind: Dot,
) -> Vec<Element> {
  let (dx, dy) = (p2.0 - p1.0, p2.1 - p1.1);
  let len = (dx * dx + dy * dy).sqrt();
  if len <= 0.0 {
    return Vec::new();
  }
  let (a, step) = dash_pattern(dx, dy, dash, width, stretch);
  match dotgap {
    None => vec![polyline(
      &[p1, p2],
      width,
      Some(format!("{} {}", px(a), px((step - a).max(0.0)))),
      None,
    )],
    Some(gap) => {
      let (ux, uy) = (dx / len, dy / len);
      let mut out = Vec::new();
      let mut start = 0.0;
      while start + a <= len + 1e-6 {
        let s = (p1.0 + ux * start, p1.1 + uy * start);
        let e = (p1.0 + ux * (start + a), p1.1 + uy * (start + a));
        out.extend(
          dot_positions(s, e, gap)
            .into_iter()
            .map(|q| dot(q, dot_kind, width)),
        );
        if step <= 0.0 {
          break;
        }
        start += step;
      }
      out
    },
  }
}

pub(crate) fn elements_to_stored(elements: Vec<Element>) -> Stored {
  Stored::VecDequeStored(
    elements
      .into_iter()
      .map(|(qname, mut attrs)| {
        attrs.insert("_qname".to_string(), qname);
        Stored::HashString(attrs)
      })
      .collect::<VecDeque<_>>(),
  )
}

/// Whether eepic is loaded: its round dots and dashed negative-stretch drawlines replace epic's.
pub(crate) fn eepic_active() -> bool { lookup_bool("eepic.sty_loaded") }

pub(crate) fn dot_kind_and_width() -> Result<(Dot, f64)> {
  let w = wholewidth_pt()?;
  // eepic's dots are tpic `dt` lines, their pen 2 milli-inch over `\@gphlinewidth` (eepic.sty:104-145).
  Ok(if eepic_active() {
    (Dot::Round, w + 2.0 * 72.27 / 1000.0)
  } else {
    (Dot::Square, w)
  })
}

/// `\lx@epic@draw{elements}`: the elements a drawing command computed, inserted where it stands.
#[rustfmt::skip]
pub(crate) fn define_draw() -> Result<()> {
  DefConstructor!("\\lx@epic@draw{}",
  sub[document, _args, props] {
    if let Some(Stored::VecDequeStored(elements)) = props.get("elements") {
      for element in elements {
        if let Stored::HashString(attrs) = element {
          let mut attrs = attrs.clone();
          let qname = attrs.remove("_qname").unwrap_or_default();
          document.insert_element(&qname, Vec::new(), Some(attrs))?;
        }
      }
    }
  },
  sizer => 0,
  reversion => "",
  properties => sub[args] {
    let key = args[0].as_ref().map(|d| d.to_string()).unwrap_or_default();
    Ok(stored_map!("elements" => lookup_value(&format!("lx@epic@elements@{}", key.trim()))
      .unwrap_or(Stored::VecDequeStored(VecDeque::new()))))
  });
  Ok(())
}

/// Stash computed elements and return the `\lx@epic@draw` call that inserts them.
pub(crate) fn draw_call(elements: Vec<Element>) -> Result<Tokens> {
  let n = match lookup_value("lx@epic@draw@count") {
    Some(Stored::Number(n)) => n.value_of() + 1,
    _ => 1,
  };
  assign_value(
    "lx@epic@draw@count",
    Stored::Number(Number::new(n)),
    Some(Scope::Global),
  );
  assign_value(
    &format!("lx@epic@elements@{n}"),
    elements_to_stored(elements),
    Some(Scope::Global),
  );
  Ok(mouth::tokenize_internal(TeXString::assembled(format!(
    "\\lx@epic@draw{{{n}}}"
  ))))
}

/// The dotted polyline through `points` with dots `gap` apart, or `char` `\put` at each dot.
pub(crate) fn dotted_line(points: &[(f64, f64)], gap: f64, char: Option<Tokens>) -> Result<Tokens> {
  let unit = units_to_pt(mouth::tokenize_internal("1"))?;
  let mut positions = Vec::new();
  for seg in points.windows(2) {
    positions.extend(dot_positions(seg[0], seg[1], gap));
  }
  match char {
    Some(c) if !c.is_empty() => {
      // epic.sty:253-256: the dot character at each position, centred when it has height.
      let mut out = Vec::new();
      for (x, y) in positions {
        out.extend(
          mouth::tokenize_internal(TeXString::assembled(format!(
            "\\put({},{}){{\\makebox(0,0){{",
            x / unit,
            y / unit
          )))
          .unlist(),
        );
        out.extend(c.clone().unlist());
        out.extend([T_END!(), T_END!()]);
      }
      Ok(Tokens::new(out))
    },
    _ => {
      let (kind, width) = dot_kind_and_width()?;
      draw_call(positions.into_iter().map(|p| dot(p, kind, width)).collect())
    },
  }
}

pub(crate) fn dash_line(
  points: &[(f64, f64)],
  dash: f64,
  dotgap: Option<f64>,
  stretch: f64,
) -> Result<Tokens> {
  let width = wholewidth_pt()?;
  let (kind, dotw) = dot_kind_and_width()?;
  let mut elements = Vec::new();
  for seg in points.windows(2) {
    elements.extend(dashed_segment(
      seg[0],
      seg[1],
      dash,
      dotgap,
      stretch,
      if dotgap.is_some() { dotw } else { width },
      kind,
    ));
  }
  draw_call(elements)
}

pub(crate) fn draw_line(points: &[(f64, f64)], stretch: f64) -> Result<Tokens> {
  if points.len() < 2 {
    return Ok(Tokens::default());
  }
  if stretch < 0.0 {
    // eepic.sty:213-227: `\@dashline[2(s+50)]{10pt}`.
    return dash_line(points, 10.0, None, 2.0 * (stretch + 50.0));
  }
  draw_call(vec![polyline(points, wholewidth_pt()?, None, None)])
}

fn number_arg(tokens: Option<Tokens>, default: f64) -> f64 {
  tokens
    .map(|t| t.to_string())
    .and_then(|s| s.trim().parse().ok())
    .unwrap_or(default)
}

#[rustfmt::skip]
LoadDefinitions!({
  define_draw()?;
  // epic.sty:144-159, :272, :416: the stretch defaults and the join state.
  RawTeX!(r"\def\dashlinestretch{0}\def\drawlinestretch{0}
\newif\if@jointhem\newif\if@firstpoint");

  // epic.sty:233-266 `\dottedline[char]{gap}(points)`
  DefMacro!("\\dottedline", sub[_args] {
    let char = read_bracket()?;
    let gap = units_to_pt(read_arg(ExpansionLevel::Off)?)?;
    let points = read_points()?;
    dotted_line(&points, gap, char)
  });
  // epic.sty:274-357 `\dashline[stretch]{dash}[dotgap](points)`
  DefMacro!("\\dashline", sub[_args] {
    let stretch = number_arg(read_bracket()?, number_arg(Some(Expand!(T_CS!("\\dashlinestretch"))), 0.0));
    let dash = units_to_pt(read_arg(ExpansionLevel::Off)?)?;
    let dotgap = match read_bracket()? { Some(g) => Some(units_to_pt(g)?), None => None };
    let points = read_points()?;
    dash_line(&points, dash, dotgap, stretch)
  });
  // epic.sty:418-522 `\drawline[stretch](points)`
  DefMacro!("\\drawline", sub[_args] {
    let stretch = number_arg(read_bracket()?, number_arg(Some(Expand!(T_CS!("\\drawlinestretch"))), 0.0));
    let points = read_points()?;
    draw_line(&points, stretch)
  });

  // epic.sty:144-174: `\jput`, and the joins (the segment from the previous `\jput` point).
  DefMacro!("\\lx@epic@join SkipSpaces Match:( Until:, Until:)", sub[(_open, x, y)] {
    let joining = Expand!(mouth::tokenize_internal(r"\if@jointhem T\else F\fi")).to_string().trim() == "T";
    let p = (units_to_pt(x)?, units_to_pt(y)?);
    if !joining {
      return Ok(Tokens::default());
    }
    let previous = state_string("lx@epic@join@point").split_once(',')
      .and_then(|(a, b)| Some((a.parse::<f64>().ok()?, b.parse::<f64>().ok()?)));
    assign_value("lx@epic@join@point", Stored::String(pin(format!("{},{}", p.0, p.1))), Some(Scope::Global));
    let Some(q) = previous else { return Ok(Tokens::default()) };
    let kind = state_string("lx@epic@join@kind");
    let param = |k: &str| state_string(&format!("lx@epic@join@{k}"));
    match kind.as_str() {
      "dotted" => {
        let char = param("char");
        let char = if char.is_empty() { None } else { Some(mouth::tokenize_internal(TeXString::assembled(char))) };
        dotted_line(&[q, p], param("gap").parse().unwrap_or(1.0), char)
      },
      "dash" => dash_line(&[q, p], param("dash").parse().unwrap_or(1.0),
        param("dotgap").parse().ok(), param("stretch").parse().unwrap_or(0.0)),
      _ => draw_line(&[q, p], param("stretch").parse().unwrap_or(0.0)),
    }
  });
  RawTeX!(r"\long\def\jput(#1,#2)#3{\lx@epic@join(#1,#2)\put(#1,#2){#3}}");
  DefPrimitive!("\\lx@epic@joinstart{}", sub[(kind)] {
    let kind = kind.to_string();
    let kind = kind.trim();
    assign_value("lx@epic@join@kind", Stored::String(pin(kind)), None);
    assign_value("lx@epic@join@point", Stored::String(pin("")), Some(Scope::Global));
    match kind {
      "dotted" => {
        let char = read_bracket()?.map(|t| t.untex().to_string()).unwrap_or_default();
        let gap = units_to_pt(read_arg(ExpansionLevel::Off)?)?;
        assign_value("lx@epic@join@char", Stored::String(pin(char)), None);
        assign_value("lx@epic@join@gap", Stored::String(pin(format!("{gap}"))), None);
      },
      "dash" => {
        let stretch = number_arg(read_bracket()?, number_arg(Some(Expand!(T_CS!("\\dashlinestretch"))), 0.0));
        let dash = units_to_pt(read_arg(ExpansionLevel::Off)?)?;
        let dotgap = match read_bracket()? { Some(g) => format!("{}", units_to_pt(g)?), None => String::new() };
        assign_value("lx@epic@join@stretch", Stored::String(pin(format!("{stretch}"))), None);
        assign_value("lx@epic@join@dash", Stored::String(pin(format!("{dash}"))), None);
        assign_value("lx@epic@join@dotgap", Stored::String(pin(dotgap)), None);
      },
      _ => {
        let stretch = number_arg(read_bracket()?, number_arg(Some(Expand!(T_CS!("\\drawlinestretch"))), 0.0));
        assign_value("lx@epic@join@stretch", Stored::String(pin(format!("{stretch}"))), None);
      },
    }
  });
  RawTeX!(r"\def\dottedjoin{\@jointhemtrue\lx@epic@joinstart{dotted}}\def\enddottedjoin{\@jointhemfalse}
\def\dashjoin{\@jointhemtrue\lx@epic@joinstart{dash}}\def\enddashjoin{\@jointhemfalse}
\def\drawjoin{\@jointhemtrue\lx@epic@joinstart{draw}}\def\enddrawjoin{\@jointhemfalse}");

  // epic.sty:29-43 `\multiputlist(x,y)(dx,dy)[pos]{a,b,…}`
  DefMacro!("\\multiputlist", sub[_args] {
    let pts = read_points()?;
    let pos = read_bracket()?.map(|t| t.untex().to_string()).unwrap_or_default();
    let list = read_arg(ExpansionLevel::Off)?;
    let unit = units_to_pt(mouth::tokenize_internal("1"))?;
    let (Some(&(x, y)), Some(&(dx, dy))) = (pts.first(), pts.get(1)) else { return Ok(Tokens::default()) };
    let mut out = Vec::new();
    let mut items: Vec<Vec<Token>> = vec![Vec::new()];
    let mut depth = 0usize;
    for t in list.unlist() {
      match t.get_catcode() { Catcode::BEGIN => depth += 1, Catcode::END => depth = depth.saturating_sub(1), _ => {} }
      if depth == 0 && t.get_catcode() == Catcode::OTHER && t.with_str(|s| s == ",") {
        items.push(Vec::new());
      } else if let Some(i) = items.last_mut() { i.push(t); }
    }
    for (k, item) in items.into_iter().enumerate() {
      out.extend(mouth::tokenize_internal(TeXString::assembled(format!(
        "\\put({},{}){{\\makebox(0,0)[{pos}]{{", (x + k as f64 * dx) / unit, (y + k as f64 * dy) / unit))).unlist());
      out.extend(item);
      out.extend([T_END!(), T_END!()]);
    }
    Ok(Tokens::new(out))
  });
  // epic.sty:49-69
  RawTeX!(r"\long\def\matrixput(#1,#2)(#3,#4)#5(#6,#7)#8#9{\multiput(#1,#2)(#6,#7){#8}{\multiput(0,0)(#3,#4){#5}{#9}}}");
  // epic.sty:80-110 `\grid(w,h)(dx,dy)[x0,y0]`
  DefMacro!("\\grid", sub[_args] {
    let pts = read_points()?;
    let labels = read_bracket()?;
    let unit = units_to_pt(mouth::tokenize_internal("1"))?;
    let (Some(&(w, h)), Some(&(dx, dy))) = (pts.first(), pts.get(1)) else { return Ok(Tokens::default()) };
    let (w, h, dx, dy) = (w / unit, h / unit, dx / unit, dy / unit);
    let mut s = format!("\\makebox({w},{h})[bl]{{");
    if dx > 0.0 {
      let nx = (w / dx + 1e-9).floor() as i64;
      for i in 0..=nx { s += &format!("\\put({},0){{\\line(0,1){{{h}}}}}", i as f64 * dx); }
    }
    if dy > 0.0 {
      let ny = (h / dy + 1e-9).floor() as i64;
      for j in 0..=ny { s += &format!("\\put(0,{}){{\\line(1,0){{{w}}}}}", j as f64 * dy); }
    }
    if let Some(l) = labels {
      let l = l.untex().to_string();
      let (x0, y0) = l.split_once(',').unwrap_or((l.as_str(), "0"));
      let x0: f64 = x0.trim().parse().unwrap_or(0.0);
      let y0: f64 = y0.trim().parse().unwrap_or(0.0);
      if dx > 0.0 {
        for i in 0..=((w / dx + 1e-9).floor() as i64) {
          let v = (x0 + i as f64 * dx) as i64;
          let x = i as f64 * dx;
          s += &format!("\\put({x},0){{\\makebox(0,-2)[t]{{{v}}}}}\\put({x},{h}){{\\makebox(0,2)[b]{{{v}}}}}");
        }
      }
      if dy > 0.0 {
        for j in 0..=((h / dy + 1e-9).floor() as i64) {
          let v = (y0 + j as f64 * dy) as i64;
          let y = j as f64 * dy;
          s += &format!("\\put(0,{y}){{\\makebox(-2,0)[r]{{{v}}}}}\\put({w},{y}){{\\makebox(2,0)[l]{{{v}}}}}");
        }
      }
    }
    s += "}";
    Ok(mouth::tokenize_internal(TeXString::assembled(s)))
  });
  // epic.sty:114-118: a filled `\@wholewidth` square, centred / at its bottom-left corner.
  DefMacro!("\\picsquare", sub[_args] {
    let w = wholewidth_pt()?;
    draw_call(vec![dot((-w / 2.0, -w / 2.0), Dot::Square, w)])
  });
  DefMacro!("\\picsquare@bl", sub[_args] {
    let w = wholewidth_pt()?;
    draw_call(vec![dot((0.0, 0.0), Dot::Square, w)])
  });
  // epic.sty:540-559, as the package defines it.
  RawTeX!(r"\long\def\splittwoargs#1 #2 {(#1,#2)}
\newif\if@stillmore\newread\@datafile
\long\def\putfile#1#2{\openin\@datafile = #1
\@stillmoretrue
\loop\ifeof\@datafile\relax\else\read\@datafile to\@dataline\fi
\ifeof\@datafile\@stillmorefalse\else\ifx\@dataline\@empty\relax\else
\expandafter\expandafter\expandafter\put\expandafter\splittwoargs\@dataline{#2}\fi\fi
\if@stillmore\repeat\closein\@datafile}");
});
