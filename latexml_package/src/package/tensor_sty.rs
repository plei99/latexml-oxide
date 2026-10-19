//! tensor.sty (v2.2, 2023/07/18) — tensor indices, staggered or aligned, before and after a symbol.
//!
//! No Perl binding exists; native binding, OXIDIZED_DESIGN #588(g). The package typesets
//! `\tensor{T}{^a_b^c}` as one superscript and one subscript whose items are padded with
//! `\hphantom`s of each other, so every index has its own column (tensor.sty:154-162,
//! `\tnsr@Add`); `\tensor*` drops the phantoms, so the superscripts run together over the
//! subscripts (`\tnsr@Spcfalse`). Here the columns are made explicit: the indices become the
//! script pairs of one multiscript application (`\lx@multiscripts`, the nested
//! `SUPERSCRIPTOP`/`SUBSCRIPTOP` shape amsmath's `\sideset` builds), which the MathML stage
//! writes as `<mmultiscripts>`, one pair per column, an empty `<mrow/>` for a slot a column does
//! not fill. The kerns, smashes and struts of the `align`/`nosmash`/`nostrut` options only
//! adjust the printed spacing.
use latexml_core::digested::DigestedData;

use crate::prelude::*;

/// One index item of an index string: `true` for a superscript.
type IndexItem = (bool, Tokens);

/// Split an index string (`^a_{bc}^\mu`) into its items, as `\tnsr@Pro` reads it
/// (tensor.sty:102-132): `^` or `_` then one undelimited argument; a `*` toggles width matching
/// in starred use and is not an index. Returns the items and whether a token other than these
/// was found (the package's "Misordered sub/superscript items" error).
fn index_items(spec: &Tokens) -> (Vec<IndexItem>, bool, bool) {
  let toks = spec.clone().unlist();
  let mut items = Vec::new();
  let mut misordered = false;
  let mut star = false;
  let mut i = 0;
  let skip_spaces = |i: &mut usize| {
    while *i < toks.len() && toks[*i].get_catcode() == Catcode::SPACE {
      *i += 1;
    }
  };
  while i < toks.len() {
    let t = &toks[i];
    let cc = t.get_catcode();
    if cc == Catcode::SPACE {
      i += 1;
      continue;
    }
    let sup = match cc {
      Catcode::SUPER => Some(true),
      Catcode::SUB => Some(false),
      _ => None,
    };
    if cc == Catcode::OTHER && t.with_str(|s| s == "*") {
      star = true;
      i += 1;
      continue;
    }
    let Some(sup) = sup else {
      misordered = true;
      i += 1;
      continue;
    };
    i += 1;
    skip_spaces(&mut i);
    if i >= toks.len() {
      misordered = true;
      break;
    }
    // One undelimited argument: a token or a balanced group, braces stripped.
    if toks[i].get_catcode() == Catcode::BEGIN {
      let mut depth = 0usize;
      let start = i + 1;
      while i < toks.len() {
        match toks[i].get_catcode() {
          Catcode::BEGIN => depth += 1,
          Catcode::END => {
            depth -= 1;
            if depth == 0 {
              break;
            }
          },
          _ => {},
        }
        i += 1;
      }
      items.push((sup, Tokens::new(toks[start..i.min(toks.len())].to_vec())));
      i += 1;
    } else {
      items.push((sup, Tokens::new(vec![toks[i]])));
      i += 1;
    }
  }
  (items, misordered, star)
}

/// The columns of an index string as `\lx@ms@col{sub}{sup}` calls: one column per item when
/// staggered (`\tnsr@Spctrue`), one column holding all superscripts and all subscripts, each in
/// order, when aligned.
fn index_columns(items: &[IndexItem], staggered: bool) -> Vec<Token> {
  let mut out = Vec::new();
  let mut col = |sub: &[Token], sup: &[Token]| {
    out.push(T_CS!("\\lx@ms@col"));
    out.push(T_BEGIN!());
    out.extend_from_slice(sub);
    out.push(T_END!());
    out.push(T_BEGIN!());
    out.extend_from_slice(sup);
    out.push(T_END!());
  };
  if staggered {
    for (sup, item) in items {
      let braced = braced(item);
      if *sup {
        col(&[], &braced);
      } else {
        col(&braced, &[]);
      }
    }
  } else if !items.is_empty() {
    let mut sups = Vec::new();
    let mut subs = Vec::new();
    for (sup, item) in items {
      let braced = braced(item);
      if *sup {
        sups.extend(braced);
      } else {
        subs.extend(braced);
      }
    }
    col(&subs, &sups);
  }
  out
}

