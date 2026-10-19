//! dynkin-diagrams.sty (2024/12/04, B. McKay) — Dynkin and Coxeter diagrams drawn with TikZ.
//!
//! No Perl binding exists; native binding, OXIDIZED_DESIGN #588(s). `\dynkin[keys]{series}[twist]{spec}`
//! (dynkin-diagrams.sty:97) draws the diagram of a root system: roots of a series A–I placed along a
//! line, D's two last roots forking at ±60°, E's root 2 above root 4 (:4313-5126), bonds single,
//! double or triple with an arrow toward the short root (B: to root n, C: to n−1, F4: 2→3, G2: to 1),
//! an indefinite bond (`.` in the spec) a third solid, a third dotted, a third solid at 1.6 edge
//! lengths (:767), roots marked `*` (filled), `o` (open), `O` (double), `x`/`X` (crossed), `t` (open,
//! crossed) (:680-705); `Coxeter` draws single bonds labelled 4 (B, F), 5 (H) or the gonality (I);
//! `fold` folds the diagram in two, joining the paired roots by grey bars (`\dynkin@fold`, :4255,
//! `\ESixTwoPly`, :4870); `labels={…}` labels the roots. The package computes all this through
//! pgfkeys, xparse argument processors and xstring loops; here the spec is read and the diagram
//! computed natively and drawn as the TikZ picture the package draws (`\tikz[baseline=(origin.base)]`
//! in text, the commands alone inside a `tikzpicture`, :97-123), with the package's key defaults (its
//! options included: `\usepackage[edge length=1cm]{dynkin-diagrams}`). The package itself is loaded
//! too, as its `\dynkin`'s fallback: a diagram outside this set (Satake codes, twisted and affine
//! series, Kac labels, ply 3 or 4, the root-mark drawing API) is drawn by the package's own code.
//! Witnesses (errors in the default configuration, before → after): 2610.18170 (1 → 0; folds, marks, labels), 2610.19203 (5 → 0; Coxeter diagrams).
use crate::prelude::*;

const SUPPORTED_KEYS: &[&str] = &[
  "edge length",
  "edge-length",
  "edgelength",
  "root radius",
  "root-radius",
  "rootradius",
  "fold radius",
  "fold-radius",
  "foldradius",
  "fold",
  "Coxeter",
  "gonality",
  "labels",
  "label",
];

/// A length key's value (`1.5cm`, `.35cm`, `\dynkin@edge@length`), in pt, when it is a TeX dimension
/// (a decimal and a unit, or a register). The keys are `.estore in` values that pgfmath evaluates
/// (dynkin-diagrams.sty:2171, :2352), so an expression (`2*.35cm`) is `None`: the package's own
/// `\dynkin` draws it.
fn dimension_pt(tokens: Tokens) -> Result<Option<f64>> {
  let tokens = do_expand(tokens)?;
  let toks = tokens.clone().unlist();
  let solid: Vec<&Token> = toks
    .iter()
    .filter(|t| t.get_catcode() != Catcode::SPACE)
    .collect();
  let register = solid.len() == 1 && solid[0].is_executable();
  if !register && !is_decimal_dimension(&tokens.clone().untex()) {
    return Ok(None);
  }
  // Read in a mouth of its own: the dimension's scan ends with the value.
  reading_from_mouth(Mouth::default(), move || -> Result<Option<f64>> {
    unread(tokens);
    let d = read_dimension()?;
    while read_token()?.is_some() {}
    Ok(Some(d.pt_value(None)))
  })
}

/// `[+-]<decimal> <unit>`: a dimension `\dimen` reads whole.
fn is_decimal_dimension(text: &str) -> bool {
  let t = text.trim();
  let t = t.trim_start_matches(['+', '-', ' ']);
  let number_end = t
    .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == ','))
    .unwrap_or(t.len());
  let (number, unit) = t.split_at(number_end);
  number.chars().any(|c| c.is_ascii_digit())
    && [
      "pt", "pc", "in", "bp", "cm", "mm", "dd", "cc", "sp", "em", "ex", "px",
    ]
    .contains(&unit.trim())
}

