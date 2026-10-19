//! xstring.sty / xstring.tex (v1.86, 2023/08/22, C. Tellechea) — string tests and operations.
//!
//! No Perl binding exists; native binding, OXIDIZED_DESIGN #588(q). Each xstring command reads an
//! optional `*`, an optional leading `[n]`, its arguments and an optional trailing `[\name]`
//! (`\xs_newmacro`, xstring.tex:235-298); it expands its string and number arguments by the mode set by
//! `\fullexpandarg` (`\edef`, the default), `\expandarg` (the first token once) or `\noexpandarg`
//! (:113-130), `\detokenize`s them under `*` (:187-191), works on *units* — a token, a braced group
//! kept with its braces, or a space (:134-180) — compared as token lists, and either leaves its
//! result in the input or `\def`s it to `\name` (:108); a test leaves its true or false branch. Here
//! each command is a primitive (none is expandable in TeX, they `\def` and `\futurelet`) doing the
//! same reading and computing the result natively, with the modes in the state. `\exploregroups`
//! (searches descending into groups, a pattern never crossing a group boundary, :411-442) is followed by
//! every command that searches, counts, substitutes, removes braces or cuts;
//! the cutting commands under `\exploregroups` cut after a leaf unit and return the parts of the group it lies in
//! (`\xs_read_reserved_*`, :335-370); `\StrBetween` and `\StrMid` never explore (:731, :881).
use crate::prelude::*;

type Unit = Vec<Token>;

fn mode(key: &str, default: &str) -> String {
  lookup_value(&format!("xstring@{key}"))
    .map(|v| v.to_string())
    .unwrap_or_else(|| default.to_string())
}
fn set_mode(key: &str, value: &str) {
  assign_value(&format!("xstring@{key}"), Stored::String(pin(value)), None);
}
fn exploring() -> bool { mode("explore", "no") == "yes" }

/// The top-level units of a token list.
fn units(toks: &[Token]) -> Vec<Unit> {
  let mut out = Vec::new();
  let mut i = 0;
  while i < toks.len() {
    if toks[i].get_catcode() == Catcode::BEGIN {
      let mut depth = 0usize;
      let start = i;
      while i < toks.len() {
        match toks[i].get_catcode() {
          Catcode::BEGIN => depth += 1,
          Catcode::END => {
            depth = depth.saturating_sub(1);
            if depth == 0 {
              break;
            }
          },
          _ => {},
        }
        i += 1;
      }
      out.push(toks[start..(i + 1).min(toks.len())].to_vec());
      i += 1;
    } else {
      out.push(vec![toks[i]]);
      i += 1;
    }
  }
  out
}

fn is_group(u: &Unit) -> bool { u.len() >= 2 && u[0].get_catcode() == Catcode::BEGIN }
fn group_inner(u: &Unit) -> Vec<Token> { u[1..u.len() - 1].to_vec() }
fn join(units: &[Unit]) -> Vec<Token> { units.iter().flatten().cloned().collect() }

/// The tokens with their first token expanded once (the `\expandafter` of xstring's `\expandarg`,
/// xstring.tex:116-118), the rest untouched.
fn expand_first(toks: Tokens) -> Result<Tokens> {
  let mut v = toks.unlist();
  if v.is_empty() {
    return Ok(Tokens::new(v));
  }
  let first = v.remove(0);
  // In a mouth of its own: the expansion reads its arguments from the argument only.
  reading_from_mouth(Mouth::default(), move || -> Result<Tokens> {
    unread(Tokens::new(v));
    if !expand_once_partial(first)? {
      unread_one(first);
    }
    let mut out = Vec::new();
    while let Some(t) = read_token()? {
      out.push(t);
    }
    Ok(Tokens::new(out))
  })
}

/// Expand an argument by the current mode (xstring.tex:113-130, :279-289).
fn expand_arg(toks: Tokens) -> Result<Tokens> {
  match mode("expand", "full").as_str() {
    "full" => do_expand(do_expand(toks)?),
    "first" => expand_first(toks),
    _ => Ok(toks),
  }
}

fn detokenize(toks: Tokens) -> Result<Tokens> {
  let mut t = vec![T_CS!("\\detokenize"), T_BEGIN!()];
  t.extend(toks.unlist());
  t.push(T_END!());
  do_expand(Tokens::new(t))
}

/// A string argument: read, expanded, detokenized under `*`.
fn string_arg(star: bool) -> Result<Vec<Token>> {
  let t = read_arg(ExpansionLevel::Off)?;
  Ok(if star {
    // `\xs_expand_and_detokenize` (:187-191): one expansion by the mode, then `\detokenize`.
    let once = match mode("expand", "full").as_str() {
      "full" => do_expand(t)?,
      "first" => expand_first(t)?,
      _ => t,
    };
    detokenize(once)?.unlist()
  } else {
    expand_arg(t)?.unlist()
  })
}

/// A number as TeX reads it after `\\ifnum` (registers, `\\value{…}` and `\\numexpr` included).
fn int_value(toks: Tokens) -> Result<i64> {
  if toks
    .clone()
    .unlist()
    .iter()
    .all(|t| t.get_catcode() == Catcode::SPACE)
  {
    return Ok(0);
  }
  // Read in a mouth of its own, so the number's scan ends with the argument (`\\numexpr`
  // included, which would swallow a `\\relax` marker) and nothing after it is touched.
  reading_from_mouth(Mouth::default(), move || -> Result<i64> {
    unread(toks);
    let n = read_number()?.value_of();
    // What follows is expanded (closing a conditional the number came from); spaces and `\relax`
    // end a number as they end `\ifnum`'s, anything else is not part of it.
    let mut rest = Vec::new();
    while let Some(t) = read_x_token(Some(false), false, None)? {
      if t.get_catcode() != Catcode::SPACE && t != T_CS!("\\relax") {
        rest.push(t);
      }
    }
    if !rest.is_empty() {
      Error!(
        "unexpected",
        "number",
        format!(
          "xstring: a number was expected, but '{}' follows {n} in the argument",
          Tokens::new(rest).untex()
        )
      );
    }
    Ok(n)
  })
}

