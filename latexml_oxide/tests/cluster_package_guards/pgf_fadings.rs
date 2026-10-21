//! TikZ path fadings (`path fading=`, `\pgfsetfading`) are `svg:mask`s of the fading's box
//! (OXIDIZED_DESIGN_DIVERGENCES #615; Perl's `\pgfsys@fadingfrombox`/`\pgfsys@usefading` are `# NOT IMPLEMENTED`
//! no-ops, pgfsys-latexml.def.ltxml:811-824, so a faded fill came out opaque).

use latexml::util::test::assert_element;

use super::{convert, error_count, perfect_kernel_batch46::warning_count};

const PREAMBLE: &str = "\\documentclass{article}\n\\usepackage{tikz}\n\\usetikzlibrary{fadings}\n";

/// The witness: a white rectangle fading from opaque (left) to transparent (right) over a blue one.
const FADED_FILL: &str = "\\tikzfading[name=fade right, left color=transparent!0, right color=transparent!100]\n\
  \\begin{document}\n\\begin{tikzpicture}\n  \\fill[blue!20] (0,0) rectangle (4,2);\n  \\draw (0,1) -- (4,1);\n\
  \\fill[white, path fading=fade right, fit fading=false,\n    fading transform={xscale=2.8, yscale=2.2}] (-1,-0.5) rectangle (2,2.5);\n\
  \\end{tikzpicture}\n\\end{document}\n";

