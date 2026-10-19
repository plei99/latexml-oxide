//! eepic.sty (Conrad Kwok, 1988) — epic's and LaTeX's picture objects drawn with tpic `\special`s.
//!
//! No Perl binding exists; native binding, OXIDIZED_DESIGN #588(t). eepic replaces the drawing of
//! epic and of LaTeX's picture objects by tpic specials (eepic.sty:59-476): lines of any slope
//! (`\line`, :311-341) and circles of any size (`\circle`, `\circle*`, :294-296), which the kernel's
//! picture model already draws; dotted lines with round dots (`\dottedline`, :104-145); epic's dashes
//! and negative-stretch drawlines as tpic segments (:147-227); and new objects: `\path(…)` (a polyline,
//! :376-388), `\spline(…)` (tpic's `sp`: the quadratic B-spline through the midpoints of the control
//! polygon, straight at its ends, :363-375), `\ellipse{xd}{yd}`/`\ellipse*` (:297-299),
//! `\arc{d}{start}{end}` (radians, clockwise on the page, :300), `\allinethickness{dim}` (:462-464),
//! `\Thicklines` (1.2pt, :269-272), `\blacken`/`\whiten`/`\shade` (fill the next path, circle, ellipse
//! or arc, :466-468), `\filltype{black|white|shade}` (what `*` fills with, :471-476), `\texture` (the
//! shade's pattern, :469) and `\maxovaldiam` (:391-409). Each is drawn here in the picture model through
//! epic_sty.rs's elements; the tpic pen sizes and milli-inch rounding are the driver's. eepic does not
//! load epic: epic's `\dashline`, `\drawline`, `\jput` and joins exist only when epic is loaded too.
use epic_sty::{
  Element, draw_call, element, polyline, px, read_points, units_to_pt, wholewidth_pt,
};

use crate::prelude::*;

/// The fill a `\blacken`/`\whiten`/`\shade` left for the next object, consumed by it.
fn take_fill() -> Option<String> {
  let fill = epic_sty::state_string("lx@eepic@fill");
  if fill.is_empty() {
    return None;
  }
  assign_value(
    "lx@eepic@fill",
    Stored::String(pin("")),
    Some(Scope::Global),
  );
  Some(fill)
}

/// The colour `*` fills with (`\filltype`).
fn star_fill() -> String {
  let fill = epic_sty::state_string("lx@eepic@filltype");
  if fill.is_empty() {
    "#000000".to_string()
  } else {
    fill
  }
}

fn spline(points: &[(f64, f64)], width: f64) -> Vec<Element> {
  if points.len() < 3 {
    return vec![polyline(points, width, None, None)];
  }
  let mid = |a: (f64, f64), b: (f64, f64)| ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
  let n = points.len();
  let mut out = vec![polyline(
    &[points[0], mid(points[0], points[1])],
    width,
    None,
    None,
  )];
  for i in 1..n - 1 {
    let (s, c, e) = (
      mid(points[i - 1], points[i]),
      points[i],
      mid(points[i], points[i + 1]),
    );
    out.push(element("ltx:bezier", &[
      (
        "points",
        format!(
          "{},{} {},{} {},{}",
          px(s.0),
          px(s.1),
          px(c.0),
          px(c.1),
          px(e.0),
          px(e.1)
        ),
      ),
      ("stroke", "#000000".to_string()),
      ("stroke-width", format!("{width}")),
    ]));
  }
  out.push(polyline(
    &[mid(points[n - 2], points[n - 1]), points[n - 1]],
    width,
    None,
    None,
  ));
  out
}