/// `\number\numexpr<pre><arg><post>`, as `\StrMid` (`#2-1`, xstring.tex:886) and `\StrRight`/
/// `\StrGobbleRight` (`<length>-#2`, :912, :920) compute their cut.
fn numexpr_value(pre: &str, arg: Tokens, post: &str) -> Result<i64> {
  let mut t = vec![T_CS!("\\numexpr")];
  t.extend(tk(pre));
  t.extend(arg.unlist());
  t.extend(tk(post));
  int_value(Tokens::new(t))
}

fn number_arg() -> Result<i64> { int_value(expand_arg(read_arg(ExpansionLevel::Off)?)?) }

fn number_arg_tokens() -> Result<Tokens> { expand_arg(read_arg(ExpansionLevel::Off)?) }

/// The optional leading `[n]` or `[n1,n2]` (expanded by the mode).
fn leading_numbers(defaults: &[i64]) -> Result<Vec<i64>> {
  match read_optional(None)? {
    Some(t) => {
      let t = expand_arg(t)?.unlist();
      let mut v = Vec::new();
      for part in t.split(|t| t.get_catcode() == Catcode::OTHER && t.with_str(|s| s == ",")) {
        v.push(int_value(Tokens::new(part.to_vec()))?);
      }
      while v.len() < defaults.len() {
        v.push(defaults[v.len()]);
      }
      Ok(v)
    },
    None => Ok(defaults.to_vec()),
  }
}

fn star() -> Result<bool> {
  skip_spaces()?;
  if if_next(T_OTHER!("*"))? {
    read_token()?;
    Ok(true)
  } else {
    Ok(false)
  }
}

/// The result: into `\name` when a trailing `[\name]` follows, else into the input.
fn deliver(result: Vec<Token>) -> Result<()> {
  let mut out = Vec::new();
  if let Some(name) = read_optional(None)? {
    out.push(T_CS!("\\def"));
    out.extend(name.unlist());
    out.push(T_BEGIN!());
    out.extend(result);
    out.push(T_END!());
  } else {
    out = result;
  }
  unread(Tokens::new(out));
  Ok(())
}

/// A test's branch: read `{T}{F}` and leave the one chosen.
fn branch(truth: bool) -> Result<()> {
  let t = read_arg(ExpansionLevel::Off)?;
  let f = read_arg(ExpansionLevel::Off)?;
  unread(if truth { t } else { f });
  Ok(())
}

/// `\groupID`, global (xstring.tex:785-800): `0` at the top level, else the group's path, each group
/// counted among the groups of its level.
fn set_group_path(id: &str) -> Result<()> { raw_tex(&format!("\\gdef\\groupID{{{id}}}")) }

/// Under `\\exploregroups`, `\\groupID` after a scan of the first `n` leaf units: every group entered,
/// in order, by its ordinal among its level's groups (never popped, as `\\xs_calc_grouIP` piles it up).
fn entered_upto(s: &[Unit], n: usize, seen: &mut usize, entered: &mut Vec<usize>) -> bool {
  let mut ordinal = 0;
  for u in s {
    if *seen >= n {
      return true;
    }
    if is_group(u) {
      ordinal += 1;
      entered.push(ordinal);
      if entered_upto(&units(&group_inner(u)), n, seen, entered) {
        return true;
      }
    } else {
      *seen += 1;
    }
  }
  *seen >= n
}

fn cut_group_id(s: &[Unit], n: usize) -> String {
  let mut entered = Vec::new();
  entered_upto(s, n, &mut 0, &mut entered);
  piled_id(&entered)
}

/// Under `\\exploregroups`, the string's units with all braces removed (`\\IfSubStrBefore`,
/// xstring.tex:490-495; the patterns keep theirs).
fn flat_if_exploring(s: Vec<Unit>) -> Vec<Unit> {
  if exploring() {
    let mut l = Vec::new();
    leaves(&s, &mut l);
    l
  } else {
    s
  }
}

/// The group path and index of the `n`-th leaf unit (1-based) under `\exploregroups`.
fn locate_leaf(s: &[Unit], n: usize, path: &mut Vec<usize>) -> Option<usize> {
  let mut seen = 0;
  for (i, u) in s.iter().enumerate() {
    if is_group(u) {
      let inner = units(&group_inner(u));
      let count = leaf_count(&inner);
      if seen + count >= n {
        path.push(i);
        return locate_leaf(&inner, n - seen, path);
      }
      seen += count;
    } else {
      seen += 1;
      if seen == n {
        return Some(i);
      }
    }
  }
  None
}