fn kv(tokens: &[Token]) -> Vec<(String, String)> {
  let mut items: Vec<Vec<Token>> = vec![Vec::new()];
  let mut depth = 0usize;
  for t in tokens {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ => {},
    }
    if depth == 0 && t.get_catcode() == Catcode::OTHER && t.with_str(|s| s == ",") {
      items.push(Vec::new());
    } else if let Some(i) = items.last_mut() {
      i.push(*t);
    }
  }
  items
    .into_iter()
    .filter_map(|item| {
      let s = Tokens::new(item).untex().to_string();
      let s = s.trim();
      if s.is_empty() {
        return None;
      }
      let (k, v) = s.split_once('=').unwrap_or((s, ""));
      let v = v.trim();
      let v = v
        .strip_prefix('{')
        .and_then(|v| v.strip_suffix('}'))
        .unwrap_or(v);
      Some((k.trim().to_string(), v.trim().to_string()))
    })
    .collect()
}

#[derive(Clone, Copy, PartialEq)]
enum Bond {
  Single,
  Double,
  Triple,
}

struct Diagram {
  /// Root marks, in root order.
  marks:      Vec<char>,
  pos:        Vec<(f64, f64)>,
  /// (from, to, bond, arrow toward `to`, indefinite, Coxeter label)
  edges:      Vec<(usize, usize, Bond, bool, bool, Option<String>)>,
  /// Arcs of a fold: (start root, start angle, end angle).
  arcs:       Vec<(usize, f64, f64)>,
  bars:       Vec<(usize, usize)>,
  label_dirs: Vec<&'static str>,
}

/// The marks and the indefinite edges (between string positions) of a spec (:2772-2830).
fn parse_spec(series: char, spec: &str, coxeter: bool) -> Option<(Vec<char>, Vec<bool>)> {
  let spec = spec.trim();
  let expanded: String = if spec.is_empty() {
    match series {
      'A' => "**.**".into(),
      'B' | 'C' => {
        if coxeter {
          "***.***".into()
        } else {
          "**.***".into()
        }
      },
      'D' => "**.****".into(),
      'H' | 'I' => "**".into(),
      _ => return None,
    }
  } else if let Ok(n) = spec.parse::<usize>() {
    "*".repeat(n)
  } else {
    spec.into()
  };
  let mut marks = Vec::new();
  let mut indefinite = Vec::new();
  let mut chars = expanded.chars().peekable();
  let mut pending_dot = false;
  while let Some(c) = chars.next() {
    match c {
      '*' | 'o' | 'O' | 'x' | 'X' | 't' => {
        let mut n = String::new();
        while let Some(d) = chars.peek().filter(|d| d.is_ascii_digit()) {
          n.push(*d);
          chars.next();
        }
        let count = n.parse::<usize>().unwrap_or(1);
        for _ in 0..count {
          if !marks.is_empty() {
            indefinite.push(pending_dot);
          }
          pending_dot = false;
          marks.push(c);
        }
      },
      '.' => pending_dot = true,
      ' ' => {},
      _ => return None,
    }
  }
  Some((marks, indefinite))
}