fn braced(item: &Tokens) -> Vec<Token> {
  let mut v = vec![T_BEGIN!()];
  v.extend(item.clone().unlist());
  v.push(T_END!());
  v
}

/// `\lx@multiscripts{written}{pre columns}{base}{post columns}` for a tensor-like command; the
/// first argument is the call as written (its reversion). Shared with leftindex_sty.rs.
pub(crate) fn multiscripts_call(
  written: Vec<Token>,
  pre: Vec<Token>,
  base: Vec<Token>,
  post: Vec<Token>,
) -> Tokens {
  let mut out = vec![T_CS!("\\lx@multiscripts"), T_BEGIN!()];
  out.extend(written);
  out.push(T_END!());
  for part in [pre, base, post] {
    out.push(T_BEGIN!());
    out.extend(part);
    out.push(T_END!());
  }
  Tokens::new(out)
}

/// The `\lx@ms@col` whatsits of a digested column list, as (sub, sup) pairs.
fn columns_of(list: Option<&Digested>) -> Vec<(Option<Digested>, Option<Digested>)> {
  let Some(list) = list else {
    return Vec::new();
  };
  let mut cols = Vec::new();
  for item in list.unlist() {
    if let DigestedData::Whatsit(w) = item.data() {
      let w = w.borrow();
      let is_col = w
        .get_definition()
        .get_cs()
        .with_cs_name(|name| name == "\\lx@ms@col");
      if is_col {
        let nonempty = |a: Option<&Digested>| {
          a.filter(|d| !d.unlist().is_empty() && !d.is_empty().unwrap_or(false))
            .cloned()
        };
        cols.push((nonempty(w.get_arg(1)), nonempty(w.get_arg(2))));
      }
    }
  }
  cols
}

/// Wrap `inner` in a script application: `<XMApp><XMTok role=y scriptpos=xN/><inner/><XMWrap>script</XMWrap></XMApp>`,
/// the shape of amsmath's `sidesetWrap` (amsmath_sty.rs `sideset_wrap_impl`, Perl
/// amsmath.sty.ltxml:1227-1234), whose `scriptpos` level names the column.
fn script_wrap(
  document: &mut Document,
  inner: Node,
  pos: &str,
  sup: bool,
  level: usize,
  script: &Digested,
) -> Result<Node> {
  document.open_element("ltx:XMApp", None, None)?;
  let mut tok_attrs: HashMap<String, String> = HashMap::default();
  tok_attrs.insert(
    "role".to_string(),
    if sup { "SUPERSCRIPTOP" } else { "SUBSCRIPTOP" }.to_string(),
  );
  tok_attrs.insert("scriptpos".to_string(), format!("{pos}{level}"));
  document.open_element("ltx:XMTok", Some(tok_attrs), None)?;
  document.close_element("ltx:XMTok")?;
  let mut current = document.get_node().clone();
  document.append_tree(&mut current, vec![inner.clone()])?;
  document.remove_node_keeping_box(inner);
  document.open_element("ltx:XMWrap", None, None)?;
  document.absorb(script, None)?;
  document.close_element("ltx:XMWrap")?;
  let closed = document.close_element("ltx:XMApp")?;
  Ok(closed.unwrap_or_else(|| document.get_node().clone()))
}

