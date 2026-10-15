//! A TikZ node without `text width` sets its lines as the rows of a nested `\vbox{\halign{…}}` (tikz.code.tex:4252-4256),
//! and tex.web §679 `append_to_vlist` puts the `\baselineskip` between the rows of that vertical list, so the
//! lines of a three-line node are 12pt (16.60 px) apart. The SVG alignment (pgfsys_latexml_def.rs
//! `tikz_alignment_bindings`) stacked the rows tightly, a line's height and depth apart (9.61 px), and a browser drew
//! the lines on top of one another; the node's box was 22pt short of pdfTeX's `\box0` (30.94444pt + 1.49998pt).
//! pgf's matrices set `\offinterlineskip` before their `\halign` (pgfmodulematrix.code.tex:181), which leaves no
//! glue, so their rows (a `\matrix`, a `tikzcd`) keep the spacing their cells give them. Perl has no interline glue
//! here either (Alignment.pm `normalize_sum_sizes` sums the row heights and depths): OXIDIZED_DESIGN_DIVERGENCES #587.
use regex::Regex;

use super::perfect_kernel_batch46::{convert, error_count, warning_count};

/// 12pt in px (`pt · 100 / 72.27`), the `\baselineskip` of 10pt `article` text.
const BASELINESKIP_PX: f64 = 16.6044;

const MULTILINE_NODES: &str = r"\documentclass{article}
\usepackage{tikz}
\begin{document}
\begin{tikzpicture}
\node[align=left] at (0,0) {Wide first line\\ Short\\ Third $x_1$};
\node[align=center] at (5,0) {Wide first line\\ Short\\ Third $x_1$};
\node[align=right] at (10,0) {Wide first line\\ Short\\ Third $x_1$};
\node[align=left,text width=3cm] at (0,-3) {Wide first line\\ Short\\ Third $x_1$};
\end{tikzpicture}
\end{document}
";

type Affine = [f64; 6];

fn mul(m: Affine, t: Affine) -> Affine {
  let [a, b, c, d, e, f] = m;
  let [aa, bb, cc, dd, ee, ff] = t;
  [
    a * aa + c * bb,
    b * aa + d * bb,
    a * cc + c * dd,
    b * cc + d * dd,
    a * ee + c * ff + e,
    b * ee + d * ff + f,
  ]
}