/// A cut after the `n`-th unit (`\xs_read_reserved_*`, xstring.tex:335-370): at the top level, or under
/// `\exploregroups` after the `n`-th leaf, both parts then taken from the group the leaf lies in.
fn split_units(s: &[Unit], n: i64) -> Result<(Vec<Token>, Vec<Token>)> {
  let total = if exploring() { leaf_count(s) } else { s.len() };
  // `\xs_StrSplit_ii` (:813-814) clears `\groupID` first.
  set_group_path("")?;
  if n <= 0 {
    return Ok((Vec::new(), join(s)));
  }
  set_group_path(&if exploring() {
    cut_group_id(s, (n as usize).min(total))
  } else {
    "0".to_string()
  })?;
  // Under `\exploregroups` a cut at the last leaf is still inside that leaf's group.
  if n as usize > total || (n as usize == total && !exploring()) {
    return Ok((join(s), Vec::new()));
  }
  if !exploring() {
    let n = n as usize;
    return Ok((join(&s[..n]), join(&s[n..])));
  }
  let mut path = Vec::new();
  let idx = locate_leaf(s, n as usize, &mut path).unwrap_or(0);
  let level = hit_level(s, &Hit {
    path,
    start: 0,
    id: String::new(),
  });
  Ok((join(&level[..=idx]), join(&level[idx + 1..])))
}

/// `\\StrSplit*` when the cut ends a group (`\\xs_continuetonext`, xstring.tex:822-838): the parts at the
/// nearest enclosing level with anything after the group, the group itself kept in the left part.
fn split_units_closing(s: &[Unit], n: i64) -> (Vec<Token>, Vec<Token>) {
  let mut path = Vec::new();
  let _ = locate_leaf(s, n as usize, &mut path);
  while let Some(g) = path.pop() {
    let level = hit_level(s, &Hit {
      path:  path.clone(),
      start: 0,
      id:    String::new(),
    });
    if g + 1 < level.len() {
      return (join(&level[..=g]), join(&level[g + 1..]));
    }
  }
  (join(s), Vec::new())
}

fn total_units(s: &[Unit]) -> i64 {
  if exploring() {
    leaf_count(s) as i64
  } else {
    s.len() as i64
  }
}

/// Occurrence starts (unit indices) of `pat` in `s`, non-overlapping, left to right.
fn occurrences(s: &[Unit], pat: &[Unit]) -> Vec<usize> {
  let mut out = Vec::new();
  if pat.is_empty() {
    return out;
  }
  let mut i = 0;
  while i + pat.len() <= s.len() {
    if s[i..i + pat.len()] == *pat {
      out.push(i);
      i += pat.len();
    } else {
      i += 1;
    }
  }
  out
}

/// An occurrence under `\exploregroups`: the path of groups it lies in, and its index there.
#[derive(Clone)]
struct Hit {
  path:  Vec<usize>,
  start: usize,
  /// `\groupID` at the hit: under `\exploregroups`, the ordinal (among its level's groups) of every
  /// group entered so far, never popped on leaving one (`\xs_calc_grouIP`'s `\gdef`, xstring.tex:785-800).
  id:    String,
}

fn explore_hits(
  s: &[Unit],
  pat: &[Unit],
  path: &mut Vec<usize>,
  entered: &mut Vec<usize>,
  out: &mut Vec<Hit>,
) {
  let starts = occurrences(s, pat);
  let mut next_start = starts.iter().peekable();
  let mut i = 0;
  let mut ordinal = 0;
  while i < s.len() {
    if next_start.peek() == Some(&&i) {
      out.push(Hit {
        path:  path.clone(),
        start: i,
        id:    piled_id(entered),
      });
      next_start.next();
      // A group inside the match is not entered, so it is not counted (`\xs_read_reserved_D`, :348-358).
      i += pat.len();
      continue;
    }
    if is_group(&s[i]) {
      ordinal += 1;
      entered.push(ordinal);
      path.push(i);
      explore_hits(&units(&group_inner(&s[i])), pat, path, entered, out);
      path.pop();
    }
    i += 1;
  }
}

fn piled_id(entered: &[usize]) -> String {
  if entered.is_empty() {
    "0".to_string()
  } else {
    entered
      .iter()
      .map(|n| n.to_string())
      .collect::<Vec<_>>()
      .join(",")
  }
}

fn hits(s: &[Unit], pat: &[Unit]) -> Vec<Hit> {
  if exploring() {
    let mut out = Vec::new();
    explore_hits(s, pat, &mut Vec::new(), &mut Vec::new(), &mut out);
    out
  } else {
    occurrences(s, pat)
      .into_iter()
      .map(|start| Hit {
        path: Vec::new(),
        start,
        id: "0".to_string(),
      })
      .collect()
  }
}

/// The units of the group a hit lies in.
fn hit_level(s: &[Unit], hit: &Hit) -> Vec<Unit> {
  let mut level = s.to_vec();
  for &g in &hit.path {
    level = units(&group_inner(&level[g]));
  }
  level
}

fn leaf_count(s: &[Unit]) -> usize {
  s.iter()
    .map(|u| {
      if is_group(u) {
        leaf_count(&units(&group_inner(u)))
      } else {
        1
      }
    })
    .sum()
}

fn leaves(s: &[Unit], out: &mut Vec<Unit>) {
  for u in s {
    if is_group(u) {
      leaves(&units(&group_inner(u)), out);
    } else {
      out.push(u.clone());
    }
  }
}

fn substitute(s: &[Unit], a: &[Unit], b: &[Token], limit: &mut i64) -> Vec<Token> {
  let mut out = Vec::new();
  let mut i = 0;
  while i < s.len() {
    if *limit != 0 && i + a.len() <= s.len() && !a.is_empty() && s[i..i + a.len()] == *a {
      out.extend_from_slice(b);
      i += a.len();
      if *limit > 0 {
        *limit -= 1;
      }
      continue;
    }
    if exploring() && is_group(&s[i]) {
      out.push(T_BEGIN!());
      out.extend(substitute(&units(&group_inner(&s[i])), a, b, limit));
      out.push(T_END!());
    } else {
      out.extend(s[i].clone());
    }
    i += 1;
  }
  out
}