/// The multiscript constructors shared by tensor and leftindex: `\lx@ms@col{sub}{sup}` is one
/// script column, `\lx@multiscripts{written}{pre columns}{base}{post columns}` the application.
#[rustfmt::skip]
pub(crate) fn define_multiscripts() -> Result<()> {
  DefConstructor!("\\lx@ms@col InScriptStyle InScriptStyle", "");
  DefConstructor!("\\lx@multiscripts Undigested {}{}{}", sub[document, args, props] {
    let pre = columns_of(args[1].as_ref());
    let post = columns_of(args[3].as_ref());
    document.open_element("ltx:XMArg", None, None)?;
    if let Some(base) = &args[2] {
      document.absorb(base, None)?;
    }
    let mut node = document.close_element("ltx:XMArg")?.unwrap_or_else(|| document.get_node().clone());
    let level0 = props.get("scriptlevel")
      .map(|v| v.to_string().parse::<usize>().unwrap_or(0)).unwrap_or(0);
    // Prescripts are wrapped innermost first, the column nearest the base first (as `\sideset`
    // walks its prescripts in reverse); each column its own level.
    let mut level = level0;
    for (sub, sup) in pre.iter().rev() {
      if let Some(s) = sub { node = script_wrap(document, node, "pre", false, level, s)?; }
      if let Some(s) = sup { node = script_wrap(document, node, "pre", true, level, s)?; }
      level += 1;
    }
    for (sub, sup) in &post {
      level += 1;
      if let Some(s) = sub { node = script_wrap(document, node, "post", false, level, s)?; }
      if let Some(s) = sup { node = script_wrap(document, node, "post", true, level, s)?; }
    }
    document.set_attribute(&mut node, "scriptpos", &format!("post{level0}"))?;
  },
  properties => { Ok(stored_map!("scriptlevel" => get_script_level())) },
  reversion => "#1");
  // `\lx@ms@postscripts{written}{columns}`: the columns as postscripts of whatever precedes
  // (tensor's `\indices`, whose object is the preceding atom): one `POSTSUPERSCRIPT` /
  // `POSTSUBSCRIPT` item per index, its `scriptpos` level naming its column, which the math
  // parser keeps when it attaches the scripts to their base (parser.rs `new_script_node`).
  DefConstructor!("\\lx@ms@postscripts Undigested {}", sub[document, args, props] {
    let cols = columns_of(args[1].as_ref());
    let level0 = props.get("scriptlevel")
      .map(|v| v.to_string().parse::<usize>().unwrap_or(0)).unwrap_or(0);
    for (k, (sub, sup)) in cols.iter().enumerate() {
      for (role, rule, script) in [
        ("POSTSUBSCRIPT", "Subscript", sub),
        ("POSTSUPERSCRIPT", "Superscript", sup),
      ] {
        let Some(script) = script else { continue };
        let mut attrs: HashMap<String, String> = HashMap::default();
        attrs.insert("role".to_string(), role.to_string());
        attrs.insert("scriptpos".to_string(), format!("post{}", level0 + 1 + k));
        document.open_element("ltx:XMApp", Some(attrs), None)?;
        let mut arg_attrs: HashMap<String, String> = HashMap::default();
        arg_attrs.insert("rule".to_string(), rule.to_string());
        document.open_element("ltx:XMArg", Some(arg_attrs), None)?;
        document.absorb(script, None)?;
        document.close_element("ltx:XMArg")?;
        document.close_element("ltx:XMApp")?;
      }
    }
  },
  properties => { Ok(stored_map!("scriptlevel" => get_script_level())) },
  reversion => "#1");
  Ok(())
}