/// The composition of an SVG `transform` list (`translate(…)` and `matrix(…)`).
fn transform_of(attrs: &str) -> Affine {
  let list = Regex::new(r#"transform="([^"]*)""#).unwrap();
  let item = Regex::new(r"(\w+)\(([^)]*)\)").unwrap();
  let mut m = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
  let Some(list) = list.captures(attrs) else {
    return m;
  };
  for it in item.captures_iter(&list[1]) {
    let v: Vec<f64> = it[2]
      .split(|c: char| c == ',' || c.is_whitespace())
      .filter(|s| !s.is_empty())
      .map(|s| s.parse().unwrap())
      .collect();
    let t = match &it[1] {
      "translate" => [1.0, 0.0, 0.0, 1.0, v[0], *v.get(1).unwrap_or(&0.0)],
      "matrix" => [v[0], v[1], v[2], v[3], v[4], v[5]],
      other => panic!("unexpected transform {other}"),
    };
    m = mul(m, t);
  }
  m
}

/// A `foreignObject` as the viewer places it: the y of its baseline, and of its top and bottom, in the `svg:svg`'s user
/// space (y down). The object's `transform` ends in `matrix(1 0 0 -1 0 h)`, `h` the box's height, so the parent's origin
/// is the baseline.
struct Line {
  x:        f64,
  baseline: f64,
  top:      f64,
  bottom:   f64,
}

/// A path of the form `M x y h w v h -w Z` (a node's `draw`n rectangle), as its top and bottom y in user space.
struct Frame {
  top:    f64,
  bottom: f64,
}

/// Every `foreignObject` and `M x y h w v h -w Z` path of `xml`, in document order, placed by the `svg:g` transforms
/// around them.
fn placed(xml: &str) -> (Vec<Line>, Vec<Frame>) {
  let tag = Regex::new(r"<(/?)(svg:g|svg:foreignObject|svg:path|picture)\b([^>]*?)(/?)>").unwrap();
  let height = Regex::new(r#" height="([^"]*)""#).unwrap();
  let path =
    Regex::new(r#" d="M ([-\d.]+) ([-\d.]+) h ([-\d.]+) v ([-\d.]+) h [-\d.]+ Z""#).unwrap();
  let mut stack: Vec<Affine> = vec![[1.0, 0.0, 0.0, 1.0, 0.0, 0.0]];
  let (mut lines, mut frames) = (Vec::new(), Vec::new());
  for t in tag.captures_iter(xml) {
    let (closing, name, attrs, selfclosed) = (&t[1] == "/", &t[2], &t[3], &t[4] == "/");
    match name {
      "picture" if !closing => stack = vec![[1.0, 0.0, 0.0, 1.0, 0.0, 0.0]],
      "svg:g" if closing => {
        stack.pop();
      },
      "svg:g" => {
        let m = mul(*stack.last().unwrap(), transform_of(attrs));
        if !selfclosed {
          stack.push(m);
        }
      },
      "svg:foreignObject" if !closing => {
        let parent = *stack.last().unwrap();
        let own = mul(parent, transform_of(attrs));
        let h: f64 = height.captures(attrs).unwrap()[1].parse().unwrap();
        lines.push(Line {
          x:        own[4],
          baseline: parent[5],
          top:      own[5],
          bottom:   own[5] + h,
        });
      },
      "svg:path" => {
        if let Some(p) = path.captures(attrs) {
          let m = *stack.last().unwrap();
          let (y, dy): (f64, f64) = (p[2].parse().unwrap(), p[4].parse().unwrap());
          let (a, b) = (m[3] * y + m[5], m[3] * (y + dy) + m[5]);
          frames.push(Frame {
            top:    a.min(b),
            bottom: a.max(b),
          });
        }
      },
      _ => {},
    }
  }
  (lines, frames)
}

fn baselines(lines: &[Line]) -> Vec<f64> { lines.iter().map(|l| l.baseline).collect() }

fn assert_close(got: f64, want: f64, tol: f64, what: &str) {
  assert!(
    (got - want).abs() <= tol,
    "{what}: {got} vs {want} (±{tol})"
  );
}

/// The three lines of a left, a centred and a right aligned node are a `\baselineskip` apart (pdfTeX: `\box0` is
/// 30.94444pt high, 6.94444 + 12 + 12), and the `text width` node, a paragraph, is as it was.
#[test]
fn lines_of_a_multiline_node_are_a_baselineskip_apart() {
  let (stderr, xml) = convert(MULTILINE_NODES, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let (lines, _) = placed(&xml);
  assert_eq!(
    lines.len(),
    10,
    "three lines in each of three nodes, one paragraph:\n{xml}"
  );
  for (node, name) in [(0, "left"), (3, "center"), (6, "right")] {
    let b = baselines(&lines[node..node + 3]);
    assert_close(
      b[1] - b[0],
      BASELINESKIP_PX,
      0.05,
      &format!("{name}: line 1 to 2"),
    );
    assert_close(
      b[2] - b[1],
      BASELINESKIP_PX,
      0.05,
      &format!("{name}: line 2 to 3"),
    );
  }
  // The lines keep their sides: left flush, right flush and centred.
  assert_close(lines[0].x, lines[1].x, 0.05, "align=left: lines flush left");
  assert_close(lines[0].x, lines[2].x, 0.05, "align=left: lines flush left");
  assert_close(
    lines[4].x - lines[3].x,
    (86.98 - 33.09) / 2.0,
    0.05,
    "align=center: Short centred",
  );
  assert_close(
    lines[7].x - lines[6].x,
    86.98 - 33.09,
    0.05,
    "align=right: Short flush right",
  );
  // The `text width` node is one block of the same 44.89 px, and no matrix.
  let paragraph = &lines[9];
  assert_close(
    paragraph.bottom - paragraph.top,
    44.89,
    0.01,
    "text width node",
  );
}

/// A `draw`n multi-line node is as high as pdfTeX's box (30.94444pt + 1.49998pt = 44.89 px) plus the two `inner sep`s
/// (2 × 0.3333em = 9.22 px), and its rectangle encloses every line.
#[test]
fn drawn_multiline_node_encloses_all_its_lines() {
  let tex = r"\documentclass{article}
\usepackage{tikz}
\begin{document}
\begin{tikzpicture}
\node[draw,align=left] at (0,0) {Wide first line\\ Short\\ Third $x_1$};
\node[draw] at (0,-3) {One line};
\end{tikzpicture}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let (lines, frames) = placed(&xml);
  assert_eq!((lines.len(), frames.len()), (4, 2), "{xml}");
  let inner_sep = 4.61;
  let frame = &frames[0];
  assert_close(
    frame.bottom - frame.top,
    44.89 + 2.0 * inner_sep,
    0.1,
    "drawn node height",
  );
  // Its first line is an inner sep below the rectangle's top, its last (and the depth of `x_1`) an inner sep above its
  // bottom.
  assert_close(
    lines[0].baseline - 9.61 - frame.top,
    inner_sep,
    0.1,
    "top of the first line",
  );
  assert_close(
    frame.bottom - lines[2].bottom,
    inner_sep,
    0.1,
    "bottom of the last line",
  );
  // A one-line node is untouched.
  let one = &frames[1];
  assert_close(one.bottom - one.top, 18.83, 0.01, "one-line node height");
}

/// `\\[10pt]` is a `\noalign{\vskip10pt}` between two rows (tikz.code.tex:4231-4232 parse the
/// `[10pt]`, :4255 sets the skip): the lines are 10pt + 12pt apart, the
/// glue between the boxes adding to the interline glue it is followed by (§679: the depth above is that of the box).
#[test]
fn newline_skip_adds_to_the_baselineskip() {
  let tex = r"\documentclass{article}
\usepackage{tikz}
\begin{document}
\begin{tikzpicture}
\node[align=left] at (0,0) {Wide first line\\[10pt] Short\\ Third $x_1$};
\end{tikzpicture}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let (lines, _) = placed(&xml);
  assert_eq!(lines.len(), 3, "{xml}");
  let b = baselines(&lines);
  assert_close(
    b[1] - b[0],
    BASELINESKIP_PX + 13.8376,
    0.05,
    "line 1 to 2, after \\\\[10pt]",
  );
  assert_close(b[2] - b[1], BASELINESKIP_PX, 0.05, "line 2 to 3");
}

/// `\matrix` and `tikzcd` rows (`\offinterlineskip`) keep the spacing their cells and `row sep` give them; the
/// baselines are those of the tight stacking before interline glue was applied to the SVG alignments.
#[test]
fn matrix_and_tikzcd_rows_get_no_interline_glue() {
  let matrix = r"\documentclass{article}
\usepackage{tikz}
\usetikzlibrary{matrix}
\begin{document}
\begin{tikzpicture}
\matrix[matrix of nodes,row sep=1em,column sep=2em,nodes={draw}] {
 A & B \\ Cg & D$x_1$ \\ };
\matrix[matrix of math nodes] at (5,0) {
 a & b \\ c^2 & d_1 \\ };
\matrix[matrix,row sep=3mm] at (0,-4) {
 \node{one}; & \node{two}; \\ \node{three}; & \node{four}; \\ };
\end{tikzpicture}
\end{document}
";
  let (stderr, xml) = convert(matrix, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let (lines, _) = placed(&xml);
  assert_eq!(lines.len(), 12, "{xml}");
  let want = [
    18.96, 18.96, 52.03, 52.03, 25.63, 25.63, 46.12, 46.12, 177.25, 178.53, 209.17, 209.17,
  ];
  for (i, (got, want)) in baselines(&lines).iter().zip(want).enumerate() {
    assert_close(*got, want, 0.02, &format!("matrix line {i}"));
  }
  let cd = r"\documentclass{article}
\usepackage{tikz-cd}
\begin{document}
\begin{tikzcd}
A \arrow[r] \arrow[d] & B \arrow[d] \\
C \arrow[r] & D^2
\end{tikzcd}
\begin{tikzcd}[row sep=large,column sep=small]
A & B \\ C & D \\ E & F
\end{tikzcd}
\end{document}
";
  let (stderr, xml) = convert(cd, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let (lines, _) = placed(&xml);
  assert_eq!(lines.len(), 10, "{xml}");
  let want = [
    14.52, 14.52, 60.82, 60.82, 14.51, 14.51, 71.45, 71.45, 128.40, 128.40,
  ];
  for (i, (got, want)) in baselines(&lines).iter().zip(want).enumerate() {
    assert_close(*got, want, 0.02, &format!("tikzcd line {i}"));
  }
}

/// The interline parameters are those in force where the node's lines are set, as in pdfTeX (`\showbox`-measured:
/// the north-to-south distance of the `draw`n node less its 0.4pt line width): a node in a `\matrix` cell gets the
/// `\baselineskip` 12pt that pgf's `\normalbaselines` per cell restores (pgfmodulematrix.code.tex:357) although the
/// matrix itself runs `\offinterlineskip`; `font=\small` sets 11pt; a node in a `tabular` cell has `\lineskip` and
/// `\baselineskip` zero (`\@array`, latex.ltx), so its lines touch.
#[test]
fn interline_parameters_are_those_in_force_where_the_node_is_set() {
  let tex = r"\documentclass{article}
\usepackage{tikz}
\usetikzlibrary{matrix}
\begin{document}
\begin{tikzpicture}
\node[draw,align=left] at (0,0) {A\\ B\\ C};
\node[draw,align=left,font=\small] at (3,0) {A\\ B\\ C};
\matrix[matrix of nodes] at (6,0) {
 \node[draw,align=left]{A\\ B\\ C}; & x \\ };
\end{tikzpicture}

\begin{tabular}{c}
\tikz\node[draw,align=left]{A\\ B\\ C};
\end{tabular}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let (lines, frames) = placed(&xml);
  assert_eq!((lines.len(), frames.len()), (13, 4), "{xml}");
  let b = baselines(&lines);
  // (first line, baseline separation in px)
  for (name, first, skip) in [
    ("12pt", 0, BASELINESKIP_PX),
    ("font=\\small, 11pt", 3, 15.2214),
    ("matrix cell, 12pt", 6, BASELINESKIP_PX),
    ("tabular cell, touching", 10, 9.46),
  ] {
    assert_close(
      b[first + 1] - b[first],
      skip,
      0.05,
      &format!("{name}: line 1 to 2"),
    );
    assert_close(
      b[first + 2] - b[first + 1],
      skip,
      0.05,
      &format!("{name}: line 2 to 3"),
    );
  }
  // pdfTeX's node heights less the line width, in px.
  for (frame, height) in frames.iter().zip([51.89, 48.17, 51.89, 37.59]) {
    assert_close(frame.bottom - frame.top, height, 0.05, "node height");
  }
}