/// The diagram of `series` with these marks (unfolded), coordinates in pt.
fn layout(
  series: char,
  marks: Vec<char>,
  indefinite: &[bool],
  len: f64,
  coxeter: bool,
  gonality: &str,
) -> Option<Diagram> {
  let n = marks.len();
  let step = |i: usize| {
    if indefinite.get(i).copied().unwrap_or(false) {
      1.6 * len
    } else {
      len
    }
  };
  let mut d = Diagram {
    marks,
    pos: Vec::new(),
    edges: Vec::new(),
    arcs: Vec::new(),
    bars: Vec::new(),
    label_dirs: vec!["below"; n],
  };
  let chain = |d: &mut Diagram, upto: usize| {
    let mut x = 0.0;
    for i in 0..upto {
      d.pos.push((x, 0.0));
      x += step(i);
    }
  };
  let single = |a, b, ind| (a, b, Bond::Single, false, ind, None);
  match series {
    'A' | 'B' | 'C' | 'H' | 'I' => {
      if n == 0 {
        return None;
      }
      chain(&mut d, n);
      for i in 0..n.saturating_sub(1) {
        d.edges.push(single(
          i,
          i + 1,
          indefinite.get(i).copied().unwrap_or(false),
        ));
      }
      if n >= 2 {
        let last = d.edges.len() - 1;
        match series {
          'B' | 'C' if coxeter => d.edges[last].5 = Some("4".into()),
          'B' => {
            d.edges[last].2 = Bond::Double;
            d.edges[last].3 = true;
          },
          'C' => {
            let (a, b) = (d.edges[last].0, d.edges[last].1);
            d.edges[last] = (b, a, Bond::Double, true, false, None);
          },
          'H' => d.edges[0].5 = Some("5".into()),
          'I' => {
            let g = gonality.trim();
            if !g.is_empty() && g != "0" {
              d.edges[0].5 = Some(g.into());
            }
          },
          _ => {},
        }
      }
    },
    'D' => {
      if n < 4 {
        return layout('A', d.marks, indefinite, len, coxeter, gonality);
      }
      chain(&mut d, n - 2);
      let (bx, by) = d.pos[n - 3];
      let (c, s) = (len * 0.5, len * 3f64.sqrt() / 2.0);
      d.pos.push((bx + c, by + s));
      d.pos.push((bx + c, by - s));
      for i in 0..n - 3 {
        d.edges.push(single(
          i,
          i + 1,
          indefinite.get(i).copied().unwrap_or(false),
        ));
      }
      d.edges.push(single(n - 3, n - 2, false));
      d.edges.push(single(n - 3, n - 1, false));
      d.label_dirs[n - 2] = "right";
      d.label_dirs[n - 1] = "right";
    },
    'E' => {
      if !(6..=8).contains(&n) {
        return None;
      }
      // Roots 1, 3, 4, 5, … along the line; root 2 above root 4.
      let mut pos = vec![(0.0, 0.0); n];
      pos[0] = (0.0, 0.0);
      pos[2] = (len, 0.0);
      pos[3] = (2.0 * len, 0.0);
      pos[1] = (2.0 * len, len);
      for (k, p) in pos.iter_mut().enumerate().skip(4) {
        *p = (k as f64 * len - len, 0.0);
      }
      d.pos = pos;
      d.edges.push(single(0, 2, false));
      d.edges.push(single(2, 3, false));
      d.edges.push(single(1, 3, false));
      for i in 3..n - 1 {
        d.edges.push(single(i, i + 1, false));
      }
      d.label_dirs[1] = "right";
    },
    'F' => {
      if n != 4 {
        return None;
      }
      chain(&mut d, 4);
      for i in 0..3 {
        d.edges.push(single(i, i + 1, false));
      }
      if coxeter {
        d.edges[1].5 = Some("4".into());
      } else {
        d.edges[1].2 = Bond::Double;
        d.edges[1].3 = true;
      }
    },
    'G' => {
      if n != 2 {
        return None;
      }
      chain(&mut d, 2);
      d.edges.push(if coxeter {
        (0, 1, Bond::Single, false, false, Some("6".into()))
      } else {
        (1, 0, Bond::Triple, true, false, None)
      });
    },
    _ => return None,
  }
  Some(d)
}

