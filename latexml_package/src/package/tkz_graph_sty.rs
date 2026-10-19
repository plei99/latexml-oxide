//! tkz-graph.sty (v2.00, 2021/01/20, A. Matthes) — graphs drawn with TikZ.
//!
//! No Perl binding exists; native binding, OXIDIZED_DESIGN #588(x). The package reads xkeyval keys
//! (families `vertex`, `edge`, `loop`, with presets the style commands change) and builds, through
//! `\protected@edef` and `\ifthenelse`, the TikZ commands that draw: `\Vertex[keys]{name}` a
//! `\node[outer sep=0pt,style,VertexStyle](name) at (x,y){label}` in a `scope` with the label distance
//! (or the label outside with `LabelOut`, as a TikZ `label=`, tkz-graph.sty:539-613), `\Edge[keys](a)(b)`
//! a `\draw[LocalEdgeStyle,style,EdgeStyle] (a) to (b)` with an optional label node (:843-871),
//! `\Edges` a chain of them (:873-889), `\EA`/`\WE`/`\NO`/`\SO`/`\NOEA`/`\NOWE`/`\SOEA`/`\SOWE` a vertex
//! a unit away from another (:623-710), `\Vertices` vertices on a line, square, triangle or circle
//! (:713-778), `\Loop`/`\grLoop` a loop (:891-947); `\SetVertexNormal`, `\SetVertexSimple`,
//! `\SetVertexShade`, `\SetVertexArt`, `\GraphInit[vstyle=…]`, `\SetUpVertex`, `\SetUpEdge` and the
//! label switches set the TikZ styles `VertexStyle`/`EdgeStyle`/`LabelStyle` and the key presets
//! (:120-505, :782-812). Here the keys are read and the TikZ commands written natively, with the
//! package's defaults and presets (kept in the state as each family's preset list), so a graph is the
//! TikZ picture the package would build, drawn by this converter's pgf SVG driver, and a document's
//! own `\tikzset{VertexStyle/.style=…}` applies as it does there.
//! Witnesses (errors in the default configuration, before → after): 2610.14502 (3 → 0; 401 `\Vertex`, 501 `\Edge`, byte-identical to the raw package).
use crate::prelude::*;

/// A key list split at its top-level commas: (key, value). A value braced whole loses its braces.
fn parse_kv(tokens: &[Token]) -> Vec<(String, Option<Vec<Token>>)> {
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
  let trim = |v: &[Token]| -> Vec<Token> {
    let s = v
      .iter()
      .position(|t| t.get_catcode() != Catcode::SPACE)
      .unwrap_or(v.len());
    let e = v
      .iter()
      .rposition(|t| t.get_catcode() != Catcode::SPACE)
      .map_or(s, |e| e + 1);
    let v = v[s..e.max(s)].to_vec();
    // One whole group: its content.
    if v.len() >= 2
      && v[0].get_catcode() == Catcode::BEGIN
      && v[v.len() - 1].get_catcode() == Catcode::END
    {
      let mut depth = 0;
      let whole = v.iter().enumerate().all(|(i, t)| {
        match t.get_catcode() {
          Catcode::BEGIN => depth += 1,
          Catcode::END => depth -= 1,
          _ => {},
        }
        depth > 0 || i == v.len() - 1
      });
      if whole {
        return v[1..v.len() - 1].to_vec();
      }
    }
    v
  };
  items
    .into_iter()
    .filter_map(|item| {
      let eq = item
        .iter()
        .position(|t| t.get_catcode() == Catcode::OTHER && t.with_str(|s| s == "="));
      let (k, v) = match eq {
        Some(i) => (&item[..i], Some(trim(&item[i + 1..]))),
        None => (&item[..], None),
      };
      let key = Tokens::new(trim(k)).untex().trim().to_string();
      if key.is_empty() { None } else { Some((key, v)) }
    })
    .collect()
}

fn tk(s: &str) -> Vec<Token> {
  mouth::tokenize_internal(TeXString::assembled(s.to_string())).unlist()
}

/// The package's presets of a key family (`\presetkeys`), as the state keeps them.
fn presets(family: &str) -> Vec<(String, String)> {
  let src = lookup_value(&format!("tkz@graph@presets@{family}"))
    .map(|v| v.to_string())
    .unwrap_or_default();
  src
    .split('\u{1}')
    .filter_map(|kv| kv.split_once('\u{2}'))
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect()
}

/// `\presetkeys[GR]{family}{key=value,…}` (xkeyval: the given keys replace their presets).
fn preset(family: &str, assignments: &[(&str, String)]) {
  let mut current = presets(family);
  for (k, v) in assignments {
    if let Some(slot) = current.iter_mut().find(|(key, _)| key == k) {
      slot.1 = v.clone();
    } else {
      current.push((k.to_string(), v.clone()));
    }
  }
  let src = current
    .iter()
    .map(|(k, v)| format!("{k}\u{2}{v}"))
    .collect::<Vec<_>>()
    .join("\u{1}");
  assign_value(
    &format!("tkz@graph@presets@{family}"),
    Stored::String(pin(src)),
    Some(Scope::Global),
  );
}