/// `\IfDecimal`'s parse (xstring.tex:530-629): the test's result and the values it leaves in
/// `\integerpart`, `\decimalpart`, `\afterdecimal`, with the separator it read.
struct Decimal {
  ok:    bool,
  int:   String,
  dec:   String,
  after: String,
  sep:   String,
  /// The parse failed before any character it could report (`\afterdecimal` is `\xs_errnumber`).
  err:   bool,
}

fn decimal(text: &str) -> Decimal {
  // Each character is an undelimited argument, so spaces between them are skipped
  // (`\xs_parsesigns#1#2`); the rest after a bad character is taken whole, spaces included
  // (`\xs_gobtoendnumber#1\relax`).
  let chars: Vec<char> = text.chars().collect();
  let mut d = Decimal {
    ok:    false,
    int:   "0".into(),
    dec:   "0".into(),
    after: String::new(),
    sep:   String::new(),
    err:   false,
  };
  let mut i = 0;
  let skip = |i: &mut usize| {
    while *i < chars.len() && chars[*i].is_whitespace() {
      *i += 1;
    }
  };
  let mut minus = false;
  skip(&mut i);
  while i < chars.len() && (chars[i] == '+' || chars[i] == '-') {
    minus ^= chars[i] == '-';
    i += 1;
    skip(&mut i);
  }
  if i == chars.len() {
    d.err = true;
    return d;
  }
  let sign = if minus { "-" } else { "" };
  let zeros = |s: &str| s.chars().all(|c| c == '0');
  let mut int = String::new();
  while i < chars.len() && chars[i].is_ascii_digit() {
    int.push(chars[i]);
    i += 1;
    skip(&mut i);
  }
  if i == chars.len() {
    d.int = format!("{sign}{int}");
    d.ok = true;
    return d;
  }
  if chars[i] != '.' && chars[i] != ',' {
    d.int = int;
    d.after = chars[i..].iter().collect::<String>();
    return d;
  }
  d.sep = chars[i].to_string();
  d.int = if zeros(&int) {
    "0".into()
  } else {
    format!("{sign}{}", int.trim_start_matches('0'))
  };
  i += 1;
  skip(&mut i);
  let mut dec = String::new();
  while i < chars.len() && chars[i].is_ascii_digit() {
    dec.push(chars[i]);
    i += 1;
    skip(&mut i);
  }
  if i == chars.len() {
    if dec.is_empty() {
      d.err = true;
    } else {
      d.dec = if zeros(&dec) { "0".into() } else { dec };
      d.ok = true;
    }
    return d;
  }
  d.dec = if zeros(&dec) { "0".into() } else { dec };
  d.after = chars[i..].iter().collect::<String>();
  d
}

fn tk(s: &str) -> Vec<Token> {
  mouth::tokenize_internal(TeXString::assembled(s.to_string())).unlist()
}

fn text_def(cs: &str, value: &str) -> Result<()> {
  // The detokenized characters themselves (catcode 12, spaces 10), as xstring `\edef`s them.
  let mut toks = vec![T_CS!("\\def"), T_CS!(&format!("\\{cs}")), T_BEGIN!()];
  toks.extend(value.chars().map(|c| {
    if c == ' ' {
      T_SPACE!()
    } else {
      Token::new(c.to_string(), Catcode::OTHER)
    }
  }));
  toks.push(T_END!());
  digest(Tokens::new(toks))?;
  Ok(())
}

/// `\integerpart`, `\decimalpart`, `\afterdecimal` as `\IfDecimal` leaves them; a failed parse's
/// `\afterdecimal` is `\xs_errnumber`, a letter `X` (xstring.tex:506).
fn decimal_side_macros(d: &Decimal) -> Result<()> {
  text_def("integerpart", &d.int)?;
  text_def("decimalpart", &d.dec)?;
  if d.err {
    raw_tex(r"\def\afterdecimal{X}")
  } else {
    text_def("afterdecimal", &d.after)
  }
}

/// `\IfEq`'s numeric equality (xstring.tex:658-670): both decimals, equal at `\ifdim` (sp) precision.
/// Each operand is tested with `\xs_IfDecimal`, so the side macros are left from the last one tested.
fn eq_decimal(a: &[Token], b: &[Token]) -> Result<bool> {
  let da = decimal(&Tokens::new(a.to_vec()).untex());
  decimal_side_macros(&da)?;
  if !da.ok {
    return Ok(false);
  }
  let db = decimal(&Tokens::new(b.to_vec()).untex());
  decimal_side_macros(&db)?;
  if !db.ok {
    return Ok(false);
  }
  Ok(dimen_sp(a)? == dimen_sp(b)?)
}

/// `<value>pt` read as `\ifdim` reads it, in sp (TeX's rounding and its errors).
fn dimen_sp(value: &[Token]) -> Result<i64> {
  let mut t = value.to_vec();
  t.extend(tk("pt"));
  reading_from_mouth(Mouth::default(), move || -> Result<i64> {
    unread(Tokens::new(t));
    let d = read_dimension()?.value_of();
    while read_token()?.is_some() {}
    Ok(d)
  })
}