/// The diagram folded in two (ply 2, :4255-4300, :4870-4890).
fn fold(series: char, d: Diagram, len: f64, r_fold: f64, indefinite: &[bool]) -> Option<Diagram> {
  let n = d.marks.len();
  let step = |i: usize| {
    if indefinite.get(i).copied().unwrap_or(false) {
      1.6 * len
    } else {
      len
    }
  };
  let mut f = Diagram {
    marks:      d.marks,
    pos:        vec![(0.0, 0.0); n],
    edges:      Vec::new(),
    arcs:       Vec::new(),
    bars:       Vec::new(),
    label_dirs: d.label_dirs,
  };
  match series {
    'A' => {
      let h = n / 2;
      if h == 0 {
        return None;
      }
      let mut x = 0.0;
      for (i, pos) in f.pos.iter_mut().enumerate().take(h) {
        *pos = (x, r_fold);
        if i + 1 < h {
          x += step(i);
        }
      }
      let xh = f.pos[h - 1].0;
      if n % 2 == 0 {
        f.arcs.push((h - 1, 90.0, -90.0));
        f.pos[h] = (xh, -r_fold);
      } else {
        f.pos[h] = (xh + r_fold, 0.0);
        f.arcs.push((h - 1, 90.0, 0.0));
        f.arcs.push((h + 1, -90.0, 0.0));
      }
      // The second half mirrors the first under the fold.
      for j in (n - h)..n {
        f.pos[j] = (f.pos[n - 1 - j].0, -r_fold);
      }
      for i in 0..n - 1 {
        let is_arc = (n % 2 == 0 && i == h - 1) || (n % 2 == 1 && (i == h - 1 || i == h));
        if !is_arc {
          f.edges.push((
            i,
            i + 1,
            Bond::Single,
            false,
            indefinite.get(i).copied().unwrap_or(false),
            None,
          ));
        }
      }
      for i in 0..h {
        f.bars.push((i, n - 1 - i));
      }
      for (k, dir) in f.label_dirs.iter_mut().enumerate() {
        *dir = if k < h { "above" } else { "below" };
      }
    },
    'D' => {
      if n < 4 {
        return None;
      }
      let mut x = 0.0;
      for (i, pos) in f.pos.iter_mut().enumerate().take(n - 2) {
        *pos = (x, 0.0);
        if i + 1 < n - 2 {
          x += step(i);
        }
      }
      let xb = f.pos[n - 3].0;
      f.pos[n - 2] = (xb + r_fold, r_fold);
      f.pos[n - 1] = (xb + r_fold, -r_fold);
      for i in 0..n - 3 {
        f.edges.push((
          i,
          i + 1,
          Bond::Single,
          false,
          indefinite.get(i).copied().unwrap_or(false),
          None,
        ));
      }
      f.arcs.push((n - 2, 90.0, 180.0));
      f.arcs.push((n - 1, -90.0, -180.0));
      f.bars.push((n - 2, n - 1));
    },
    'E' if n == 6 => {
      f.pos[0] = (0.0, r_fold);
      f.pos[2] = (len, r_fold);
      f.pos[3] = (len + r_fold, 0.0);
      f.pos[4] = (len, -r_fold);
      f.pos[5] = (0.0, -r_fold);
      f.pos[1] = (2.0 * len + r_fold, 0.0);
      f.edges.push((0, 2, Bond::Single, false, false, None));
      f.edges.push((4, 5, Bond::Single, false, false, None));
      f.edges.push((1, 3, Bond::Single, false, false, None));
      f.arcs.push((2, 90.0, 0.0));
      f.arcs.push((4, -90.0, 0.0));
      f.bars.push((0, 5));
      f.bars.push((2, 4));
      f.label_dirs = vec!["above", "right", "above", "right", "below", "below"];
    },
    _ => return None,
  }
  Some(f)
}

fn pt(v: f64) -> String { format!("{:.3}pt", v) }
fn p(q: (f64, f64)) -> String { format!("({},{})", pt(q.0), pt(q.1)) }