/// A family's values for a call: the presets, then the given keys (a key with no value is `true`).
struct Keys(Vec<(String, Vec<Token>)>);
impl Keys {
  fn new(family: &str, given: &[Token]) -> Keys {
    let mut vals: Vec<(String, Vec<Token>)> = presets(family)
      .into_iter()
      .map(|(k, v)| (k, tk(&v)))
      .collect();
    for (k, v) in parse_kv(given) {
      let v = v.unwrap_or_else(|| tk("true"));
      if let Some(slot) = vals.iter_mut().find(|(key, _)| *key == k) {
        slot.1 = v;
      } else {
        vals.push((k, v));
      }
    }
    Keys(vals)
  }
  fn get(&self, key: &str) -> Vec<Token> {
    self
      .0
      .iter()
      .find(|(k, _)| k == key)
      .map(|(_, v)| v.clone())
      .unwrap_or_default()
  }
  fn flag(&self, key: &str) -> bool { Tokens::new(self.get(key)).untex().trim() == "true" }
  fn empty(&self, key: &str) -> bool {
    self
      .get(key)
      .iter()
      .all(|t| t.get_catcode() == Catcode::SPACE)
  }
}

fn group(v: Vec<Token>) -> Vec<Token> {
  let mut out = vec![T_BEGIN!()];
  out.extend(v);
  out.push(T_END!());
  out
}

/// tkz-graph.sty:539-613 `\@@vertex`.
fn vertex(opts: &[Token], name: Vec<Token>) -> Vec<Token> {
  let keys = Keys::new("vertex", opts);
  let style = keys.get("style");
  let position: Vec<Token> = if keys.flag("Node") {
    let mut p = tk("at (");
    p.extend(name.clone());
    p.extend(tk(")"));
    p
  } else if !keys.empty("x") {
    let mut p = tk("at (");
    p.extend(keys.get("x"));
    p.extend(tk(","));
    p.extend(keys.get("y"));
    p.extend(tk(")"));
    p
  } else if !keys.empty("d") {
    let mut p = tk("at (");
    p.extend(keys.get("a"));
    p.extend(tk(":"));
    p.extend(keys.get("d"));
    p.extend(tk(")"));
    p
  } else {
    let mut p = tk("[");
    p.extend(keys.get("position"));
    p.extend(tk("]"));
    p
  };
  let mut out = Vec::new();
  if keys.flag("empty") {
    out.extend(tk("\\begin{scope}\\node[inner sep = 0pt]("));
    out.extend(name);
    out.extend(tk(")"));
    out.extend(position);
    out.extend(tk("{};\\end{scope}"));
    return out;
  }
  let (label_text, label_option): (Vec<Token>, Vec<Token>) = if keys.flag("NoLabel") {
    (Vec::new(), Vec::new())
  } else {
    let base = if keys.empty("L") {
      name.clone()
    } else {
      keys.get("L")
    };
    let shown = if keys.flag("Math") {
      let mut m = vec![T_MATH!()];
      m.extend(base);
      m.push(T_MATH!());
      m
    } else {
      base
    };
    if keys.flag("LabelOut") {
      let mut opt = tk(",label={[text = \\lx@GR@SVN@TextColor,");
      opt.extend(style.clone());
      opt.extend(tk("]"));
      opt.extend(keys.get("Lpos"));
      opt.extend(tk(":"));
      opt.extend(group(shown));
      opt.extend(tk("}"));
      (Vec::new(), opt)
    } else {
      (shown, Vec::new())
    }
  };
  out.extend(tk("\\begin{scope}[label distance = "));
  out.extend(keys.get("Ldist"));
  out.extend(tk("]\\node[outer sep=0pt,"));
  out.extend(style);
  out.extend(tk(",VertexStyle"));
  out.extend(label_option);
  out.extend(tk("]("));
  out.extend(name);
  out.extend(tk(")"));
  out.extend(position);
  out.extend(group(label_text));
  out.extend(tk(";\\end{scope}"));
  out
}

/// tkz-graph.sty:843-871 `\@edge`.
fn edge(opts: &[Token], from: Vec<Token>, to: Vec<Token>) -> Vec<Token> {
  let keys = Keys::new("edge", opts);
  let mut out = tk("\\begingroup");
  if keys.empty("double") {
    out.extend(tk("\\tikzset{LocalEdgeStyle/.style={color = "));
    out.extend(keys.get("color"));
    out.extend(tk(", line width = "));
    out.extend(keys.get("lw"));
    out.extend(tk("}}"));
  } else {
    out.extend(tk("\\tikzset{LocalEdgeStyle/.style={line width = "));
    out.extend(keys.get("dd"));
    out.extend(tk(", color = "));
    out.extend(keys.get("double"));
    out.extend(tk(", double distance = "));
    out.extend(keys.get("lw"));
    out.extend(tk(", double = "));
    out.extend(keys.get("color"));
    out.extend(tk("}}"));
  }
  if keys.flag("local") {
    out.extend(tk("\\tikzset{EdgeStyle/.style={}}"));
  }
  out.extend(tk("\\draw[LocalEdgeStyle,"));
  out.extend(keys.get("style"));
  out.extend(tk(",EdgeStyle] ("));
  out.extend(from);
  out.extend(tk(") to "));
  if !keys.empty("label") {
    out.extend(tk("node[fill = "));
    out.extend(keys.get("labelcolor"));
    out.extend(tk(", text = "));
    out.extend(keys.get("labeltext"));
    out.extend(tk(","));
    out.extend(keys.get("labelstyle"));
    out.extend(tk(",LabelStyle]"));
    out.extend(group(keys.get("label")));
  }
  out.extend(tk("("));
  out.extend(to);
  out.extend(tk(");\\endgroup"));
  out
}