fn witness() -> String {
  let (stderr, xml) = convert(&format!("{PREAMBLE}{FADED_FILL}"), true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  if let Some(lines) = latexml::util::test::rng_error_count(&xml) {
    assert_eq!(lines, 0, "jing:\n{xml}");
  }
  xml
}

/// The fading's picture (`\tikzfading`: a 100bp square shading, turned a quarter) becomes the content of an
/// `svg:mask` — white is opaque, as in the soft mask `/S /Luminosity` of the pdf drivers
/// (pgfsys-pdftex.def:284-296) — placed as they place it: the box centred on the origin (`translate(-69.44 -69.44)`,
/// 100bp in pixels), the fading's own `xscale=2.8, yscale=2.2` applied to it. The gradient runs from black at the
/// right (transparent) to white at the left (opaque).
#[test]
fn a_path_fading_is_a_mask_of_the_fading_picture() {
  assert_element(
    &witness(),
    "svg:defs",
    &[],
    r##"<svg:defs color="#FFFFFF">
      <svg:mask height="200000" id="pgffade10" maskUnits="userSpaceOnUse" width="200000" x="-100000" y="-100000">
        <svg:g fill="#FFFFFF" stroke="#FFFFFF" transform="matrix(2.8 0.0 0.0 2.2 0 0) translate(-69.44 -69.44)">
          <svg:g class="ltx_nestedsvg" color="#000000" fill="#000000" stroke="#000000" stroke-width="0.4pt" transform="matrix(1 0 0 1 0 0) matrix(0.0 1.0 -1.0 0.0 138.89 0)">
            <svg:g class="ltx_nestedsvg" transform="matrix(1 0 0 1 0 0)">
              <svg:defs>
                <svg:linearGradient gradientTransform="rotate(90)" id="pgfsh9">
                  <svg:stop offset="0.0" stop-color="#000000"/>
                  <svg:stop offset="0.24998" stop-color="#000000"/>
                  <svg:stop offset="0.49996" stop-color="#808080"/>
                  <svg:stop offset="0.75002" stop-color="#FFFFFF"/>
                  <svg:stop offset="1.0" stop-color="#FFFFFF"/>
                </svg:linearGradient>
              </svg:defs>
              <svg:rect height="138.89" style="fill:url(#pgfsh9);stroke:none" width="138.89"/>
            </svg:g>
          </svg:g>
        </svg:g>
      </svg:mask>
    </svg:defs>"##,
  );
}

/// The mask is referenced by the group of the faded path, which is what the pdf driver's `gs` is for the path; the
/// white rectangle itself is drawn unchanged inside it.
#[test]
fn the_faded_path_is_drawn_in_a_group_that_refers_to_the_mask() {
  assert_element(
    &witness(),
    "svg:g",
    &[r##"mask="url(#pgffade10)""##],
    r##"<svg:g color="#FFFFFF" fill="#FFFFFF" mask="url(#pgffade10)" stroke="#FFFFFF">
      <svg:path d="M -39.37 -19.69 M -39.37 -19.69 L -39.37 98.43 L 78.74 98.43 L 78.74 -19.69 Z M 78.74 98.43" style="stroke:none"/>
    </svg:g>"##,
  );
}

/// The fading is part of the graphic state of the scope that sets it: the paths after the faded one are not masked,
/// and the mask belongs to the faded one's group alone.
#[test]
fn a_fading_ends_with_its_scope() {
  let tex = format!(
    "{PREAMBLE}\\tikzfading[name=fade right, left color=transparent!0, right color=transparent!100]\n\
     \\begin{{document}}\n\\begin{{tikzpicture}}\n\\fill[red, path fading=fade right] (0,0) rectangle (2,1);\n\
     \\draw (0,2) -- (2,2);\n\\end{{tikzpicture}}\n\\end{{document}}\n"
  );
  let (stderr, xml) = convert(&tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("mask=\"url(").count(), 1, "{xml}");
  assert_element(
    &xml,
    "svg:g",
    &[r##"mask="url(#pgffade10)""##],
    r##"<svg:g color="#FF0000" fill="#FF0000" mask="url(#pgffade10)" stroke="#FF0000">
      <svg:path d="M 0 0 M 0 0 L 0 39.37 L 78.74 39.37 L 78.74 0 Z M 78.74 39.37" style="stroke:none"/>
    </svg:g>"##,
  );
  assert_element(
    &xml,
    "svg:path",
    &[r#"d="M 0 78.74 L 78.74 78.74""#],
    r##"<svg:path d="M 0 78.74 L 78.74 78.74" style="fill:none"/>"##,
  );
  let faded = xml.find("mask=\"url(").unwrap();
  let line = xml.find("M 0 78.74 L 78.74 78.74").unwrap();
  assert!(
    xml[faded..line].contains("</svg:g>"),
    "the line is drawn after the faded group closes:\n{xml}"
  );
}

/// A fading of the library (`pgflibraryfadings.code.tex:21-40`: `\pgfdeclarefading{east}{\pgfuseshading{…}}`, the
/// shading typeset outside any picture) goes through the same two macros: its mask holds that picture alone, and
/// the default `fit fading` has scaled the mask to the path (the pdf driver's `\pgfsetfadingforcurrentpath`).
#[test]
fn a_library_fading_masks_the_path() {
  let tex = format!(
    "{PREAMBLE}\\begin{{document}}\n\\begin{{tikzpicture}}\n\\fill[red, path fading=east] (0,0) rectangle (2,1);\n\
     \\end{{tikzpicture}}\n\\end{{document}}\n"
  );
  let (stderr, xml) = convert(&tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  if let Some(lines) = latexml::util::test::rng_error_count(&xml) {
    assert_eq!(lines, 0, "jing:\n{xml}");
  }
  assert_element(
    &xml,
    "svg:defs",
    &[],
    r##"<svg:defs color="#FF0000">
      <svg:mask height="200000" id="pgffade9" maskUnits="userSpaceOnUse" width="200000" x="-100000" y="-100000">
        <svg:g fill="#FFFFFF" stroke="#FFFFFF" transform="matrix(1.13835 0.0 0.0 0.56917 39.37 19.69) translate(-69.44 -69.44)">
          <svg:g class="ltx_nestedsvg" color="#FFFFFF" transform="matrix(1 0 0 1 0 0)">
            <svg:defs>
              <svg:linearGradient id="pgfsh1">
                <svg:stop offset="0.0" stop-color="#FFFFFF"/>
                <svg:stop offset="0.24998" stop-color="#FFFFFF"/>
                <svg:stop offset="0.75002" stop-color="#000000"/>
                <svg:stop offset="1.0" stop-color="#000000"/>
              </svg:linearGradient>
            </svg:defs>
            <svg:rect height="138.89" style="fill:url(#pgfsh1);stroke:none" width="138.89"/>
          </svg:g>
        </svg:g>
      </svg:mask>
    </svg:defs>"##,
  );
}

/// Each use of a fading has a mask of its own (the cm of the use is in it); the gradient ids inside the copies of the
/// fading's box are made unique like those of any copied picture (`Document::record_svg_ids`), so every
/// `url(#…)` finds its own definition.
#[test]
fn every_use_of_a_fading_has_its_own_mask_and_ids() {
  let tex = format!(
    "{PREAMBLE}\\begin{{document}}\n\\begin{{tikzpicture}}\n\\fill[red, path fading=east] (0,0) rectangle (2,1);\n\
     \\fill[green, path fading=east] (0,2) rectangle (2,3);\n\\end{{tikzpicture}}\n\\end{{document}}\n"
  );
  let (stderr, xml) = convert(&tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let mut ids: Vec<&str> = xml
    .split(" id=\"")
    .skip(1)
    .map(|rest| &rest[..rest.find('"').unwrap()])
    .filter(|id| !id.starts_with("p1"))
    .collect();
  ids.sort_unstable();
  assert_eq!(ids, ["pgffade10", "pgffade9", "pgfsh1", "pgfsh1a"], "{xml}");
  for id in ids {
    assert_eq!(
      xml.matches(&format!("url(#{id})")).count(),
      1,
      "each id referenced once, by its own copy: {id}\n{xml}"
    );
  }
}

/// pgf: `\pgferror{Undefined fading '#1'}` (pgfsys-pdftex.def:286). The path is drawn, unmasked, and the error
/// is the only diagnostic.
#[test]
fn an_undefined_fading_is_an_error_and_masks_nothing() {
  let tex = format!(
    "{PREAMBLE}\\begin{{document}}\n\\begin{{tikzpicture}}\n\\fill[red, path fading=nosuchfading] (0,0) rectangle (2,1);\n\
     \\end{{tikzpicture}}\n\\end{{document}}\n"
  );
  let (stderr, xml) = convert(&tex, true);
  assert_eq!(error_count(&stderr), 1, "{stderr}");
  assert!(
    stderr.contains("Error:undefined:fading Undefined fading 'nosuchfading'"),
    "{stderr}"
  );
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(!xml.contains("svg:mask") && !xml.contains("mask="), "{xml}");
  assert_element(
    &xml,
    "svg:path",
    &[],
    r##"<svg:path d="M 0 0 M 0 0 L 0 39.37 L 78.74 39.37 L 78.74 0 Z M 78.74 39.37" style="stroke:none"/>"##,
  );
}

/// A shading is as large as it was declared. The driver made `\lxSVG@pos` (the shading's size, for the box and the
/// bounding box) the pixel size read as points, 1.38 times too large (OXIDIZED_DESIGN_DIVERGENCES #616): the
/// picture of a `\pgfuseshading` in text was 192px wide round a 139px rectangle, and a node holding one put the
/// shading 27px off its centre. The shading here is 100bp = 138.89px; its box is the same.
#[test]
fn a_shading_has_the_size_it_was_declared() {
  let tex = "\\documentclass{article}\n\\usepackage{tikz}\n\
    \\pgfdeclarehorizontalshading{sh}{100bp}{color(0bp)=(red); color(100bp)=(blue)}\n\
    \\begin{document}\n\\pgfuseshading{sh}\n\n\
    \\tikz \\node[inner sep=0pt] at (50bp,50bp) {\\pgfuseshading{sh}};\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  // In text: the picture is the rectangle's size.
  assert_element(
    &xml,
    "svg:svg",
    &[r#"viewBox="0 0 138.89 138.89""#],
    r##"<svg:svg height="138.89" overflow="visible" version="1.1" viewBox="0 0 138.89 138.89" width="138.89">
      <svg:g transform="translate(0,138.89) matrix(1 0 0 -1 0 0)">
        <svg:defs>
          <svg:linearGradient id="pgfsh1">
            <svg:stop offset="0.0" stop-color="#FF0000"/>
            <svg:stop offset="1.0" stop-color="#0000FF"/>
          </svg:linearGradient>
        </svg:defs>
        <svg:rect height="138.89" style="fill:url(#pgfsh1);stroke:none" width="138.89"/>
      </svg:g>
    </svg:svg>"##,
  );
  // In a node: centred on the node, whose group is not shifted (it was `-26.65 -26.65`).
  assert_element(
    &xml,
    "svg:g",
    &[r##"transform="translate(0,138.89) matrix(1 0 0 -1 0 0) matrix(1.0 0.0 0.0 1.0 0 0)""##],
    r##"<svg:g fill="#000000" stroke="#000000" stroke-width="0.4pt" transform="translate(0,138.89) matrix(1 0 0 -1 0 0) matrix(1.0 0.0 0.0 1.0 0 0)">
      <svg:g class="ltx_nestedsvg" transform="matrix(1 0 0 1 0 0)">
        <svg:defs>
          <svg:linearGradient id="pgfsh2">
            <svg:stop offset="0.0" stop-color="#FF0000"/>
            <svg:stop offset="1.0" stop-color="#0000FF"/>
          </svg:linearGradient>
        </svg:defs>
        <svg:rect height="138.89" style="fill:url(#pgfsh2);stroke:none" width="138.89"/>
      </svg:g>
    </svg:g>"##,
  );
}

/// A fading replaces the one in force in its scope (the soft mask of the graphics state, set again by the pdf
/// driver's `gs`), where the masks of nested groups would intersect. The second `\pgfsetfading` closes the group of
/// the first and the groups the scope opened after it (the fill opacity here), and reopens those; the drawing after
/// it is under the second mask alone. unit_tests_by_silviu, "Transparency": the second half of a path had only the
/// part of its fading that the first one also covered.
#[test]
fn a_second_fading_replaces_the_first_in_its_scope() {
  let tex = format!(
    "{PREAMBLE}\\pgfdeclarehorizontalshading{{sh}}{{100bp}}{{color(0pt)=(white); color(100bp)=(black)}}\n\
     \\pgfdeclarefading{{fa}}{{\\pgfuseshading{{sh}}}}\n\\begin{{document}}\n\\begin{{tikzpicture}}\n\
     \\pgfsetfading{{fa}}{{}}\n\\fill[red] (0,0) rectangle (1,1);\n\\pgfsetfillopacity{{.5}}\n\
     \\pgfsetfading{{fa}}{{\\pgftransformshift{{\\pgfpoint{{1cm}}{{1cm}}}}}}\n\\fill[green] (0,2) rectangle (1,3);\n\
     \\end{{tikzpicture}}\n\\end{{document}}\n"
  );
  let (stderr, xml) = convert(&tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  if let Some(lines) = latexml::util::test::rng_error_count(&xml) {
    assert_eq!(lines, 0, "jing:\n{xml}");
  }
  assert_eq!(xml.matches(" mask=\"url(").count(), 2, "{xml}");
  assert_element(
    &xml,
    "svg:g",
    &[r##"transform="translate(0,118.11) matrix(1 0 0 -1 0 0)""##],
    r##"<svg:g fill="#000000" stroke="#000000" stroke-width="0.4pt" transform="translate(0,118.11) matrix(1 0 0 -1 0 0)">
      <svg:defs>
        <svg:mask height="200000" id="pgffade10" maskUnits="userSpaceOnUse" width="200000" x="-100000" y="-100000">
          <svg:g fill="#FFFFFF" stroke="#FFFFFF" transform="matrix(1.0 0.0 0.0 1.0 0 0) translate(-69.44 -69.44)">
            <svg:g class="ltx_nestedsvg" color="#FFFFFF" transform="matrix(1 0 0 1 0 0)">
              <svg:defs>
                <svg:linearGradient id="pgfsh9">
                  <svg:stop offset="0.0" stop-color="#FFFFFF"/>
                  <svg:stop offset="1.0" stop-color="#000000"/>
                </svg:linearGradient>
              </svg:defs>
              <svg:rect height="138.89" style="fill:url(#pgfsh9);stroke:none" width="138.89"/>
            </svg:g>
          </svg:g>
        </svg:mask>
      </svg:defs>
      <svg:g mask="url(#pgffade10)">
        <svg:g color="#FF0000" fill="#FF0000" stroke="#FF0000">
          <svg:path d="M 0 0 M 0 0 L 0 39.37 L 39.37 39.37 L 39.37 0 Z M 39.37 39.37" style="stroke:none"/>
        </svg:g>
        <svg:g fill-opacity="0.5">
          <svg:defs>
            <svg:mask height="200000" id="pgffade11" maskUnits="userSpaceOnUse" width="200000" x="-100000" y="-100000">
              <svg:g fill="#FFFFFF" stroke="#FFFFFF" transform="matrix(1.0 0.0 0.0 1.0 39.37 39.37) translate(-69.44 -69.44)">
                <svg:g class="ltx_nestedsvg" color="#FFFFFF" transform="matrix(1 0 0 1 0 0)">
                  <svg:defs>
                    <svg:linearGradient id="pgfsh9a">
                      <svg:stop offset="0.0" stop-color="#FFFFFF"/>
                      <svg:stop offset="1.0" stop-color="#000000"/>
                    </svg:linearGradient>
                  </svg:defs>
                  <svg:rect height="138.89" style="fill:url(#pgfsh9a);stroke:none" width="138.89"/>
                </svg:g>
              </svg:g>
            </svg:mask>
          </svg:defs>
        </svg:g>
      </svg:g>
      <svg:g fill-opacity="0.5">
        <svg:g color="#00FF00" fill="#00FF00" mask="url(#pgffade11)" stroke="#00FF00">
          <svg:path d="M 0 78.74 M 0 78.74 L 0 118.11 L 39.37 118.11 L 39.37 78.74 Z M 39.37 118.11" style="stroke:none"/>
        </svg:g>
      </svg:g>
    </svg:g>"##,
  );
}

/// A fading made of text (`\pgfdeclarefading{…}{\color{white}Ti\emph{k}Z}`) has its text in a `foreignObject`, which
/// browsers do not paint inside a mask (Chromium): the mask would hide the path entirely, worse than the opaque
/// drawing of before. The path is left unmasked, with one warning naming the fading, and no mask is written.
#[test]
fn a_text_fading_leaves_the_path_unmasked_with_a_warning() {
  let tex = format!(
    "{PREAMBLE}\\pgfdeclarefading{{tikz}}{{\\color{{white}}Ti\\emph{{k}}Z}}\n\\begin{{document}}\n\
     \\begin{{tikzpicture}}\n\\pgfsetfading{{tikz}}{{}}\n\\fill[red] (0,0) rectangle (2,2);\n\\end{{tikzpicture}}\n\
     \\end{{document}}\n"
  );
  let (stderr, xml) = convert(&tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  assert!(
    stderr.contains("Warning:unexpected:fading The fading 'tikz' has text in its box"),
    "{stderr}"
  );
  if let Some(lines) = latexml::util::test::rng_error_count(&xml) {
    assert_eq!(lines, 0, "jing:\n{xml}");
  }
  assert!(
    !xml.contains("svg:mask") && !xml.contains("mask=") && !xml.contains("foreignObject"),
    "{xml}"
  );
  assert_element(
    &xml,
    "svg:g",
    &[r##"transform="translate(0,78.74) matrix(1 0 0 -1 0 0)""##],
    r##"<svg:g color="#FF0000" fill="#FF0000" stroke="#FF0000" stroke-width="0.4pt" transform="translate(0,78.74) matrix(1 0 0 -1 0 0)">
      <svg:path d="M 0 0 M 0 0 L 0 78.74 L 78.74 78.74 L 78.74 0 Z M 78.74 78.74" style="stroke:none"/>
    </svg:g>"##,
  );
}

/// A mask is evaluated in the user space of the group that refers to it, that group's `transform` included, where
/// the soft mask of the PDF graphics state stays where `gs` set it. `collapse_svg_group` merges a lone child
/// `svg:g` into its parent, `transform` too (Perl's rule, TeX_Box.pool.ltxml:296-313, which leaves a `clip-path` group
/// alone, :250, for the same reason); merged into the group of a mask, a shifted scope would shift the mask. Here the fill is
/// drawn 1cm to the right of the fading's place, in a scope the fading was set outside of.
#[test]
fn a_transformed_scope_does_not_move_the_mask_of_its_group() {
  let tex = format!(
    "{PREAMBLE}\\begin{{document}}\n\\begin{{tikzpicture}}\n\\pgfsetfading{{east}}{{}}\n\
     \\begin{{pgfscope}}\\pgftransformshift{{\\pgfpoint{{1cm}}{{0cm}}}}\\pgflowlevelsynccm\n\
     \\fill[red] (0,0) rectangle (2,1);\n\\end{{pgfscope}}\n\\end{{tikzpicture}}\n\\end{{document}}\n"
  );
  let (stderr, xml) = convert(&tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "svg:g",
    &[r##"mask="url(#pgffade9)""##],
    r##"<svg:g mask="url(#pgffade9)">
      <svg:g color="#FF0000" fill="#FF0000" stroke="#FF0000" transform="matrix(1.0 0.0 0.0 1.0 39.37 0)">
        <svg:path d="M 0 0 M 0 0 L 0 39.37 L 78.74 39.37 L 78.74 0 Z M 78.74 39.37" style="stroke:none"/>
      </svg:g>
    </svg:g>"##,
  );
}