#[rustfmt::skip]
LoadDefinitions!({
  // tensor.sty:28-36: options. `align`, `nosmash` and `nostrut` only change spacing; `text`
  // makes the struts apply in text math too.
  RawTeX!(r"\newif\iftnsr@Aln \newif\iftnsr@Txt \newif\iftnsr@Sma \tnsr@Smatrue \newif\iftnsr@Str \tnsr@Strtrue");
  DeclareOption!("align", { RawTeX!(r"\tnsr@Alntrue"); });
  DeclareOption!("text", { RawTeX!(r"\tnsr@Txttrue"); });
  DeclareOption!("nosmash", { RawTeX!(r"\tnsr@Smafalse"); });
  DeclareOption!("nostrut", { RawTeX!(r"\tnsr@Strfalse"); });
  ProcessOptions!();

  // The package's user hooks (tensor.sty:49-50, :168, :173, :179), definable as in the package.
  RawTeX!(r"\newcommand\nuclideFont{\mathrm}\newcommand\massnumFont{\nuclideFont}\newcommand\indexmarker{}\newcommand\tensorStrut{}\let\tensorSmash\relax");

  define_multiscripts()?;

  // tensor.sty:37-40, :70-74 `\tensor`, `\tensor*`: `[#1]` prescripts, `{#2}` the object, `{#3}` indices.
  DefMacro!("\\tensor OptionalMatch:* [] {} {}", sub[(star, pre, base, post)] {
    tensor_expansion(star.is_some(), "\\tensor", pre, Some(base), post)
  }, robust => true);
  // tensor.sty:41-44, :63-69 `\indices`, `\indices*`: indices for the preceding object.
  DefMacro!("\\indices OptionalMatch:* {}", sub[(star, post)] {
    tensor_expansion(star.is_some(), "\\indices", None, None, post)
  }, robust => true);
  // tensor.sty:45-48, :191-196 `\nuclide[A][Z]{X}`: `\DeclareRobustCommand\nuclide[1][]{\ncld@Mno{#1}\ncld@Aux}`,
  // `\ncld@Aux[2][]`: the mass number, then the atomic number, then the element, as
  // `\tensor*[^{A}_{Z}]{\mathit{\nuclideFont{X}}}{}`
  // in math (`\ensuremath`).
  DefMacro!("\\nuclide[][]{}",
    "\\ensuremath{\\tensor*[^{\\massnumFont{#1}}_{\\massnumFont{#2}}]{\\mkern1mu{\\mathit{\\nuclideFont{#3}}{}}}{}}",
    robust => true);
});

/// The expansion of `\tensor`/`\indices`: the multiscript call, after the package's diagnostics
/// (a `*` in an unstarred index string is a warning and ignored, tensor.sty:106-113; any other
/// token an error, :125, :198-207).
fn tensor_expansion(
  starred: bool,
  cs: &str,
  pre: Option<Tokens>,
  base: Option<Tokens>,
  post: Tokens,
) -> Result<Tokens> {
  let mut out = Vec::new();
  let mut written = vec![T_CS!(cs)];
  if starred {
    written.push(T_OTHER!("*"));
  }
  let pre_tokens = pre.unwrap_or_default();
  if !pre_tokens.is_empty() {
    written.push(T_OTHER!("["));
    written.extend(pre_tokens.clone().unlist());
    written.push(T_OTHER!("]"));
  }
  if let Some(b) = &base {
    written.extend(braced(b));
  }
  written.extend(braced(&post));
  let (pre_items, pre_bad, pre_star) = index_items(&pre_tokens);
  let (post_items, post_bad, post_star) = index_items(&post);
  if !starred && (pre_star || post_star) {
    out.extend(mouth::tokenize_internal(
      r"\PackageWarning{tensor}{'*' not allowed in argument here; I shall ignore it.\MessageBreak Either remove it or use '\string\tensor*'}",
    ).unlist());
  }
  if pre_bad || post_bad {
    out.extend(mouth::tokenize_internal(
      r"\PackageError{tensor}{Misordered sub/superscript items; \MessageBreak index tokens may have been lost.\MessageBreak Press <return> and I shall try to continue}{Index string probably has extra/missing '^' or '_'.}",
    ).unlist());
  }
  let staggered = !starred;
  if base.is_none() {
    let mut call = vec![T_CS!("\\lx@ms@postscripts"), T_BEGIN!()];
    call.extend(written);
    call.extend([T_END!(), T_BEGIN!()]);
    call.extend(index_columns(&post_items, staggered));
    call.push(T_END!());
    out.extend(call);
    return Ok(Tokens::new(out));
  }
  out.extend(
    multiscripts_call(
      written,
      index_columns(&pre_items, staggered),
      base.map(|b| b.unlist()).unwrap_or_default(),
      index_columns(&post_items, staggered),
    )
    .unlist(),
  );
  Ok(Tokens::new(out))
}