/// A comma list's items (top level), spaces trimmed.
fn list_items(tokens: &[Token]) -> Vec<Vec<Token>> {
  parse_kv(tokens).into_iter().map(|(k, _)| tk(&k)).collect()
}

/// tkz-graph.sty:623-710: a vertex at `(dx,dy)` units from `from`.
fn step_vertex(opts: &[Token], from: Vec<Token>, name: Vec<Token>, dx: i32, dy: i32) -> Vec<Token> {
  let keys = Keys::new("vertex", opts);
  let unit = keys.get("unit");
  let coord = |d: i32| -> Vec<Token> {
    match d {
      0 => tk("0"),
      1 => unit.clone(),
      _ => {
        let mut v = tk("-");
        v.extend(unit.clone());
        v
      },
    }
  };
  let mut out = tk("\\begingroup\\path (");
  out.extend(from);
  out.extend(tk(")--++("));
  out.extend(coord(dx));
  out.extend(tk(","));
  out.extend(coord(dy));
  out.extend(tk(") coordinate ("));
  out.extend(name.clone());
  out.extend(tk(");"));
  let mut vopts = opts.to_vec();
  vopts.extend(tk(",Node"));
  out.extend(vertex(&vopts, name));
  out.extend(tk("\\endgroup"));
  out
}

const DIRECTIONS: &[(&str, i32, i32)] = &[
  ("EA", 1, 0),
  ("WE", -1, 0),
  ("NO", 0, 1),
  ("SO", 0, -1),
  ("NOEA", 1, 1),
  ("NOWE", -1, 1),
  ("SOEA", 1, -1),
  ("SOWE", -1, -1),
];

/// Read `[opts]` (optional) for a command defined by hand.
fn read_opts() -> Result<Vec<Token>> {
  Ok(read_optional(None)?.map(|t| t.unlist()).unwrap_or_default())
}

fn read_paren() -> Result<Vec<Token>> {
  skip_spaces()?;
  if !if_next(T_OTHER!("("))? {
    return Ok(Vec::new());
  }
  read_token()?;
  Ok(
    read_until(&Tokens!(T_OTHER!(")")))?
      .unwrap_or_default()
      .unlist(),
  )
}

/// GraphInit's `vstyle` choices (tkz-graph.sty:229-391), in order: the preset each sets and its
/// `VertexStyle`/`EdgeStyle`.
const VSTYLES: &[(&str, &str, &str, &str, &str)] = &[
  (
    "Empty",
    "LabelOut",
    "false",
    r"shape = \VertexShape, color = \VertexLineColor, fill = \VertexLightFillColor, inner sep = \VertexInnerSep, outer sep = 0.5\pgflinewidth, text = \VertexTextColor, minimum size = \VertexSmallMinSize, line width = \VertexLineWidth",
    r"line width = \EdgeLineWidth,\EdgeColor",
  ),
  (
    "Hasse",
    "NoLabel",
    "true",
    r"draw, shape = \VertexShape, color = \VertexLineColor, fill = \VertexLightFillColor, inner sep = \VertexInnerSep, outer sep = 0.5\pgflinewidth, text = \VertexTextColor, minimum size = \VertexSmallMinSize, line width = \VertexLineWidth",
    r"line width = \EdgeLineWidth, \EdgeColor",
  ),
  (
    "Simple",
    "NoLabel",
    "true",
    r"draw, shape = \VertexShape, color = \VertexLineColor, fill = \VertexDarkFillColor, inner sep = \VertexInnerSep, outer sep = 0.5\pgflinewidth, text = \VertexTextColor, minimum size = \VertexSmallMinSize, line width = \VertexLineWidth",
    r"line width = \EdgeLineWidth, \EdgeColor",
  ),
  (
    "Classic",
    "LabelOut",
    "true",
    r"draw, shape = \VertexShape, color = \VertexLineColor, fill = \VertexDarkFillColor, inner sep = \VertexInnerSep, outer sep = 0.5\pgflinewidth, text = \VertexTextColor, minimum size = \VertexSmallMinSize, line width = \VertexLineWidth",
    r"line width = \EdgeLineWidth, \EdgeColor",
  ),
  (
    "Normal",
    "LabelOut",
    "false",
    r"draw, shape = \VertexShape, minimum size = \VertexInterMinSize, line width = \VertexLineWidth, color = \VertexLineColor, outer sep = 0.5\pgflinewidth, fill = \VertexLightFillColor, text = \VertexTextColor, line width = \VertexLineWidth",
    r"line width = \EdgeLineWidth, \EdgeColor",
  ),
  (
    "Shade",
    "LabelOut",
    "false",
    r"draw, shape = \VertexShape, ball color = \VertexBallColor, color = \VertexLineColor, text = \VertexTextColor, inner sep = \VertexInnerSep, outer sep = \VertexOuterSep, minimum size = \VertexBigMinSize, line width = \VertexLineWidth",
    r"line width = \EdgeLineWidth, \EdgeColor, double = \EdgeFillColor, double distance = \EdgeDoubleDistance",
  ),
  (
    "Dijkstra",
    "LabelOut",
    "false",
    r"draw, shape = \VertexShape, color = \VertexLineColor, fill = \VertexLightFillColor, inner sep = \VertexInnerSep, outer sep = 0.5\pgflinewidth, text = \VertexTextColor, minimum size = \VertexInterMinSize, line width = \VertexLineWidth",
    r"line width = \EdgeLineWidth, \EdgeColor",
  ),
  (
    "Welsh",
    "LabelOut",
    "true",
    r"draw, shape = \VertexShape, color = \VertexLineColor, fill = \VertexLightFillColor, inner sep = \VertexInnerSep, outer sep = \VertexOuterSep, text = \VertexTextColor, minimum size = \VertexSmallMinSize, line width = \VertexLineWidth",
    r"line width = \EdgeLineWidth, \EdgeColor",
  ),
  (
    "Art",
    "NoLabel",
    "true",
    r"shape = \VertexShape, ball color = \VertexBallColor, color = \VertexLineColor, inner sep = \VertexInnerSep, outer sep = \VertexOuterSep, minimum size = \VertexSmallMinSize, line width = \VertexLineWidth",
    r"line width = \EdgeLineWidth, \EdgeArtColor",
  ),
  (
    "Shade Art",
    "NoLabel",
    "true",
    r"shape = \VertexShape, ball color = \VertexBallColor, inner sep = \VertexInnerSep, outer sep = \VertexOuterSep, minimum size = \VertexSmallMinSize, line width = \VertexLineWidth",
    r"line width = \EdgeLineWidth, \EdgeColor, double = \EdgeFillColor, double distance = .5\EdgeDoubleDistance",
  ),
];