#[rustfmt::skip]
LoadDefinitions!({
  epic_sty::define_draw()?;
  RawTeX!(r"\@ifundefined{maxovaldiam}{\newdimen\maxovaldiam}{}\maxovaldiam=40pt
\def\dashlinestretch{0}\def\drawlinestretch{0}");
  AssignValue!("lx@eepic@fill" => Stored::String(pin("")), Some(Scope::Global));

  // eepic.sty:104-145: `\dottedline{gap}(points)`, round dots (a `[char]` is put at each dot, as epic does).
  DefMacro!("\\dottedline", sub[_args] {
    let char = epic_sty::read_bracket()?;
    let gap = units_to_pt(read_arg(ExpansionLevel::Off)?)?;
    let points = read_points()?;
    epic_sty::dotted_line(&points, gap, char)
  });
  // eepic.sty:376-388 `\path(points)`
  DefMacro!("\\path", sub[_args] {
    let points = read_points()?;
    let fill = take_fill();
    draw_call(vec![polyline(&points, wholewidth_pt()?, None, fill.as_deref())])
  });
  // eepic.sty:363-375 `\spline(points)`
  DefMacro!("\\spline", sub[_args] {
    let points = read_points()?;
    take_fill();
    draw_call(spline(&points, wholewidth_pt()?))
  });
  // eepic.sty:294-296: `\circle{d}`, `\circle*{d}` (filled with `\filltype`), any size; a pending fill
  // fills the outline.
  DefMacro!("\\circle OptionalMatch:* {}", sub[(star, diameter)] {
    let r = units_to_pt(diameter)? / 2.0;
    let w = wholewidth_pt()?;
    let (fill, stroke) = if star.is_some() { (star_fill(), "none".to_string()) }
      else { (take_fill().unwrap_or_else(|| "none".to_string()), "#000000".to_string()) };
    draw_call(vec![element("ltx:circle", &[
      ("x", "0".to_string()), ("y", "0".to_string()), ("r", px(r)),
      ("fill", fill), ("stroke", stroke), ("stroke-width", format!("{w}")),
    ])])
  });
  // eepic.sty:297-299 `\ellipse{xd}{yd}`, `\ellipse*`
  DefMacro!("\\ellipse OptionalMatch:* {}{}", sub[(star, xd, yd)] {
    let rx = units_to_pt(xd)? / 2.0;
    let ry = units_to_pt(yd)? / 2.0;
    let w = wholewidth_pt()?;
    let (fill, stroke) = if star.is_some() { (star_fill(), "none".to_string()) }
      else { (take_fill().unwrap_or_else(|| "none".to_string()), "#000000".to_string()) };
    draw_call(vec![element("ltx:ellipse", &[
      ("x", "0".to_string()), ("y", "0".to_string()), ("rx", px(rx)), ("ry", px(ry)),
      ("fill", fill), ("stroke", stroke), ("stroke-width", format!("{w}")),
    ])])
  });
  // eepic.sty:300 `\arc{d}{start}{end}`: radians, clockwise on the page from `start` to `end`, which
  // in the picture's upward coordinates is anticlockwise from `-end` to `-start`; a whole turn is a
  // circle.
  DefMacro!("\\arc{}{}{}", sub[(diameter, start, end)] {
    let r = units_to_pt(diameter)? / 2.0;
    let s: f64 = start.to_string().trim().parse().unwrap_or(0.0);
    let e: f64 = end.to_string().trim().parse().unwrap_or(0.0);
    let w = wholewidth_pt()?;
    let fill = take_fill().unwrap_or_else(|| "none".to_string());
    if e - s >= 2.0 * std::f64::consts::PI - 1e-9 {
      return draw_call(vec![element("ltx:circle", &[
        ("x", "0".to_string()), ("y", "0".to_string()), ("r", px(r)),
        ("fill", fill), ("stroke", "#000000".to_string()), ("stroke-width", format!("{w}")),
      ])]);
    }
    let deg = |a: f64| { let d = (a.to_degrees() * 100.0).round() / 100.0; format!("{}", d + 0.0) };
    draw_call(vec![element("ltx:arc", &[
      ("x", "0".to_string()), ("y", "0".to_string()), ("r", px(r)),
      ("angle1", deg(-e)), ("angle2", deg(-s)),
      ("fill", fill), ("stroke", "#000000".to_string()), ("stroke-width", format!("{w}")),
    ])])
  });
  // eepic.sty:269-272, :462-464
  RawTeX!(r"\def\Thicklines{\@wholewidth1.2pt\@halfwidth.6pt}
\def\allinethickness#1{\@wholewidth#1\@halfwidth.5\@wholewidth}");
  // eepic.sty:466-476: fills.
  DefPrimitive!("\\blacken", { assign_value("lx@eepic@fill", Stored::String(pin("#000000")), Some(Scope::Global)); });
  DefPrimitive!("\\whiten", { assign_value("lx@eepic@fill", Stored::String(pin("#FFFFFF")), Some(Scope::Global)); });
  DefPrimitive!("\\shade", { assign_value("lx@eepic@fill", Stored::String(pin("#808080")), Some(Scope::Global)); });
  DefMacro!("\\texture{}", None);
  DefPrimitive!("\\filltype{}", sub[(kind)] {
    let color = match kind.to_string().trim() { "white" => "#FFFFFF", "shade" => "#808080", _ => "#000000" };
    assign_value("lx@eepic@filltype", Stored::String(pin(color)), None);
  });
});