/// The TikZ commands drawing a diagram: bonds, fold arcs and bars, then the marks over them, then the
/// labels (the package's order: edges on its background layer, marks refreshed last, :216).
fn draw(d: &Diagram, r: f64, labels: &[String]) -> String {
  let mut s = String::new();
  for (a, b, bond, arrow, indefinite, label) in &d.edges {
    let (pa, pb) = (d.pos[*a], d.pos[*b]);
    let (dx, dy) = (pb.0 - pa.0, pb.1 - pa.1);
    let len = (dx * dx + dy * dy).sqrt().max(1e-9);
    let (ux, uy) = (dx / len, dy / len);
    let (nx, ny) = (-uy, ux);
    let off = |q: (f64, f64), k: f64| (q.0 + nx * k, q.1 + ny * k);
    if *indefinite {
      let t1 = (pa.0 + dx / 3.0, pa.1 + dy / 3.0);
      let t2 = (pa.0 + 2.0 * dx / 3.0, pa.1 + 2.0 * dy / 3.0);
      s += &format!(
        "\\draw[thin]{}--{};\\draw[thin,densely dotted]{}--{};\\draw[thin]{}--{};",
        p(pa),
        p(t1),
        p(t1),
        p(t2),
        p(t2),
        p(pb)
      );
    } else {
      match bond {
        Bond::Single => s += &format!("\\draw[thin]{}--{};", p(pa), p(pb)),
        Bond::Double => {
          s += &format!(
            "\\draw[thin]{}--{};\\draw[thin]{}--{};",
            p(off(pa, r / 2.0)),
            p(off(pb, r / 2.0)),
            p(off(pa, -r / 2.0)),
            p(off(pb, -r / 2.0))
          )
        },
        Bond::Triple => {
          s += &format!(
            "\\draw[thin]{}--{};\\draw[thin]{}--{};\\draw[thin]{}--{};",
            p(off(pa, r)),
            p(off(pb, r)),
            p(pa),
            p(pb),
            p(off(pa, -r)),
            p(off(pb, -r))
          )
        },
      }
    }
    if *arrow {
      // The arrowhead at the bond's middle, pointing at `to` (:1577: tip .75r past the middle, 1.5r long).
      let tip = (
        pa.0 + dx / 2.0 + ux * 0.75 * r,
        pa.1 + dy / 2.0 + uy * 0.75 * r,
      );
      let back = (tip.0 - ux * 1.5 * r, tip.1 - uy * 1.5 * r);
      let w = if *bond == Bond::Triple {
        1.4 * r
      } else {
        1.1 * r
      };
      s += &format!(
        "\\draw[thin,line join=round]{}--{}--{};",
        p(off(back, w)),
        p(tip),
        p(off(back, -w))
      );
    }
    if let Some(l) = label {
      let mid = (pa.0 + dx / 2.0, pa.1 + dy / 2.0);
      s += &format!(
        "\\node[above,inner sep={},black,scale=.7] at {} {{\\({l}\\)}};",
        pt(r),
        p(mid)
      );
    }
  }
  for (root, from, to) in &d.arcs {
    s += &format!(
      "\\draw[thin]{} arc ({from}:{to}:{});",
      p(d.pos[*root]),
      pt((d.pos[*root].1).abs().max(1e-9))
    );
  }
  for (a, b) in &d.bars {
    s += &format!(
      "\\draw[draw=black!40,line width={},line cap=round,shorten <={},shorten >={}]{}--{};",
      pt(r),
      pt(2.0 * r),
      pt(2.0 * r),
      p(d.pos[*a]),
      p(d.pos[*b])
    );
  }
  let diag = r / 2f64.sqrt();
  for (k, mark) in d.marks.iter().enumerate() {
    let q = p(d.pos[k]);
    s += &match mark {
      '*' => format!("\\fill[black]{q} circle ({});", pt(r)),
      'o' => format!(
        "\\filldraw[draw=black,fill=white,thin]{q} circle ({});",
        pt(r)
      ),
      'O' => format!(
        "\\filldraw[draw=black,fill=white,thin]{q} circle ({});\\draw[thin]{q} circle ({});",
        pt(2.0 * r),
        pt(r)
      ),
      'x' | 'X' => format!(
        "\\draw[{},line cap=round]{q}+({},{})--+({},{}){q}+({},{})--+({},{});",
        if *mark == 'X' { "very thick" } else { "thick" },
        pt(-r),
        pt(-r),
        pt(r),
        pt(r),
        pt(-r),
        pt(r),
        pt(r),
        pt(-r)
      ),
      _ => format!(
        "\\filldraw[draw=black,fill=white,thin]{q} circle ({});\\draw[thin]{q}+({},{})--+({},{}){q}+({},{})--+({},{});",
        pt(r),
        pt(-diag),
        pt(-diag),
        pt(diag),
        pt(diag),
        pt(-diag),
        pt(diag),
        pt(diag),
        pt(-diag)
      ),
    };
  }
  for (k, label) in labels.iter().enumerate().take(d.marks.len()) {
    if !label.trim().is_empty() {
      s += &format!(
        "\\node[{},inner sep={},black,scale=.7] at {} {{\\({}\\)}};",
        d.label_dirs[k],
        pt(r),
        p(d.pos[k]),
        label.trim()
      );
    }
  }
  s
}