/// The value of `key` in a style command's key list, or its default.
fn style_key(given: &[(String, Option<Vec<Token>>)], key: &str, default: &str) -> String {
  given
    .iter()
    .find(|(k, _)| k == key)
    .and_then(|(_, v)| {
      v.as_ref()
        .map(|v| Tokens::new(v.clone()).untex().to_string())
    })
    .unwrap_or_else(|| default.to_string())
}

#[rustfmt::skip]
LoadDefinitions!({
  // tkz-graph.sty:22-29
  RequirePackage!("ifthen");
  RequirePackage!("xkeyval");
  RequirePackage!("tikz");
  RawTeX!(r"\usetikzlibrary{arrows}");
  // tkz-graph.sty:30-110: counters, colours and dimensions, as the package defines them.
  RawTeX!(r"\newcounter{tkz@gr@a}\newcounter{tkz@gr@b}\newcounter{tkz@gr@c}\newcounter{tkz@gr@e}\newcounter{tkz@gr@d}
\newcounter{tkz@gr@p}\newcounter{tkz@gr@i}\newcounter{tkz@gr@n}\newcounter{tkz@gr@ta}\newcounter{tkz@gr@tb}
\def\tkz@gr@background@color{white}\def\tkz@gr@text@color{black}
\global\edef\tkz@suc@bkc{\tkz@gr@background@color}\global\edef\tkz@suc@txt{\tkz@gr@text@color}
\pgfkeys{/tkzsupcol/.cd,background/.code={\global\edef\tkz@suc@bkc{#1}},text/.code={\global\edef\tkz@suc@txt{#1}}}
\def\tkzSetUpColors{\pgfutil@ifnextchar[{\tkz@SetUpColors}{\tkz@SetUpColors[]}}
\def\tkz@SetUpColors[#1]{\begingroup\pgfkeys{/tkzsupcol/.cd,background=\tkz@gr@background@color,text=\tkz@gr@text@color}\pgfqkeys{/tkzsupcol}{#1}\pagecolor{\tkz@suc@bkc}\color{\tkz@suc@txt}\endgroup}
\newcommand*{\GraphUnit}{1}\newcommand*{\VertexInnerSep}{2pt}\newcommand*{\VertexOuterSep}{0pt}
\newcommand*{\VertexDistance}{3cm}\newcommand*{\VertexShape}{circle}\newcommand*{\VertexLineWidth}{0.5pt}
\newcommand*{\VertexLineColor}{\tkz@suc@txt}\newcommand*{\VertexLightFillColor}{\tkz@suc@bkc}
\newcommand*{\VertexDarkFillColor}{\tkz@suc@txt}\newcommand*{\VertexTextColor}{\tkz@suc@txt}
\newcommand*{\VertexFillColor}{\tkz@suc@txt}\newcommand*{\VertexBallColor}{orange}
\newcommand*{\VertexBigMinSize}{24pt}\newcommand*{\VertexInterMinSize}{18pt}\newcommand*{\VertexSmallMinSize}{12pt}
\newcommand*{\EdgeFillColor}{orange}\newcommand*{\EdgeArtColor}{orange}\newcommand*{\EdgeColor}{\tkz@suc@txt}
\newcommand*{\EdgeDoubleDistance}{1pt}\newcommand*{\EdgeLineWidth}{0.8pt}
\newcommand*{\LabelTextColor}{\tkz@suc@txt}\newcommand*{\LabelFillColor}{\tkz@suc@bkc}
\tikzset{pre/.style={<-,shorten <=1pt,>=stealth',semithick}}
\tikzset{post/.style={->,shorten <=1pt,>=stealth',semithick}}
\def\lx@GR@SVN@TextColor{\VertexTextColor}");

  // The key presets (tkz-graph.sty:520-536, :823-835, :907-913).
  preset("vertex", &[("Node", "false".into()), ("NoLabel", "false".into()), ("LabelOut", "false".into()),
    ("empty", "false".into()), ("Math", "false".into()), ("x", "".into()), ("y", "".into()), ("a", "".into()),
    ("d", "".into()), ("unit", "\\GraphUnit".into()), ("dir", "\\EA".into()), ("position", "".into()),
    ("style", "".into()), ("L", "".into()), ("Ldist", "0cm".into()), ("Lpos", "0".into())]);
  preset("edge", &[("style", "pos=.5".into()), ("lw", "\\EdgeLineWidth".into()), ("label", "".into()),
    ("labeltext", "\\LabelTextColor".into()), ("labelcolor", "\\LabelFillColor".into()), ("labelstyle", "".into()),
    ("local", "false".into()), ("double", "".into()), ("dd", "\\EdgeDoubleDistance".into()),
    ("incolor", "\\EdgeFillColor".into()), ("color", "\\EdgeColor".into())]);
  preset("loop", &[("style", "pre,thick".into()), ("label", "".into()), ("labelstyle", "".into()),
    ("color", "\\tkz@suc@txt".into()), ("dist", "4cm".into()), ("dir", "WE".into())]);

  // tkz-graph.sty:120-225: the vertex style commands.
  DefPrimitive!("\\lx@GR@setvertex{}[]", sub[(kind, opts)] {
    let kind = kind.untex().trim().to_string();
    let given = parse_kv(&opts.map(|o| o.unlist()).unwrap_or_default());
    let k = |key: &str, default: &str| style_key(&given, key, default);
    let (vstyle, estyle, nolabel) = match kind.as_str() {
      "Simple" => (format!("draw, shape = {}, color = {}, fill = {}, inner sep = {}, outer sep = {}, minimum size = {}, line width = {}",
          k("Shape", "\\VertexShape"), k("LineColor", "\\VertexLineColor"), k("FillColor", "\\VertexFillColor"),
          k("InnerSep", "\\VertexInnerSep"), k("OuterSep", "\\VertexOuterSep"), k("MinSize", "\\VertexSmallMinSize"),
          k("LineWidth", "\\VertexLineWidth")),
        "line width = \\EdgeLineWidth, \\EdgeColor".to_string(), Some("true")),
      "Shade" => (format!("draw, shape = {}, ball color = {}, inner sep = {}, outer sep = {}, minimum size = {}, line width = \\VertexLineWidth",
          k("Shape", "\\VertexShape"), k("BallColor", "\\VertexBallColor"), k("InnerSep", "\\VertexInnerSep"),
          k("OuterSep", "\\VertexOuterSep"), k("MinSize", "\\VertexBigMinSize")),
        "line width = \\EdgeLineWidth, \\EdgeColor, double = \\EdgeFillColor, double distance = \\EdgeDoubleDistance".to_string(), Some("false")),
      "Art" => (format!("draw, shape = {}, ball color = {}, inner sep = {}, outer sep = {}, minimum size = {}, line width = \\VertexLineWidth",
          k("Shape", "\\VertexShape"), k("BallColor", "\\VertexBallColor"), k("InnerSep", "\\VertexInnerSep"),
          k("OuterSep", "\\VertexOuterSep"), k("MinSize", "\\VertexSmallMinSize")),
        "line width = \\EdgeLineWidth, \\EdgeArtColor".to_string(), Some("true")),
      _ => {
        let text = k("TextColor", "\\VertexTextColor");
        RawTeX!(format!("\\def\\lx@GR@SVN@TextColor{{{text}}}").as_str());
        (format!("shape = {}, minimum size = {}, line width = {}, color = {}, fill = {}, text = {}, inner sep = {}, outer sep = {}, draw",
          k("Shape", "\\VertexShape"), k("MinSize", "\\VertexInterMinSize"), k("LineWidth", "\\VertexLineWidth"),
          k("LineColor", "\\VertexLineColor"), k("FillColor", "\\VertexLightFillColor"), text,
          k("InnerSep", "\\VertexInnerSep"), k("OuterSep", "\\VertexOuterSep")),
        "line width = \\EdgeLineWidth, \\EdgeColor".to_string(), None)
      },
    };
    if let Some(v) = nolabel {
      preset("vertex", &[("NoLabel", v.to_string())]);
    }
    RawTeX!(format!("\\tikzset{{VertexStyle/.style = {{{vstyle}}}}}\\tikzset{{EdgeStyle/.style = {{{estyle}}}}}\\tikzset{{LabelStyle/.style={{}}}}").as_str());
  });
  RawTeX!(r"\newcommand*{\SetVertexNormal}[1][]{\lx@GR@setvertex{Normal}[#1]}
\newcommand*{\SetVertexSimple}[1][]{\lx@GR@setvertex{Simple}[#1]}
\newcommand*{\SetVertexShade}[1][]{\lx@GR@setvertex{Shade}[#1]}
\newcommand*{\SetVertexArt}[1][]{\lx@GR@setvertex{Art}[#1]}
\SetVertexNormal");

  // tkz-graph.sty:229-399 `\GraphInit[unit=…,vstyle=…]`, `\SetGraphUnit`.
  DefPrimitive!("\\GraphInit[]", sub[(opts)] {
    let given = parse_kv(&opts.map(|o| o.unlist()).unwrap_or_default());
    let vstyle = style_key(&given, "vstyle", "Normal");
    let unit = style_key(&given, "unit", "\\GraphUnit");
    match VSTYLES.iter().find(|(name, ..)| name.eq_ignore_ascii_case(vstyle.trim())) {
      Some((_, key, value, vs, es)) => {
        preset("vertex", &[(key, value.to_string())]);
        RawTeX!(format!("\\tikzset{{VertexStyle/.style = {{{vs}}}}}\\tikzset{{EdgeStyle/.style = {{{es}}}}}\\tikzset{{LabelStyle/.style={{}}}}").as_str());
      },
      None => {
        RawTeX!(format!("\\PackageError{{xkeyval}}{{value `{vstyle}' is not allowed}}{{}}").as_str());
      },
    }
    RawTeX!(format!("\\edef\\GraphUnit{{{unit}}}").as_str());
  });
  RawTeX!(r"\newcommand*{\SetGraphUnit}[1]{\renewcommand{\GraphUnit}{#1}}
\newcommand*{\SetGraphShadeColor}[3]{\tikzset{VertexStyle/.append style = {ball color=#1}}\tikzset{EdgeStyle/.append style = {color=#2,double=#3}}}
\newcommand*{\SetGraphArtColor}[2]{\tikzset{VertexStyle/.append style = {ball color=#1}}\tikzset{EdgeStyle/.append style = {color=#2}}}
\newcommand*{\SetGraphColor}[2]{\tikzset{VertexStyle/.append style = {fill=#1}}\tikzset{EdgeStyle/.append style = {color=#2}}}
\newcommand*{\AddVertexColor}[2]{\begingroup\tikzset{VertexStyle/.append style = {fill=#1}}\foreach \v in {#2}{\Vertex[Node,NoLabel]{\v}}\endgroup}");

  // tkz-graph.sty:421-443: label presets.
  DefPrimitive!("\\lx@GR@preset{}{}{}", sub[(family, key, value)] {
    preset(family.untex().trim(), &[(key.untex().trim(), value.untex().trim().to_string())]);
  });
  RawTeX!(r"\newcommand*{\SetVertexNoLabel}{\lx@GR@preset{vertex}{NoLabel}{true}}
\newcommand*{\SetVertexLabel}{\lx@GR@preset{vertex}{NoLabel}{false}}
\newcommand*{\SetVertexLabelOut}{\lx@GR@preset{vertex}{LabelOut}{true}}
\newcommand*{\SetVertexLabelIn}{\lx@GR@preset{vertex}{LabelOut}{false}}
\newcommand*{\SetVertexMath}{\lx@GR@preset{vertex}{Math}{true}}
\newcommand*{\SetVertexNoMath}{\lx@GR@preset{vertex}{Math}{false}}");
  // tkz-graph.sty:444-505 `\SetUpVertex[keys]`
  DefPrimitive!("\\SetUpVertex[]", sub[(opts)] {
    let given = parse_kv(&opts.map(|o| o.unlist()).unwrap_or_default());
    let k = |key: &str, default: &str| style_key(&given, key, default);
    preset("vertex", &[("Lpos", k("Lpos", "0")), ("Ldist", k("Ldist", "0cm")), ("style", k("Style", "")),
      ("unit", k("Unit", "\\GraphUnit"))]);
    RawTeX!(format!("\\tikzset{{VertexStyle/.append style = {{inner sep = {}, outer sep = {}, minimum size = {}, text = {}, color = {}, fill = {}, line width = {}}}}}",
      k("InnerSep", "\\VertexInnerSep"), k("OuterSep", "\\VertexOuterSep"), k("MinSize", "\\VertexInterMinSize"),
      k("TextColor", "\\VertexTextColor"), k("LineColor", "\\VertexLineColor"), k("FillColor", "\\VertexLightFillColor"),
      k("LineWidth", "\\VertexLineWidth")).as_str());
    for flag in ["NoLabel", "LabelOut", "Math"] {
      if given.iter().any(|(key, v)| key == flag && v.as_ref().is_none_or(|v| Tokens::new(v.clone()).untex().trim() == "true")) {
        preset("vertex", &[(flag, "true".to_string())]);
      }
    }
  });
  // tkz-graph.sty:782-814 `\SetUpEdge[keys]`, `\DisableEdgeStyle`
  DefPrimitive!("\\SetUpEdge[]", sub[(opts)] {
    let given = parse_kv(&opts.map(|o| o.unlist()).unwrap_or_default());
    let k = |key: &str, default: &str| style_key(&given, key, default);
    preset("edge", &[("color", k("color", "\\EdgeColor")), ("lw", k("lw", "\\EdgeLineWidth")),
      ("label", k("label", "")), ("labeltext", k("labeltext", "\\LabelTextColor")),
      ("labelcolor", k("labelcolor", "\\LabelFillColor")), ("labelstyle", k("labelstyle", "")),
      ("style", k("style", ""))]);
    RawTeX!(r"\tikzset{LabelStyle/.style={}}\tikzset{EdgeStyle/.style={}}");
  });
  RawTeX!(r"\newcommand*{\DisableEdgeStyle}{\tikzset{EdgeStyle/.style={}}}");

  // The drawing commands.
  DefMacro!("\\Vertex", sub[_args] {
    let opts = read_opts()?;
    let name = read_arg(ExpansionLevel::Off)?.unlist();
    Ok(Tokens::new(vertex(&opts, name)))
  });
  for (dir, dx, dy) in DIRECTIONS.iter().copied() {
    def_macro(T_CS!(&format!("\\{dir}")), None, ExpansionBody::Closure(Rc::new(move |_args| {
      let opts = read_opts()?;
      let from = read_paren()?;
      let name = read_arg(ExpansionLevel::Off)?.unlist();
      Ok(Tokens::new(step_vertex(&opts, from, name, dx, dy)))
    })), None)?;
  }
  DefMacro!("\\Edge", sub[_args] {
    let opts = read_opts()?;
    let from = read_paren()?;
    let to = read_paren()?;
    Ok(Tokens::new(edge(&opts, from, to)))
  });
  DefMacro!("\\Edges", sub[_args] {
    let opts = read_opts()?;
    let list = read_paren()?;
    let items = list_items(&list);
    let mut out = tk("\\begingroup");
    for pair in items.windows(2) {
      out.extend(edge(&opts, pair[0].clone(), pair[1].clone()));
    }
    out.extend(tk("\\endgroup"));
    Ok(Tokens::new(out))
  });
  // tkz-graph.sty:713-778 `\Vertices[keys]{layout}{list}`
  DefMacro!("\\Vertices", sub[_args] {
    let opts = read_opts()?;
    let layout = read_arg(ExpansionLevel::Off)?.untex().trim().to_string();
    let list = list_items(&read_arg(ExpansionLevel::Off)?.unlist());
    let keys = Keys::new("vertex", &opts);
    let with = |extra: &str| { let mut o = opts.clone(); o.extend(tk(extra)); o };
    let next = ",x={},y={},a={},d={},Node=false";
    let mut out = tk("\\begingroup");
    let step = |out: &mut Vec<Token>, dir: &str, extra: &str, from: &Vec<Token>, to: &Vec<Token>| {
      let (_, dx, dy) = DIRECTIONS.iter().find(|(d, ..)| *d == dir).copied().unwrap_or(("EA", 1, 0));
      out.extend(step_vertex(&with(&format!("{next}{extra}")), from.clone(), to.clone(), dx, dy));
    };
    match layout.as_str() {
      "line" => {
        let dir = Tokens::new(keys.get("dir")).untex().trim().trim_start_matches('\\').to_string();
        for (i, v) in list.iter().enumerate() {
          if i == 0 { out.extend(vertex(&opts, v.clone())); } else { step(&mut out, &dir, "", &list[i - 1], v); }
        }
      },
      "square" if list.len() >= 4 => {
        out.extend(tk("\\begin{scope}[rotate=45]"));
        out.extend(vertex(&with(",Lpos=-135"), list[0].clone()));
        step(&mut out, "EA", ",Lpos=-45", &list[0], &list[1]);
        step(&mut out, "NO", ",Lpos=45", &list[1], &list[2]);
        step(&mut out, "WE", ",Lpos=135", &list[2], &list[3]);
        out.extend(tk("\\end{scope}"));
      },
      "tr1" | "tr2" | "tr3" | "tr4" if list.len() >= 3 => {
        let (d1, d2) = match layout.as_str() { "tr1" => ("EA", "NO"), "tr2" => ("NO", "EA"), "tr3" => ("NO", "WE"), _ => ("WE", "NO") };
        out.extend(vertex(&opts, list[0].clone()));
        step(&mut out, d1, "", &list[0], &list[1]);
        step(&mut out, d2, "", &list[1], &list[2]);
      },
      "circle" => {
        let n = list.len().max(1) as f64;
        let lpos: f64 = Tokens::new(keys.get("Lpos")).untex().trim().parse().unwrap_or(0.0);
        let unit = Tokens::new(keys.get("unit")).untex().to_string();
        for (k, v) in list.iter().enumerate() {
          let angle = 360.0 / n * k as f64;
          out.extend(vertex(&with(&format!(",a={angle},d={unit},Lpos={}", lpos + angle)), v.clone()));
        }
      },
      _ => {},
    }
    out.extend(tk("\\endgroup"));
    Ok(Tokens::new(out))
  });
  // tkz-graph.sty:891-947 `\grLoop[keys](v)`, `\Loop`
  DefMacro!("\\grLoop", sub[_args] {
    let opts = read_opts()?;
    let v = read_paren()?;
    let keys = Keys::new("loop", &opts);
    let dist = Tokens::new(keys.get("dist")).untex().to_string();
    let (inn, out_) = match Tokens::new(keys.get("dir")).untex().trim() {
      "EA" => (45, -45), "NO" => (135, 45), "SO" => (-45, -135), "NOWE" => (180, 90),
      "NOEA" => (90, 0), "SOWE" => (-90, -180), "SOEA" => (0, -90), _ => (225, 135),
    };
    let mut out = tk(&format!("\\tikzset{{LoopStyle/.style = {{in={inn}, out={out_}, distance={dist}}}}}\\draw["));
    out.extend(keys.get("color"));
    out.extend(tk("] ("));
    out.extend(v.clone());
    out.extend(tk(") edge [EdgeStyle,LoopStyle,"));
    out.extend(keys.get("style"));
    out.extend(tk("] node [LabelStyle,"));
    out.extend(keys.get("labelstyle"));
    out.extend(tk("]"));
    out.extend(group(keys.get("label")));
    out.extend(tk(" ("));
    out.extend(v);
    out.extend(tk(");"));
    Ok(Tokens::new(out))
  });
  Let!("\\Loop", "\\grLoop");
  // tkz-graph.sty:915-1026: the composite commands, as the package defines them (over the above).
  RawTeX!(r"\newcommand*{\grProb}[7][]{\begingroup\def\lx@GR@pb{#1}\lx@GR@pbkeys
\tikzset{LabelStyle/.append style ={pos=.5}}
\Vertex[Lpos=\lx@GR@LposA,Ldist=\lx@GR@Ldist,L=#2]{grA}
\EA[unit=\lx@GR@unit,Lpos=\lx@GR@LposB,Ldist=\lx@GR@Ldist,L=#3](grA){grB}
\Edge[style={post,bend left=60},label={$#4$},labelstyle={above}](grA)(grB)
\Edge[style={post,bend left=60},label={$#5$},labelstyle={below}](grB)(grA)
\grLoop[dist=\lx@GR@LoopDist,dir=WE,label={$#6$},labelstyle={left}](grA)
\grLoop[dist=\lx@GR@LoopDist,dir=EA,label={$#7$},labelstyle={right}](grB)\endgroup}
\def\lx@GR@pbkeys{\def\lx@GR@LposA{180}\def\lx@GR@LposB{0}\def\lx@GR@Ldist{0cm}\def\lx@GR@LoopDist{4cm}\def\lx@GR@unit{4}%
\@for\lx@GR@kv:=\lx@GR@pb\do{\expandafter\lx@GR@pbkey\lx@GR@kv==\relax}}
\def\lx@GR@pbkey#1=#2=#3\relax{\ifx\relax#1\relax\else\expandafter\def\csname lx@GR@\zap@space#1 \@empty\endcsname{#2}\fi}
\newcommand*{\grProbThree}[7][]{\begingroup\def\lx@GR@pb{#1}\lx@GR@pbkeys\begin{scope}[]
\tikzset{LabelStyle/.append style ={pos=.5}}
\Vertices[empty,unit=\lx@GR@unit]{circle}{grA,grB,grC}
\Vertex[Node,Lpos=\lx@GR@LposA,Ldist=\lx@GR@Ldist,L=#2]{grA}
\Vertex[Node,Lpos=\lx@GR@LposA,Ldist=\lx@GR@Ldist,L=#3]{grB}
\Vertex[Node,Lpos=\lx@GR@LposA,Ldist=\lx@GR@Ldist,L=#4]{grC}
\foreach \x/\y/\z in {#5}{\grLoop[dist=\lx@GR@LoopDist,dir=EA,label={$\x$}](grA)\Edge[style={post,bend right=20},label={$\y$}](grA)(grB)\Edge[style={post,bend right=20},label={$\z$}](grA)(grC)}
\foreach \x/\y/\z in {#6}{\grLoop[dist=\lx@GR@LoopDist,dir=NO,label={$\x$}](grB)\Edge[style={post,bend right=20},label={$\y$}](grB)(grA)\Edge[style={post,bend right=20},label={$\z$}](grB)(grC)}
\foreach \x/\y/\z in {#7}{\grLoop[dist=\lx@GR@LoopDist,dir=SO,label={$\x$}](grC)\Edge[style={post,bend right=20},label={$\y$}](grC)(grA)\Edge[style={post,bend right=20},label={$\z$}](grC)(grB)}
\end{scope}\endgroup}
\newcommand{\SetVertexLabeledSmall}{\SetVertexLabel\tikzset{VertexStyle/.style = {draw, shape = circle, inner sep = 1pt, minimum size = 10pt}}\tikzset{EdgeStyle/.style = {double = \tkz@suc@txt, double distance = 1pt, color = \tkz@suc@bkc, line width = 0.7pt}}}
\newcommand{\SetVertexNotLabeledSmall}{\tikzset{EdgeStyle/.style = {double = \tkz@suc@txt, double distance = 1pt, color = \tkz@suc@bkc, line width = 0.7pt}}\SetVertexNoLabel\SetVertexSimple[MinSize=6pt,FillColor=gray]\renewcommand*{\VertexLineWidth}{0pt}}
\newcommand{\SetVertexForPresentation}[3]{\GraphInit[vstyle=Shade]\SetVertexNoLabel\renewcommand*{\VertexBigMinSize}{12pt}\renewcommand*{\EdgeDoubleDistance}{2pt}\renewcommand*{\EdgeLineWidth}{0.7pt}\SetGraphShadeColor{#1}{#2}{#3}}
\newcommand{\setxyzvec}[1][20]{\pgfmathsetmacro{\xone}{cos(180+#1)}\pgfmathsetmacro{\yone}{sin(180+#1)}\pgfmathsetmacro{\xtwo}{cos(360-#1)}\pgfmathsetmacro{\ytwo}{sin(360-#1)}\pgfsetxvec{\pgfpoint{\xone cm}{\yone cm}}\pgfsetyvec{\pgfpoint{\xtwo cm}{\ytwo cm}}\pgfsetzvec{\pgfpoint{0cm}{1cm}}}");
  // tkz-graph.sty:780-781: after loading, no edge or label style.
  RawTeX!(r"\tikzset{LabelStyle/.style={}}\tikzset{EdgeStyle/.style={}}");
});