#[rustfmt::skip]
LoadDefinitions!({
  // xstring.tex:1106-1110: the defaults.
  set_mode("expand", "full");
  set_mode("explore", "no");
  set_mode("compare", "normal");
  set_mode("ingroups", "yes");
  RawTeX!(r"\def\xstringversion{1.86}\def\xstringdate{2023/08/22}\def\xstringname{xstring}\gdef\groupID{}
\def\integerpart{}\def\decimalpart{}\def\afterdecimal{}\def\afterinteger{}");

  // xstring.tex:113-130, :743-746, :972-975: the modes.
  DefPrimitive!("\\fullexpandarg", { set_mode("expand", "full"); });
  DefPrimitive!("\\expandarg", { set_mode("expand", "first"); });
  DefPrimitive!("\\noexpandarg", { set_mode("expand", "none"); });
  Let!("\\normalexpandarg", "\\noexpandarg");
  DefPrimitive!("\\saveexpandmode", { let m = mode("expand", "full"); set_mode("saved@expand", &m); });
  DefPrimitive!("\\restoreexpandmode", { let m = mode("saved@expand", "full"); set_mode("expand", &m); });
  DefPrimitive!("\\exploregroups", { set_mode("explore", "yes"); });
  DefPrimitive!("\\noexploregroups", { set_mode("explore", "no"); });
  DefPrimitive!("\\saveexploremode", { let m = mode("explore", "no"); set_mode("saved@explore", &m); });
  DefPrimitive!("\\restoreexploremode", { let m = mode("saved@explore", "no"); set_mode("explore", &m); });
  DefPrimitive!("\\comparestrict", { set_mode("compare", "strict"); });
  DefPrimitive!("\\comparenormal", { set_mode("compare", "normal"); });
  DefPrimitive!("\\savecomparemode", { let m = mode("compare", "normal"); set_mode("saved@compare", &m); });
  DefPrimitive!("\\restorecomparemode", { let m = mode("saved@compare", "normal"); set_mode("compare", &m); });
  DefPrimitive!("\\expandingroups", { set_mode("ingroups", "yes"); });
  DefPrimitive!("\\noexpandingroups", { set_mode("ingroups", "no"); });
  RawTeX!(r"\saveexpandmode\saveexploremode\savecomparemode");

  // xstring.tex:444-484 `\IfSubStr*[n]{s}{p}{T}{F}`
  DefPrimitive!("\\IfSubStr", {
    let st = star()?;
    let n = leading_numbers(&[1])?[0];
    let s = units(&string_arg(st)?);
    let p = units(&string_arg(st)?);
    let hit = if n > 0 && !s.is_empty() && !p.is_empty() { hits(&s, &p).get(n as usize - 1).cloned() } else { None };
    let found = hit.is_some();
    set_group_path(&hit.map(|h| h.id).unwrap_or_default())?;
    branch(found)?;
  });
  // xstring.tex:486-503 `\IfSubStrBefore`, `\IfSubStrBehind`
  DefPrimitive!("\\IfSubStrBefore", {
    let st = star()?;
    let n = leading_numbers(&[1, 1])?;
    let s = flat_if_exploring(units(&string_arg(st)?));
    let a = units(&string_arg(st)?);
    let b = units(&string_arg(st)?);
    let oa = occurrences(&s, &a);
    let ob = occurrences(&s, &b);
    set_group_path("")?;
    let ok = n[0] > 0 && n[1] > 0 && match (oa.get(n[0] as usize - 1), ob.get(n[1] as usize - 1)) {
      (Some(&x), Some(&y)) => x + a.len() <= y,
      _ => false,
    };
    branch(ok)?;
  });
  DefPrimitive!("\\IfSubStrBehind", {
    let st = star()?;
    let n = leading_numbers(&[1, 1])?;
    let s = flat_if_exploring(units(&string_arg(st)?));
    let a = units(&string_arg(st)?);
    let b = units(&string_arg(st)?);
    let oa = occurrences(&s, &a);
    let ob = occurrences(&s, &b);
    set_group_path("")?;
    let ok = n[0] > 0 && n[1] > 0 && match (oa.get(n[0] as usize - 1), ob.get(n[1] as usize - 1)) {
      (Some(&x), Some(&y)) => y + b.len() <= x,
      _ => false,
    };
    branch(ok)?;
  });
  // xstring.tex:449-475 `\IfBeginWith`, `\IfEndWith`
  DefPrimitive!("\\IfBeginWith", {
    let st = star()?;
    let s = units(&string_arg(st)?);
    let p = units(&string_arg(st)?);
    branch(!s.is_empty() && !p.is_empty() && s.len() >= p.len() && s[..p.len()] == p[..])?;
  });
  DefPrimitive!("\\IfEndWith", {
    let st = star()?;
    let s = units(&string_arg(st)?);
    let p = units(&string_arg(st)?);
    let occ = occurrences(&s, &p);
    branch(!s.is_empty() && !p.is_empty() && occ.last().is_some_and(|&i| i + p.len() == s.len()))?;
  });
  // xstring.tex:658-685 `\IfStrEq`, `\IfEq`
  DefPrimitive!("\\IfStrEq", {
    let st = star()?;
    let a = string_arg(st)?;
    let b = string_arg(st)?;
    branch(a == b)?;
  });
  DefPrimitive!("\\IfEq", {
    let st = star()?;
    let a = string_arg(st)?;
    let b = string_arg(st)?;
    branch(a == b || eq_decimal(&a, &b)?)?;
  });
  // xstring.tex:530-635 `\IfDecimal`, `\IfInteger`
  DefPrimitive!("\\IfDecimal", {
    // xstring.tex:535: the argument `\expanded` whatever the mode, then detokenized.
    let x = detokenize(do_expand(expand_arg(read_arg(ExpansionLevel::Off)?)?)?)?;
    let d = decimal(&x.untex());
    decimal_side_macros(&d)?;
    branch(d.ok)?;
  });
  // xstring.tex:631-655
  DefPrimitive!("\\IfInteger", {
    let x = detokenize(do_expand(expand_arg(read_arg(ExpansionLevel::Off)?)?)?)?;
    let d = decimal(&x.untex());
    decimal_side_macros(&d)?;
    let (integer, after) = if d.ok {
      if d.dec == "0" { (true, String::new()) } else { (false, format!("{}{}", d.sep, d.dec)) }
    } else if !d.sep.is_empty() && d.err {
      (false, d.sep.clone())
    } else {
      // The separator, then `\afterdecimal` (`\xs_errnumber`'s `X` on a failed parse, :650).
      (false, format!("{}{}", d.sep, if d.err { "X" } else { d.after.as_str() }))
    };
    text_def("afterinteger", &after)?;
    branch(integer)?;
  });
  // xstring.tex:690-715 `\IfStrEqCase`, `\IfEqCase`
  for (cs, numeric) in [("\\IfStrEqCase", false), ("\\IfEqCase", true)] {
    def_primitive(T_CS!(cs), None, Some(PrimitiveBody::Closure(Rc::new(move |_args: Vec<ArgWrap>| {
      let st = star()?;
      let s = string_arg(st)?;
      let cases = read_arg(ExpansionLevel::Off)?.unlist();
      let items = units(&cases);
      let items: Vec<Unit> = items.into_iter().filter(|u| u.iter().any(|t| t.get_catcode() != Catcode::SPACE)).collect();
      let mut chosen: Option<Vec<Token>> = None;
      for pair in items.chunks(2) {
        if pair.len() < 2 { break; }
        let c = if is_group(&pair[0]) { group_inner(&pair[0]) } else { pair[0].clone() };
        let c = expand_arg(Tokens::new(c))?;
        let c = if st { detokenize(c)?.unlist() } else { c.unlist() };
        if c == s || (numeric && eq_decimal(&s, &c)?) {
          chosen = Some(if is_group(&pair[1]) { group_inner(&pair[1]) } else { pair[1].clone() });
          break;
        }
      }
      let other = read_optional(None)?;
      unread(Tokens::new(chosen.unwrap_or_else(|| other.map(|o| o.unlist()).unwrap_or_default())));
      Ok(Vec::new())
    }))), PrimitiveOptions::default())?;
  }

  // xstring.tex:718-729 `\StrBefore`, `\StrBehind`
  for (cs, behind) in [("\\StrBefore", false), ("\\StrBehind", true)] {
    def_primitive(T_CS!(cs), None, Some(PrimitiveBody::Closure(Rc::new(move |_args: Vec<ArgWrap>| {
      let st = star()?;
      let n = leading_numbers(&[1])?[0];
      let s = units(&string_arg(st)?);
      let p = units(&string_arg(st)?);
      let found = if n < 1 || s.is_empty() || p.is_empty() { None } else { hits(&s, &p).get(n as usize - 1).cloned() };
      set_group_path(&found.as_ref().map(|h| h.id.clone()).unwrap_or_default())?;
      let result = match found {
        Some(h) => {
          let level = hit_level(&s, &h);
          if behind { join(&level[h.start + p.len()..]) } else { join(&level[..h.start]) }
        },
        None => Vec::new(),
      };
      deliver(result)?;
      Ok(Vec::new())
    }))), PrimitiveOptions::default())?;
  }
  // xstring.tex:731-748 `\StrBetween*[n1,n2]{s}{A}{B}` (never explores)
  DefPrimitive!("\\StrBetween", {
    let st = star()?;
    let n = leading_numbers(&[1, 1])?;
    let s = units(&string_arg(st)?);
    let a = units(&string_arg(st)?);
    let b = units(&string_arg(st)?);
    let mut result = Vec::new();
    if n[0] > 0 && n[1] > 0 && !a.is_empty() && !b.is_empty()
      && let Some(&bi) = occurrences(&s, &b).get(n[1] as usize - 1) {
        let prefix = &s[..bi];
        if let Some(&ai) = occurrences(prefix, &a).get(n[0] as usize - 1) {
          result = join(&prefix[ai + a.len()..]);
        }
      }
    raw_tex(r"\gdef\groupID{}")?;
    deliver(result)?;
  });
  // xstring.tex:750-775 `\StrSubstitute[n]{s}{A}{B}`, `\StrDel[n]{s}{A}`
  DefPrimitive!("\\StrSubstitute", {
    let n = leading_numbers(&[0])?[0];
    let s = units(&string_arg(false)?);
    let a = units(&string_arg(false)?);
    let b = string_arg(false)?;
    let mut limit = if n < 1 { -1 } else { n };
    deliver(if a.is_empty() { join(&s) } else { substitute(&s, &a, &b, &mut limit) })?;
  });
  DefPrimitive!("\\StrDel", {
    let n = leading_numbers(&[0])?[0];
    let s = units(&string_arg(false)?);
    let a = units(&string_arg(false)?);
    let mut limit = if n < 1 { -1 } else { n };
    deliver(if a.is_empty() { join(&s) } else { substitute(&s, &a, &[], &mut limit) })?;
  });
  // xstring.tex:776-800 `\StrLen`
  DefPrimitive!("\\StrLen", {
    let s = units(&string_arg(false)?);
    let n = if exploring() { leaf_count(&s) } else { s.len() };
    deliver(tk(&n.to_string()))?;
  });
  // xstring.tex:881-924 `\StrMid`, `\StrLeft`, `\StrRight`, `\StrGobbleLeft`, `\StrGobbleRight`, `\StrChar`
  // A cut at `n` (`\StrLeft`/`\StrGobbleLeft`) or at the length less `n` (`\StrRight`/`\StrGobbleRight`,
  // :909-922), each keeping one part.
  DefPrimitive!("\\StrLeft", {
    let s = units(&string_arg(false)?);
    let n = number_arg()?;
    deliver(split_units(&s, n)?.0)?;
  });
  DefPrimitive!("\\StrGobbleLeft", {
    let s = units(&string_arg(false)?);
    let n = number_arg()?;
    deliver(split_units(&s, n)?.1)?;
  });
  DefPrimitive!("\\StrRight", {
    let s = units(&string_arg(false)?);
    let n = numexpr_value(&format!("{}-", total_units(&s)), number_arg_tokens()?, "")?;
    deliver(split_units(&s, n)?.1)?;
  });
  DefPrimitive!("\\StrGobbleRight", {
    let s = units(&string_arg(false)?);
    let n = numexpr_value(&format!("{}-", total_units(&s)), number_arg_tokens()?, "")?;
    deliver(split_units(&s, n)?.0)?;
  });
  // `\StrMid` (:881-892): the units from `#2` to `#3`, never exploring groups; `\groupID` cleared.
  DefPrimitive!("\\StrMid", {
    let s = units(&string_arg(false)?);
    let a = numexpr_value("", number_arg_tokens()?, "-1")?.max(0) + 1;
    let b = number_arg()?.min(s.len() as i64);
    raw_tex(r"\gdef\groupID{}")?;
    deliver(if a > b { Vec::new() } else { join(&s[(a - 1) as usize..b as usize]) })?;
  });
  DefPrimitive!("\\StrChar", {
    let s = units(&string_arg(false)?);
    let n = number_arg()?;
    let pool = if exploring() { let mut l = Vec::new(); leaves(&s, &mut l); l } else { s };
    deliver(if n < 1 || n as usize > pool.len() { Vec::new() } else { pool[n as usize - 1].clone() })?;
  });
  // xstring.tex:946-1017 `\StrCount`, `\StrPosition`, `\StrCompare`
  DefPrimitive!("\\StrCount", {
    let s = units(&string_arg(false)?);
    let p = units(&string_arg(false)?);
    let n = if s.is_empty() || p.is_empty() { 0 } else { hits(&s, &p).len() };
    deliver(tk(&n.to_string()))?;
  });
  DefPrimitive!("\\StrPosition", {
    let n = leading_numbers(&[1])?[0];
    let s = units(&string_arg(false)?);
    let p = units(&string_arg(false)?);
    let hit = if n < 1 || p.is_empty() { None } else { hits(&s, &p).get(n as usize - 1).cloned() };
    set_group_path(&hit.as_ref().map(|h| h.id.clone()).unwrap_or_default())?;
    // Under `\\exploregroups` the position counts leaf units, within the hit's group.
    let pos = hit.map_or(0, |h| {
      if exploring() { leaf_count(&hit_level(&s, &h)[..h.start]) + 1 } else { h.start + 1 }
    });
    deliver(tk(&pos.to_string()))?;
  });
  DefPrimitive!("\\StrCompare", {
    let st = star()?;
    let a = units(&string_arg(st)?);
    let b = units(&string_arg(st)?);
    let strict = mode("compare", "normal") == "strict";
    let n = if a == b { 0 } else {
      match a.iter().zip(b.iter()).position(|(x, y)| x != y) {
        Some(p) => p + 1,
        None => if strict { a.len().min(b.len()) + 1 } else { 0 },
      }
    };
    deliver(tk(&n.to_string()))?;
  });
  // xstring.tex:803-873 `\StrSplit*{s}{n}\csA\csB`, `\StrCut*[n]{s}{p}\csA\csB`
  DefPrimitive!("\\StrSplit", {
    let st = star()?;
    let s = units(&string_arg(false)?);
    let n = number_arg()?;
    let a = read_arg(ExpansionLevel::Off)?.unlist();
    let b = read_arg(ExpansionLevel::Off)?.unlist();
    let (left, right) = split_units(&s, n)?;
    // `\StrSplit*` (:803-840): a cut ending a group continues in the groups that enclose it.
    let (left, right) = if st && exploring() && right.is_empty() && n > 0 && (n as usize) <= leaf_count(&s) {
      split_units_closing(&s, n)
    } else {
      (left, right)
    };
    let mut out = vec![T_CS!("\\def")];
    out.extend(a);
    out.extend(group(left));
    out.push(T_CS!("\\def"));
    out.extend(b);
    out.extend(group(right));
    unread(Tokens::new(out));
  });
  DefPrimitive!("\\StrCut", {
    let st = star()?;
    let n = leading_numbers(&[1])?[0].max(1);
    let s = units(&string_arg(st)?);
    let p = units(&string_arg(st)?);
    let a = read_arg(ExpansionLevel::Off)?.unlist();
    let b = read_arg(ExpansionLevel::Off)?.unlist();
    let (left, right) = if p.is_empty() { (Vec::new(), Vec::new()) } else {
      match hits(&s, &p).get(n as usize - 1) {
        Some(h) => {
          let level = hit_level(&s, h);
          (join(&level[..h.start]), join(&level[h.start + p.len()..]))
        },
        None => (join(&s), Vec::new()),
      }
    };
    let mut out = vec![T_CS!("\\def")];
    out.extend(a);
    out.extend(group(left));
    out.push(T_CS!("\\def"));
    out.extend(b);
    out.extend(group(right));
    unread(Tokens::new(out));
  });
  // xstring.tex:372-409 `\StrRemoveBraces`
  DefPrimitive!("\\StrRemoveBraces", {
    let s = units(&string_arg(false)?);
    let result = if exploring() { let mut l = Vec::new(); leaves(&s, &mut l); join(&l) }
      else { s.iter().flat_map(|u| if is_group(u) { group_inner(u) } else { u.clone() }).collect() };
    deliver(result)?;
  });
  // xstring.tex:1019-1060 `\StrFindGroup{s}{id}`
  DefPrimitive!("\\StrFindGroup", {
    let s = string_arg(false)?;
    let id = expand_arg(read_arg(ExpansionLevel::Off)?)?.untex().to_string();
    let mut current = group(s);
    let mut ok = true;
    for part in id.split(',') {
      let k: i64 = part.trim().parse().unwrap_or(-1);
      if k == 0 { continue; }
      let inner = units(&group_inner(&current));
      let groups: Vec<&Unit> = inner.iter().filter(|u| is_group(u)).collect();
      match groups.get((k - 1).max(0) as usize).filter(|_| k > 0) {
        Some(g) => current = (*g).clone(),
        None => { ok = false; break; },
      }
    }
    deliver(if ok { current } else { Vec::new() })?;
  });
  // xstring.tex:1066-1104 `\StrExpand[n]{s}{\cs}`, `\scancs[n]\cs{s}`
  DefPrimitive!("\\StrExpand", {
    let n = leading_numbers(&[1])?[0];
    let s = read_arg(ExpansionLevel::Off)?.unlist();
    let cs = read_arg(ExpansionLevel::Off)?.unlist();
    let result = expand_passes(s, n)?;
    let mut out = vec![T_CS!("\\def")];
    out.extend(cs);
    out.extend(group(result));
    unread(Tokens::new(out));
  });
  DefPrimitive!("\\scancs", {
    let n = leading_numbers(&[1])?[0];
    let cs = read_arg(ExpansionLevel::Off)?.unlist();
    let s = read_arg(ExpansionLevel::Off)?.unlist();
    let result = detokenize(Tokens::new(expand_passes(s, n)?))?.unlist();
    let mut out = vec![T_CS!("\\def")];
    out.extend(cs);
    out.extend(group(result));
    unread(Tokens::new(out));
  });
  // xstring.tex:100-106 `\tokenize{\cs}{text}`: retokenized under the current catcodes.
  DefPrimitive!("\\tokenize", {
    let cs = read_arg(ExpansionLevel::Off)?.unlist();
    let text = read_arg(ExpansionLevel::Off)?;
    // `\xs_def` expands only under `\fullexpandarg`; the text is then re-read under the current catcodes
    // with no line end (`\scantokens`, `\endlinechar=-1`) and `\def`ed, not expanded.
    let text = if mode("expand", "full") == "full" { do_expand(text)? } else { text };
    let source = detokenize(text)?.untex().to_string();
    let rescanned = reading_from_mouth(Mouth::new(&source, None)?, || -> Result<Vec<Token>> {
      let mut out = Vec::new();
      while let Some(t) = read_token()? {
        out.push(t);
      }
      Ok(out)
    })?;
    let mut rescanned = rescanned;
    while rescanned.last().is_some_and(|t| t.get_catcode() == Catcode::SPACE) {
      rescanned.pop();
    }
    let mut out = vec![T_CS!("\\def")];
    out.extend(cs);
    out.extend(group(rescanned));
    unread(Tokens::new(out));
  });
  // xstring.tex:79-99 `\setverbdelim{c}`, `\verbtocs{\cs}|text|`
  DefPrimitive!("\\setverbdelim{}", sub[(c)] {
    let c = c.unlist();
    if c.len() != 1 {
      RawTeX!(r"\errmessage{Package xstring Error: verb delimiter must be one character}");
    }
    let s = c.first().map(|t| t.with_str(|s| s.to_string())).unwrap_or_else(|| "|".into());
    set_mode("verbdelim", &s);
  });
  RawTeX!(r"\def\verbtocs#1{\begingroup\let\do\@makeother\dospecials\catcode32=13 \lx@xs@verbtocs{#1}}");
  DefPrimitive!("\\lx@xs@verbtocs{}", sub[(cs)] {
    let delim = mode("verbdelim", "|");
    // Skip to the first delimiter, then read to the next.
    loop {
      match read_token()? {
        Some(t) if t.with_str(|s| s == delim) => break,
        Some(_) => {},
        None => return Ok(Vec::new()),
      }
    }
    let mut text = Vec::new();
    while let Some(t) = read_token()? {
      if t.with_str(|s| s == delim) { break; }
      text.push(t);
    }
    let mut out = vec![T_CS!("\\endgroup"), T_CS!("\\def")];
    out.extend(cs.unlist());
    out.extend(group(text));
    unread(Tokens::new(out));
  });
});

fn group(v: Vec<Token>) -> Vec<Token> {
  let mut out = vec![T_BEGIN!()];
  out.extend(v);
  out.push(T_END!());
  out
}

/// `\StrExpand`'s `n` passes, each expanding every unit once (groups' contents under
/// `\expandingroups`, xstring.tex:1066-1096).
fn expand_passes(s: Vec<Token>, n: i64) -> Result<Vec<Token>> {
  let mut current = s;
  for _ in 0..n.max(0) {
    let mut next = Vec::new();
    for u in units(&current) {
      if is_group(&u) {
        if mode("ingroups", "yes") == "yes" {
          next.push(T_BEGIN!());
          next.extend(expand_passes(group_inner(&u), 1)?);
          next.push(T_END!());
        } else {
          next.extend(u);
        }
      } else {
        next.extend(expand_first(Tokens::new(u))?.unlist());
      }
    }
    current = next;
  }
  Ok(current)
}