#[rustfmt::skip]
LoadDefinitions!({
  // The package itself (its options, keys, layers and the long tail of `\dynkin`).
  InputDefinitions!("dynkin-diagrams", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  // The fallback is the package's own `\dynkin` body (dynkin-diagrams.sty:97-106).
  RawTeX!(r"\NewDocumentCommand\lx@dynkin@orig{O{}mO{0}m}{\dynkin@save{}\ifdefined\filldraw\relax\@dynkin[vertical shift=0,#1]{#2}[#3]{#4}\else\tikz[baseline=(origin.base)]{\@dynkin[#1]{#2}[#3]{#4}}\fi\dynkin@restore{}}
\RenewDocumentCommand\dynkin{O{}mO{0}m}{\lx@dynkin@native{#1}{#2}{#3}{#4}}");

  DefMacro!("\\lx@dynkin@native{}{}{}{}", sub[(keys, series, twist, spec)] {
    let fallback = |keys: &Tokens, series: &Tokens, twist: &Tokens, spec: &Tokens| -> Result<Tokens> {
      let mut out = vec![T_CS!("\\lx@dynkin@orig"), T_OTHER!("[")];
      out.extend(keys.clone().unlist());
      out.extend([T_OTHER!("]"), T_BEGIN!()]);
      out.extend(series.clone().unlist());
      out.extend([T_END!(), T_OTHER!("[")]);
      out.extend(twist.clone().unlist());
      out.extend([T_OTHER!("]"), T_BEGIN!()]);
      out.extend(spec.clone().unlist());
      out.push(T_END!());
      Ok(Tokens::new(out))
    };
    let pairs = kv(&keys.clone().unlist());
    let series_s = series.clone().untex().trim().to_string();
    let twist_s = twist.clone().untex().trim().to_string();
    // `\dynkin E6` / `\dynkin E{6}`: the rank may follow the letter.
    let mut chars = series_s.chars();
    let letter = chars.next().unwrap_or(' ');
    let rest: String = chars.collect();
    let spec_s = if rest.is_empty() { spec.clone().untex().to_string() } else { rest.clone() };
    let supported = pairs.iter().all(|(k, _)| SUPPORTED_KEYS.contains(&k.as_str()))
      && (twist_s.is_empty() || twist_s == "0")
      && "ABCDEFGHI".contains(letter)
      && (rest.is_empty() || rest.chars().all(|c| c.is_ascii_digit()));
    if !supported {
      return fallback(&keys, &series, &twist, &spec);
    }
    let value = |names: &[&str]| pairs.iter().rev().find(|(k, _)| names.contains(&k.as_str())).map(|(_, v)| v.clone());
    let coxeter = value(&["Coxeter"]).is_some_and(|v| v.is_empty() || v == "true")
      || Expand!(mouth::tokenize_internal(r"\ifdynkin@Coxeter T\else F\fi")).to_string().trim() == "T";
    let folded = value(&["fold"]).is_some_and(|v| v.is_empty() || v == "true");
    let Some(len) = dimension_pt(match value(&["edge length", "edge-length", "edgelength"]) {
      Some(v) => mouth::tokenize_internal(TeXString::assembled(v)),
      None => Tokens!(T_CS!("\\dynkin@edge@length")),
    })? else {
      return fallback(&keys, &series, &twist, &spec);
    };
    let Some(r) = dimension_pt(match value(&["root radius", "root-radius", "rootradius"]) {
      Some(v) => mouth::tokenize_internal(TeXString::assembled(v)),
      None => Tokens!(T_CS!("\\dynkin@root@radius")),
    })? else {
      return fallback(&keys, &series, &twist, &spec);
    };
    let Some(r_fold) = dimension_pt(match value(&["fold radius", "fold-radius", "foldradius"]) {
      Some(v) => mouth::tokenize_internal(TeXString::assembled(v)),
      None => Tokens!(T_CS!("\\dynkin@fold@radius")),
    })? else {
      return fallback(&keys, &series, &twist, &spec);
    };
    let gonality = value(&["gonality"]).unwrap_or_default();
    // A rank with no spec (`\dynkin E6`), or the spec argument.
    let spec_text = if rest.is_empty() { spec_s } else { rest };
    let Some((marks, indefinite)) = parse_spec(letter, &spec_text, coxeter) else {
      return fallback(&keys, &series, &twist, &spec);
    };
    let Some(diagram) = layout(letter, marks, &indefinite, len, coxeter, &gonality) else {
      return fallback(&keys, &series, &twist, &spec);
    };
    let diagram = if folded {
      match fold(letter, diagram, len, r_fold, &indefinite) {
        Some(f) => f,
        None => return fallback(&keys, &series, &twist, &spec),
      }
    } else {
      diagram
    };
    let n = diagram.marks.len();
    let labels: Vec<String> = match (value(&["labels"]), value(&["label"])) {
      (Some(list), _) => list.split(',').map(str::to_string).collect(),
      (None, Some(v)) if v.is_empty() || v == "true" => (1..=n).map(|k| k.to_string()).collect(),
      _ => Vec::new(),
    };
    let body = draw(&diagram, r, &labels);
    // In text, a `\tikz` on the baseline (`origin`); in a `tikzpicture`, the commands themselves
    // (:97-123); the roots .5ex above the baseline.
    Ok(mouth::tokenize_internal(TeXString::assembled(format!(
      "\\ifdefined\\filldraw\\begin{{scope}}[yshift=.5ex]{body}\\end{{scope}}\\else\\tikz[baseline=(origin.base)]{{\\node[anchor=base,inner sep=0pt,outer sep=0pt](origin) at (0,0){{}};\\begin{{scope}}[yshift=.5ex]{body}\\end{{scope}}}}\\fi"))))
  });
});
