//! biblatex's own layout of a bibliography entry (OXIDIZED_DESIGN_DIVERGENCES #601).
//!
//! `MakeBibliography`'s `%FMT_SPEC` rows ([`crate::make_bibliography`]) lay an entry out the way LaTeXML's
//! `.bst`-flavoured formatter does: the author with the year, "External Links:", "Cited by:". A document that loads
//! biblatex with the `alphabetic` or `alphabetic-verb` style, whose drivers are `standard.bbx`'s, prints
//! something else: `Author, Author, and Author. “Title”. In: Journal 130.5 (2008), pp. 1337–1398. ISSN: …. DOI: ….`
//! This module is that layout, a port of the part of biblatex the standard drivers run:
//!
//! - the punctuation tracker ([`Printer`]): biblatex.sty:4130-4260 `\blx@begunit`/`\blx@endunit`, `\newunit`,
//!   `\setunit`, `\printunit`, `\finentry`, and the space-factor rules of :1738-2060 (`\blx@addpunct`,
//!   `\DeclarePunctuationPairs`, `\isdot`) that decide whether a period, comma or colon is printed after a text that
//!   already ends in one;
//! - the bibliography drivers of `standard.bbx` (:28-733) and the macros they use (`biblatex.def:2728-3300`);
//! - the field formats of `biblatex.def:450-600`, the name formats (:877-1180) and `english.lbx` strings.
//!
//! It reads the entry's `ltx:bib-*` markup, not biber's data model, so what that markup does not keep (the
//! `titleaddon` / `subtitle` distinction, `related` entries, the `maintitle` of a multi-volume work) is approximated; the
//! binding records the options the drivers read (`maxnames`, `isbn`, `doi`, … ) in the bibliography's `bibstyle`
//! ([`Layout::from_bibstyle`]).

use libxml::tree::Node;

use crate::document::{NodeData, PostDocument};

/// The options of the standard bibliography drivers, as the biblatex binding records them in `bibstyle`:
/// `biblatex;layout=standard;maxnames=5;minnames=1;isbn=false;…`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Layout {
  /// `maxbibnames` / `minbibnames` (biblatex.sty:16425 defaults `maxnames=3`, `minnames=1`; `maxnames` sets both).
  pub max_names:      usize,
  pub min_names:      usize,
  /// `isbn`, `url`, `doi`, `eprint` (standard.bbx:4-21): the toggles `\iftoggle{bbx:…}` the drivers test.
  pub isbn:           bool,
  pub url:            bool,
  pub doi:            bool,
  pub eprint:         bool,
  /// `giveninits` (biblatex.def:877 `name:given-family` with `\namepartgiveni`).
  pub given_initials: bool,
}

impl Layout {
  /// The layout a `bibstyle` attribute asks for, if it asks for biblatex's standard drivers.
  pub fn from_bibstyle(bibstyle: &str) -> Option<Layout> {
    let mut parts = bibstyle.split(';');
    if !parts.next()?.starts_with("biblatex") {
      return None;
    }
    let mut layout = Layout {
      max_names:      3,
      min_names:      1,
      isbn:           true,
      url:            true,
      doi:            true,
      eprint:         true,
      given_initials: bibstyle.starts_with("biblatex-giveninits"),
    };
    let mut standard = false;
    for part in parts {
      let (key, value) = part.split_once('=').unwrap_or((part, "true"));
      let number = value.parse::<usize>().ok().filter(|n| *n >= 1);
      match key {
        "layout" => standard = value == "standard",
        "maxnames" => layout.max_names = number.unwrap_or(layout.max_names),
        "minnames" => layout.min_names = number.unwrap_or(layout.min_names),
        "isbn" => layout.isbn = value != "false",
        "url" => layout.url = value != "false",
        "doi" => layout.doi = value != "false",
        "eprint" => layout.eprint = value != "false",
        _ => {},
      }
    }
    // `\blx@maxmin@num` (biblatex.sty:15365-15377): a maximum below the minimum is raised to it (with biblatex's
    // "must be greater than or equal" option error).
    layout.max_names = layout.max_names.max(layout.min_names);
    standard.then_some(layout)
  }
}

// ======================================================================
// The punctuation tracker

/// The space factor class of the last printed character (biblatex.sty:1719-1737). `Plain` is every character whose
/// space factor lies outside the tracked 1002-1009 range.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Sf {
  Plain,
  /// `\isdot` / `\adddot`: an abbreviation's period.
  Dot,
  Comma,
  Semicolon,
  Colon,
  /// A sentence-ending period.
  Period,
  Exclam,
  Question,
}

/// A punctuation command of a unit's delimiter (`\addperiod\space`, …).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum P {
  Period,
  Comma,
  Semicolon,
  Colon,
  /// `\adddot`.
  Dot,
  Space,
}

/// `\newunitpunct`, biblatex.def:173: `\addperiod\space`.
const NEWUNITPUNCT: &[P] = &[P::Period, P::Space];
const COMMA_SPACE: &[P] = &[P::Comma, P::Space];
const COLON_SPACE: &[P] = &[P::Colon, P::Space];
const SPACE: &[P] = &[P::Space];
const DOT: &[P] = &[P::Dot];

/// The space factor classes after which a punctuation mark is still printed
/// (biblatex.sty:2026-2035 `\blx@resetpunct`: `\DeclarePunctuationPairs{comma}{*!?}` …, `*` standing for `Dot`). A
/// period is never printed after a mark; a text without one takes any.
fn pair_allows(punct: P, after: Sf) -> bool {
  match after {
    Sf::Plain => true,
    Sf::Dot | Sf::Exclam | Sf::Question => matches!(punct, P::Comma | P::Semicolon | P::Colon),
    Sf::Comma | Sf::Semicolon | Sf::Colon | Sf::Period => false,
  }
}

/// The space factor class a text leaves behind: that of its last character, the brackets and quotes (space factor
/// code 0, biblatex.sty:1749-1754) letting the one before show through.
fn sf_after(text: &str, before: Sf) -> Sf {
  let mut sf = before;
  for c in text.chars() {
    sf = match c {
      '.' => Sf::Period,
      ',' => Sf::Comma,
      ';' => Sf::Semicolon,
      ':' => Sf::Colon,
      '!' => Sf::Exclam,
      '?' => Sf::Question,
      ')' | ']' | '(' | '[' | '<' | '>' | '\'' | '`' | '’' | '‘' | '”' | '“' | '"' | '\u{a0}' => {
        sf
      },
      c if c.is_whitespace() => sf,
      _ => Sf::Plain,
    };
  }
  sf
}

/// The printer behind a driver: the toggles of biblatex.sty:4160-4260 and the entry built so far.
struct Printer {
  layout:                Layout,
  out:                   Vec<NodeData>,
  sf:                    Sf,
  /// Nothing printed yet: `\blx@sf@par`, where the autocapitalizer capitalizes.
  start:                 bool,
  /// blx@block / blx@unit / blx@insert / blx@lastins / blx@keepunit.
  block:                 bool,
  unit:                  bool,
  insert:                bool,
  lastins:               bool,
  keepunit:              bool,
  /// `\blx@unitpunct`, and whether running it ends with `\togglefalse{blx@keepunit}` (`\printunit`).
  unitpunct:             &'static [P],
  unitpunct_clears_keep: bool,
}

impl Printer {
  fn new(layout: Layout) -> Self {
    Printer {
      layout,
      out: Vec::new(),
      sf: Sf::Plain,
      start: true,
      block: false,
      unit: false,
      insert: false,
      lastins: false,
      keepunit: false,
      unitpunct: NEWUNITPUNCT,
      unitpunct_clears_keep: false,
    }
  }

  // ---- text

  /// Append text; its last character decides the space factor.
  fn text(&mut self, text: &str) {
    if text.is_empty() {
      return;
    }
    if !text.chars().all(char::is_whitespace) {
      self.start = false;
    }
    self.sf = sf_after(text, self.sf);
    self.out.push(NodeData::Text(text.to_string()));
  }

  /// Append marked-up content whose printed text ends in `tail` (`None`: nothing printed).
  fn nodes(&mut self, nodes: Vec<NodeData>, tail: Option<String>) {
    if nodes.is_empty() {
      return;
    }
    if let Some(tail) = tail {
      self.sf = if tail.trim().is_empty() {
        Sf::Plain
      } else {
        sf_after(&tail, Sf::Plain)
      };
      self.start = false;
    }
    self.out.extend(nodes);
  }

  /// `\unspace`: drop the white space the output ends with.
  fn unspace(&mut self) {
    while let Some(NodeData::Text(text)) = self.out.last_mut() {
      let trimmed = text
        .trim_end_matches(|c: char| c.is_whitespace() && c != '\u{a0}')
        .len();
      if trimmed == text.len() {
        break;
      }
      if trimmed == 0 {
        self.out.pop();
      } else {
        text.truncate(trimmed);
        break;
      }
    }
  }

  /// `\isdot` (biblatex.sty:2098): a period just typed belongs to an abbreviation.
  fn isdot(&mut self) {
    if self.sf == Sf::Period {
      self.sf = Sf::Dot;
    }
  }

  /// `\blx@addpunct` (biblatex.sty:2125) for one mark.
  fn add_punct(&mut self, punct: P) {
    self.unspace();
    match punct {
      P::Space => {
        if !self.out.is_empty() {
          self.out.push(NodeData::Text(" ".to_string()));
        }
      },
      P::Dot => self.add_dot(),
      mark => {
        if pair_allows(mark, self.sf) {
          let (char, sf) = match mark {
            P::Period => ('.', Sf::Period),
            P::Comma => (',', Sf::Comma),
            P::Semicolon => (';', Sf::Semicolon),
            _ => (':', Sf::Colon),
          };
          self.out.push(NodeData::Text(char.to_string()));
          self.sf = sf;
        }
        // biblatex.sty:2109-2116: `\addperiod` after a dot turns it into a sentence end.
        if mark == P::Period && self.sf == Sf::Dot {
          self.sf = Sf::Period;
        }
      },
    }
  }

  /// `\adddot` (biblatex.sty:2101): an abbreviation's period, printed unless a mark ends the text already
  /// (`\DeclarePunctuationPairs{dot}{}`); the space factor becomes a dot's only after a period.
  fn add_dot(&mut self) {
    self.unspace();
    if self.sf == Sf::Plain {
      self.out.push(NodeData::Text(".".to_string()));
      self.sf = Sf::Dot;
    }
    self.isdot();
  }

  fn add_space(&mut self) { self.add_punct(P::Space); }

  /// The autocapitalizer (`\blx@imc@ifcapital`): at the start of a sentence.
  fn capital(&self) -> bool {
    self.start || matches!(self.sf, Sf::Period | Sf::Exclam | Sf::Question)
  }

  // ---- units

  /// `\blx@begunit` (biblatex.sty:4172): the punctuation pending between the previous text and this.
  fn begin_unit(&mut self) {
    let mut tempa = true;
    if self.insert {
      if self.unit {
        for punct in self.unitpunct {
          self.add_punct(*punct);
        }
        if self.unitpunct_clears_keep {
          self.keepunit = false;
        }
        self.unit = false;
        tempa = false;
      }
      if self.block {
        // `\newblockpunct` is empty (biblatex.sty:15618, `block=none`).
        self.block = false;
        tempa = false;
      }
    }
    if !tempa {
      self.insert = false;
    }
  }

  /// `\blx@endunit`.
  fn end_unit(&mut self) {
    self.insert = true;
    self.lastins = true;
  }

  /// `\blx@nounit`: a field that printed nothing.
  fn no_unit(&mut self) { self.lastins = false; }

  /// A printed unit: `\printfield` and `\printtext`. `class` names the markup around what `body` prints.
  fn print(&mut self, class: &str, body: impl FnOnce(&mut Printer)) {
    self.begin_unit();
    let mark = self.out.len();
    body(self);
    let inner = self.out.split_off(mark);
    if class.is_empty() {
      self.out.extend(inner);
    } else {
      self.out.push(wrap(
        "ltx:text",
        &[("class", &format!("ltx_bib_{class}"))],
        inner,
      ));
    }
    self.end_unit();
  }

  /// `\newunit` (biblatex.sty:4214).
  fn new_unit(&mut self) {
    if !self.keepunit {
      self.unitpunct = NEWUNITPUNCT;
      self.unitpunct_clears_keep = false;
      self.unit = true;
    }
  }

  /// `\setunit` (biblatex.sty:4223).
  fn set_unit(&mut self, punct: &'static [P]) {
    if !self.keepunit {
      self.unitpunct = punct;
      self.unitpunct_clears_keep = false;
      self.unit = true;
    }
  }

  /// `\setunit*`: only after a unit that printed.
  fn set_unit_star(&mut self, punct: &'static [P]) {
    if self.lastins {
      self.set_unit(punct);
    }
  }

  /// `\printunit` (biblatex.sty:4234): the mark is printed whatever `\newunit` and `\setunit` say, until it is.
  fn print_unit(&mut self, punct: &'static [P]) {
    self.unitpunct = punct;
    self.unitpunct_clears_keep = true;
    self.keepunit = true;
    self.unit = true;
  }

  fn new_block(&mut self) { self.block = true; }

  /// `\newunit\newblock`.
  fn new_unit_block(&mut self) {
    self.new_unit();
    self.new_block();
  }

  /// `\finentry` (biblatex.sty:4247): the entry's last period.
  fn finish(mut self) -> Vec<NodeData> {
    if self.out.is_empty() {
      return Vec::new();
    }
    self.unspace();
    self.add_punct(P::Period);
    coalesce(self.out)
  }

  // ---- strings

  /// `\bibstring{…}` in its short form, capitalized at the start of a sentence (biblatex.sty:5040).
  fn bibstring(&mut self, short: &str, dot: bool) {
    let capital = self.capital();
    let mut text = short.to_string();
    if capital {
      let mut chars = text.chars();
      if let Some(first) = chars.next() {
        text = first.to_uppercase().chain(chars).collect();
      }
    }
    self.text(&text);
    if dot {
      self.add_dot();
    }
  }
}

fn wrap(tag: &str, attributes: &[(&str, &str)], children: Vec<NodeData>) -> NodeData {
  NodeData::Element {
    tag: tag.to_string(),
    attributes: (!attributes.is_empty()).then(|| {
      attributes
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
    }),
    children,
  }
}

/// Merge adjacent text nodes.
fn coalesce(nodes: Vec<NodeData>) -> Vec<NodeData> {
  let mut merged: Vec<NodeData> = Vec::with_capacity(nodes.len());
  for node in nodes {
    match (merged.last_mut(), node) {
      (Some(NodeData::Text(last)), NodeData::Text(next)) => last.push_str(&next),
      (_, NodeData::Element { tag, attributes, children }) => merged.push(NodeData::Element {
        tag,
        attributes,
        children: coalesce(children),
      }),
      (_, node) => merged.push(node),
    }
  }
  merged
}

fn emph(children: Vec<NodeData>) -> NodeData { wrap("ltx:emph", &[], children) }

/// `\mkbibacro` (biblatex.def:402): small capitals of the lower-cased acronym.
fn acronym(text: &str) -> NodeData {
  wrap("ltx:text", &[("font", "smallcaps")], vec![NodeData::Text(
    text.to_lowercase(),
  )])
}

fn link(href: &str, class: &str, children: Vec<NodeData>) -> NodeData {
  wrap("ltx:ref", &[("href", href), ("class", class)], children)
}

// ======================================================================
// The entry's fields

/// A name of an `ltx:bib-name`, or the "and others" of a name list.
struct Name {
  given:  Option<String>,
  family: String,
  suffix: Option<String>,
}

/// biblatex's drivers (standard.bbx:28-733), by the `ltx:bibentry`'s `type`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Driver {
  Article,
  Book,
  Booklet,
  Collection,
  Inbook,
  Incollection,
  Inproceedings,
  Manual,
  Misc,
  Online,
  Proceedings,
  Report,
  Thesis,
  Unpublished,
}

impl Driver {
  /// standard.bbx:740-752 `\DeclareBibliographyAlias`, the legacy BibTeX types the reader renames, and `*` → `misc`.
  fn of(entry_type: &str) -> Driver {
    match entry_type {
      "article" | "review" | "suppperiodical" => Driver::Article,
      "book" | "mvbook" => Driver::Book,
      "booklet" => Driver::Booklet,
      "collection" | "mvcollection" | "reference" | "mvreference" => Driver::Collection,
      "inbook" | "bookinbook" | "suppbook" => Driver::Inbook,
      "incollection" | "suppcollection" | "inreference" => Driver::Incollection,
      "inproceedings" | "conference" => Driver::Inproceedings,
      "manual" => Driver::Manual,
      "website" | "online" | "electronic" | "www" => Driver::Online,
      "proceedings" | "mvproceedings" => Driver::Proceedings,
      "report" | "techreport" => Driver::Report,
      "thesis" | "phdthesis" | "mastersthesis" => Driver::Thesis,
      "unpublished" => Driver::Unpublished,
      _ => Driver::Misc,
    }
  }
}

/// How a title is set (biblatex.def:579-585 `\DeclareFieldFormat[<types>]{title}`), by the entry's own type, not by
/// the driver it prints with: the types of the article, inbook, incollection, inproceedings, patent, thesis and
/// unpublished drivers are quoted, the `supp*` types plain, every other type (`review`, `bookinbook`, `report`, …)
/// emphasised.
#[derive(Clone, Copy, PartialEq, Eq)]
enum TitleStyle {
  Quote,
  Emph,
  Plain,
}

impl TitleStyle {
  fn of(entry_type: &str) -> TitleStyle {
    match entry_type {
      "article" | "inbook" | "incollection" | "inproceedings" | "conference" | "patent"
      | "thesis" | "phdthesis" | "mastersthesis" | "unpublished" => TitleStyle::Quote,
      "suppbook" | "suppcollection" | "suppperiodical" => TitleStyle::Plain,
      _ => TitleStyle::Emph,
    }
  }
}

/// The fields of one `ltx:bibentry`.
struct Fields {
  entry:       Node,
  /// The `ltx:bib-related[@role='host']` of an `incollection`, `inbook`, `inproceedings` or `article`.
  host:        Option<Node>,
  driver:      Driver,
  /// The entry's own type, which sets its title ([`TitleStyle`]).
  title_style: TitleStyle,
}

fn collapse_ws(text: &str) -> String { text.split_whitespace().collect::<Vec<_>>().join(" ") }

impl Fields {
  fn new(entry: &Node, entry_type: &str) -> Fields {
    // The type the author wrote (`review`, `suppbook`, …) sets the title ([`TitleStyle`]); the reader folds it into the
    // type biblatex's `\DeclareBibliographyAlias` gives (`article`, `inbook`), so it is read off the raw record the
    // reader keeps beside the fields, `@review{key, …}`.
    let raw_type = PostDocument::findnodes_foreign("ltx:bib-data[@role='self']", entry)
      .into_iter()
      .next()
      .and_then(|node| {
        let record = node.get_content();
        let name = record.trim_start().strip_prefix('@')?;
        let end = name.find(['{', '(']).unwrap_or(name.len());
        Some(name[..end].trim().to_lowercase())
      })
      .filter(|name| !name.is_empty());
    let title_type = raw_type.as_deref().unwrap_or(entry_type);
    let host = PostDocument::findnodes_foreign("ltx:bib-related[@role='host']", entry)
      .into_iter()
      .next();
    Fields {
      entry: entry.clone(),
      host,
      driver: Driver::of(entry_type),
      title_style: TitleStyle::of(title_type),
    }
  }

  fn own(&self, xpath: &str) -> Vec<Node> { PostDocument::findnodes_foreign(xpath, &self.entry) }

  fn in_host(&self, xpath: &str) -> Vec<Node> {
    self
      .host
      .as_ref()
      .map(|host| PostDocument::findnodes_foreign(xpath, host))
      .unwrap_or_default()
  }

  /// A plain-text field: the first of `nodes` that is not blank.
  fn plain(nodes: &[Node]) -> Option<String> {
    nodes
      .iter()
      .map(|node| collapse_ws(&node.get_content()))
      .find(|text| !text.is_empty())
  }

  /// A field that may hold markup (a title with a formula): the content, and the text it ends in.
  fn marked(nodes: &[Node]) -> Option<(Vec<NodeData>, Option<String>)> {
    let node = nodes.iter().find(|node| {
      !node.get_content().trim().is_empty() || node.get_first_element_child().is_some()
    })?;
    let content = crate::make_bibliography::field_content(node)
      .into_iter()
      .map(|item| match item {
        NodeData::Text(text) => NodeData::Text(collapse_ws_keep_edges(&text)),
        other => other,
      })
      .collect::<Vec<_>>();
    let content = trim_edges(content);
    let tail = crate::make_bibliography::trailing_text(&content);
    Some((content, tail))
  }

  fn names(&self, role: &str, in_host: bool) -> (Vec<Name>, bool) {
    let xpath = format!("ltx:bib-name[@role='{role}']");
    let nodes = if in_host {
      self.in_host(&xpath)
    } else {
      self.own(&xpath)
    };
    let mut names = Vec::new();
    let mut others = false;
    for node in nodes {
      let first = |tag: &str| {
        PostDocument::findnodes_foreign(tag, &node)
          .into_iter()
          .next()
          .map(|n| collapse_ws(&n.get_content()))
          .filter(|text| !text.is_empty())
      };
      let family = first("ltx:surname").unwrap_or_default();
      let given = first("ltx:givenname");
      if family == "others" && given.is_none() {
        others = true;
        continue;
      }
      names.push(Name {
        given,
        family,
        suffix: first("ltx:lineage"),
      });
    }
    (names, others)
  }

  fn part(&self, role: &str, in_host: bool) -> Option<String> {
    let xpath = format!("ltx:bib-part[@role='{role}']");
    Self::plain(&if in_host {
      self.in_host(&xpath)
    } else {
      self.own(&xpath)
    })
  }

  fn data(&self, role: &str) -> Option<String> {
    Self::plain(&self.own(&format!("ltx:bib-data[@role='{role}']")))
  }

  fn identifier(&self, scheme: &str) -> Option<String> {
    self
      .own(&format!("ltx:bib-identifier[@scheme='{scheme}']"))
      .iter()
      .find_map(|node| node.get_attribute("id").filter(|id| !id.trim().is_empty()))
  }

  fn year(&self) -> Option<String> { Self::plain(&self.own("ltx:bib-date[@role='publication']")) }
}

/// A field's content without the white space around it (`\unspace` cannot reach into the markup it is wrapped in).
fn trim_edges(mut content: Vec<NodeData>) -> Vec<NodeData> {
  if let Some(NodeData::Text(first)) = content.first_mut() {
    *first = first.trim_start().to_string();
  }
  if let Some(NodeData::Text(last)) = content.last_mut() {
    *last = last.trim_end().to_string();
  }
  content.retain(|node| !matches!(node, NodeData::Text(text) if text.is_empty()));
  content
}

fn collapse_ws_keep_edges(text: &str) -> String {
  let collapsed = collapse_ws(text);
  let mut out = String::new();
  if text.starts_with(char::is_whitespace) && !collapsed.is_empty() {
    out.push(' ');
  }
  out.push_str(&collapsed);
  if text.ends_with(char::is_whitespace) && !collapsed.is_empty() {
    out.push(' ');
  }
  out
}

// ======================================================================
// Field formats and the helper macros of biblatex.def

/// A field's content and the text it ends in.
type Marked = (Vec<NodeData>, Option<String>);

/// Whether `pages` is a number (`\ifnumeral`) or a list or range of them (`\ifnumerals`), biblatex.sty:3260-3330:
/// the former takes "p.", the latter "pp.", anything else (`e1234`) no prefix.
fn pages_prefix(pages: &str) -> Option<&'static str> {
  let range_char = |c: char| matches!(c, '~' | ',' | ';' | '-' | '+' | '/' | '–' | '—' | '&');
  let is_numeral = |s: &str| {
    let s = s.trim_matches('.');
    !s.is_empty()
      && (s.chars().all(|c| c.is_ascii_digit() || c == '.')
        || s.chars().all(|c| {
          matches!(
            c.to_ascii_lowercase(),
            'i' | 'v' | 'x' | 'l' | 'c' | 'd' | 'm'
          )
        }))
  };
  let items: Vec<&str> = pages
    .split(|c: char| range_char(c) || c.is_whitespace())
    .filter(|s| !s.is_empty())
    .collect();
  if items.is_empty() || !items.iter().all(|item| is_numeral(item)) {
    return None;
  }
  let single = items.len() == 1 && !pages.chars().any(range_char);
  Some(if single { "p." } else { "pp." })
}

/// english.lbx's `lang<name>` strings (english.lbx:473-507), by name.
const LANGUAGE_NAMES: &[(&str, &str)] = &[
  ("american", "American"),
  ("basque", "Basque"),
  ("brazilian", "Brazilian"),
  ("bulgarian", "Bulgarian"),
  ("catalan", "Catalan"),
  ("croatian", "Croatian"),
  ("czech", "Czech"),
  ("danish", "Danish"),
  ("dutch", "Dutch"),
  ("english", "English"),
  ("estonian", "Estonian"),
  ("finnish", "Finnish"),
  ("french", "French"),
  ("galician", "Galician"),
  ("german", "German"),
  ("greek", "Greek"),
  ("hungarian", "Hungarian"),
  ("italian", "Italian"),
  ("japanese", "Japanese"),
  ("latin", "Latin"),
  ("latvian", "Latvian"),
  ("lithuanian", "Lithuanian"),
  ("marathi", "Marathi"),
  ("norwegian", "Norwegian"),
  ("polish", "Polish"),
  ("portuguese", "Portuguese"),
  ("romanian", "Romanian"),
  ("russian", "Russian"),
  ("serbian", "Serbian"),
  ("slovak", "Slovak"),
  ("slovene", "Slovene"),
  ("spanish", "Spanish"),
  ("swedish", "Swedish"),
  ("turkish", "Turkish"),
  ("ukrainian", "Ukrainian"),
];

/// A given name's initials, `\namepartgiveni`: "John Ronald" as "J. R.", "Jean-Pierre" as "J.-P.", an initial already
/// written ("J.R.") as its letters.
fn initials(given: &str) -> String {
  given
    .split_whitespace()
    .map(|word| {
      word
        .split('-')
        .map(|segment| {
          let pieces: Vec<String> = segment
            .split('.')
            .filter_map(|piece| piece.chars().next())
            .map(|first| format!("{first}."))
            .collect();
          pieces.join("\u{a0}")
        })
        .collect::<Vec<_>>()
        .join("-")
    })
    .collect::<Vec<_>>()
    .join("\u{a0}")
}

/// `\mkbibordinal` of english.lbx for an `edition` that is an integer.
fn ordinal(n: u64) -> String {
  let suffix = match (n % 100, n % 10) {
    (11..=13, _) => "th",
    (_, 1) => "st",
    (_, 2) => "nd",
    (_, 3) => "rd",
    _ => "th",
  };
  format!("{n}{suffix}")
}

/// One side of a date, `YYYY[-MM[-DD]]`, as `\mkbibdatelong` (`short = false`, the default `date=comp` of
/// biblatex.sty:16436, english.lbx:23: "Jan. 6, 2025", "Jan. 2025", "2025") or `\mkbibdateshort` (`urldate=short`,
/// english.lbx:41: "01/06/2025") prints it; text that is not a date as written.
fn one_date(date: &str, short: bool) -> String {
  const MONTHS: [&str; 12] = [
    "Jan.", "Feb.", "Mar.", "Apr.", "May", "June", "July", "Aug.", "Sept.", "Oct.", "Nov.", "Dec.",
  ];
  let date = date.trim();
  let parts: Vec<&str> = date.split('-').collect();
  let numeric = !parts.is_empty()
    && parts
      .iter()
      .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()));
  let valid = numeric
    && parts[0].len() == 4
    && parts
      .get(1)
      .is_none_or(|m| m.parse::<usize>().is_ok_and(|m| (1..=12).contains(&m)))
    && parts.len() <= 3;
  if !valid {
    return date.to_string();
  }
  let month = parts.get(1).map(|m| m.parse::<usize>().unwrap_or(1));
  let day = parts.get(2).map(|d| d.parse::<usize>().unwrap_or(1));
  let year = parts[0];
  match (short, month, day) {
    (_, None, _) => year.to_string(),
    (true, Some(m), None) => format!("{m:02}/{year}"),
    (true, Some(m), Some(d)) => format!("{m:02}/{d:02}/{year}"),
    (false, Some(m), None) => format!("{} {year}", MONTHS[m - 1]),
    (false, Some(m), Some(d)) => format!("{}\u{a0}{d}, {year}", MONTHS[m - 1]),
  }
}

/// `\printdate` (`short`: `\printurldate`): a date, or a range `A/B` joined by `\bibdaterangesep`, an en dash.
fn date_text(date: &str, short: bool) -> String {
  match date.split_once('/') {
    Some((from, to)) if !to.trim().is_empty() => {
      format!("{}–{}", one_date(from, short), one_date(to, short))
    },
    Some((from, _)) => format!("{}–", one_date(from, short)),
    None => one_date(date, short),
  }
}

/// A `type` that biblatex has a string for (biblatex.def:586 `\ifbibstring{#1}{\bibstring{#1}}{#1}`) prints that
/// string, in english.lbx's short form (:340-360): the reader's own defaults for `@phdthesis`, `@mastersthesis` and
/// `@techreport` (bibtex.rs) read as the keys biblatex's legacy-type mapping gives them. The string (without its
/// final dot) and whether a dot follows.
fn type_bibstring(text: &str) -> Option<(&'static str, bool)> {
  match text.trim().replace('’', "'").to_lowercase().as_str() {
    "ph.d. thesis" | "phd thesis" | "phdthesis" => Some(("PhD thesis", false)),
    "master's thesis" | "mathesis" | "mastersthesis" => Some(("MA thesis", false)),
    "bachelor's thesis" | "bathesis" => Some(("BA thesis", false)),
    "technical report" | "techreport" => Some(("tech. rep", true)),
    "resreport" => Some(("research rep", true)),
    _ => None,
  }
}

impl Printer {
  // ---- name lists

  /// `\printnames{…}` (biblatex.sty:4680; name format `given-family`, biblatex.def:877): the names, cut to
  /// `minnames` when more than `maxnames` are given, and the "et al." of a list that says `and others`.
  fn print_names(&mut self, class: &str, names: &[Name], others: bool) {
    if names.is_empty() {
      self.no_unit();
      return;
    }
    let (max, min) = (self.layout.max_names, self.layout.min_names);
    let liststop = if names.len() > max {
      min.min(names.len())
    } else {
      names.len()
    };
    let more = names.len() > liststop || others;
    self.print(class, |p| {
      for (index, name) in names.iter().take(liststop).enumerate() {
        let count = index + 1;
        if count > 1 {
          // `name:delim` (biblatex.def:1138).
          if count < liststop || more {
            p.add_punct(P::Comma);
            p.add_space();
          } else {
            if liststop > 2 {
              p.add_punct(P::Comma);
            }
            p.add_space();
            p.text("and");
            p.add_space();
          }
        }
        // `name:given-family` (biblatex.def:1069).
        if let Some(given) = &name.given {
          if p.layout.given_initials {
            p.text(&initials(given));
          } else {
            p.text(given);
          }
          p.isdot();
          p.add_space();
        }
        p.text(&name.family);
        p.isdot();
        if let Some(suffix) = &name.suffix {
          p.add_space();
          p.text(suffix);
          p.isdot();
        }
        // `name:andothers` (biblatex.def:1162).
        if count == liststop && more {
          if liststop > 1 {
            p.add_punct(P::Comma);
          }
          p.add_space();
          p.text("et al");
          p.add_dot();
        }
      }
    });
  }

  /// `\printlist{…}` of `location`, `publisher`, `organization`, `institution` (biblatex.def:624-640): the items
  /// joined by `multilistdelim`, each abbreviation-safe (`#1\isdot`); more than `maxitems` (3) are cut to the first
  /// and "et al.".
  fn print_list(&mut self, class: &str, items: &[String]) {
    if items.is_empty() {
      self.no_unit();
      return;
    }
    let liststop = if items.len() > 3 { 1 } else { items.len() };
    let more = items.len() > liststop;
    self.print(class, |p| {
      for (index, item) in items.iter().take(liststop).enumerate() {
        let count = index + 1;
        if count > 1 {
          if count < liststop || more {
            p.add_punct(P::Comma);
            p.add_space();
          } else {
            if liststop > 2 {
              p.add_punct(P::Comma);
            }
            p.add_space();
            p.text("and");
            p.add_space();
          }
        }
        p.text(item);
        p.isdot();
        if count == liststop && more {
          p.add_space();
          p.text("et al");
          p.add_dot();
        }
      }
    });
  }

  /// `\printfield{…}` of a plain field with its format.
  fn print_plain(
    &mut self,
    class: &str,
    value: Option<String>,
    format: impl FnOnce(&mut Printer, &str),
  ) {
    match value {
      Some(value) => self.print(class, |p| format(p, &value)),
      None => self.no_unit(),
    }
  }

  /// `\printfield{…}` of a field that may hold markup.
  fn print_marked(
    &mut self,
    class: &str,
    value: Option<Marked>,
    format: impl FnOnce(&mut Printer, Vec<NodeData>, Option<String>),
  ) {
    match value {
      Some((content, tail)) => self.print(class, |p| format(p, content, tail)),
      None => self.no_unit(),
    }
  }

  /// `\printtext[title]{\printfield{title}\setunit{\subtitlepunct}\printfield{subtitle}}\newunit` and its siblings
  /// `booktitle`, `maintitle`, `journal`, `issue` (biblatex.def:3159-3250; they differ in the field format only):
  /// the title and its subtitles in the entry type's title format, then the unit mark.
  fn print_title(
    &mut self,
    class: &str,
    title: Option<Marked>,
    subtitles: Vec<Marked>,
    style: TitleStyle,
  ) {
    let mut parts = title.into_iter().chain(subtitles);
    let Some(first) = parts.next() else {
      self.no_unit();
      return;
    };
    let rest: Vec<Marked> = parts.collect();
    self.print(class, |p| {
      if style == TitleStyle::Quote {
        p.text("“");
      }
      let inner_mark = p.out.len();
      p.print_marked("", Some(first), |p, content, tail| p.nodes(content, tail));
      for part in rest {
        p.set_unit(NEWUNITPUNCT);
        p.print_marked("", Some(part), |p, content, tail| p.nodes(content, tail));
      }
      if style == TitleStyle::Quote {
        // `\mkbibquote{#1\isdot}`.
        p.isdot();
        p.text("”");
      } else if style == TitleStyle::Emph {
        // `\mkbibemph{#1}`.
        let body = p.out.split_off(inner_mark);
        p.out.push(emph(coalesce(body)));
      }
    });
    self.new_unit();
  }
}

// ======================================================================
// The bibliography drivers

/// How a driver starts: the name macro it uses. The translators never lead: `translator+others` tests
/// `\ifusetranslator`, false by default (biblatex.sty:16427, biblatex.def:2849-2859), so an entry with a translator
/// and nothing else starts with its title and says "Trans. by …" later.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Lead {
  /// `author`, and `author/translator+others` of the article, inbook, incollection and inproceedings drivers.
  Author,
  /// `author/editor` (manual): the plain `editor` macro, whose `editorstrg` never takes the translators along.
  AuthorEditor,
  /// `author/editor+others/translator+others` (book, booklet, misc, online).
  AuthorEditorOthers,
}

/// The macros and drivers read the fields of one entry; the printer is theirs to drive.
struct Run<'a> {
  fields:             &'a Fields,
  p:                  Printer,
  /// `\clearname{editor}` / `\clearname{translator}`: printed already, so not again by `byeditor+others`.
  editor_cleared:     bool,
  translator_cleared: bool,
}

impl Run<'_> {
  /// Whether the entry's publication data (publisher, location, series, volume, number, edition) live in its host.
  fn hosted(&self) -> bool {
    matches!(
      self.fields.driver,
      Driver::Incollection | Driver::Inbook | Driver::Inproceedings
    )
  }

  fn scoped_part(&self, role: &str) -> Option<String> { self.fields.part(role, self.hosted()) }

  fn scoped_list(&self, tag: &str) -> Vec<String> {
    let xpath = format!("ltx:{tag}");
    let mut nodes = if self.hosted() {
      self.fields.in_host(&xpath)
    } else {
      self.fields.own(&xpath)
    };
    // The reader files a proceedings' `location` under its conference (bibtex.rs
    // `\bib@field@inproceedings@location`), the place the volume was published.
    if tag == "bib-place"
      && matches!(
        self.fields.driver,
        Driver::Inproceedings | Driver::Proceedings
      )
    {
      nodes.extend(
        self
          .fields
          .own("ltx:bib-related[@role='event'][@type='conference']/ltx:bib-place"),
      );
    }
    nodes
      .iter()
      .map(|n| collapse_ws(&n.get_content()))
      .filter(|t| !t.is_empty())
      .collect()
  }

  /// `event+venue+date` (standard.bbx:828-843): "Conf (Paris, 05/2001)". The reader keeps the event of a proceedings
  /// or talk as `ltx:bib-related[@role='event']`: its title is the `eventtitle`, its place the `venue`.
  fn event_venue_date(&mut self) {
    let title = Fields::marked(
      &self
        .fields
        .own(".//ltx:bib-related[@role='event']/ltx:bib-title"),
    );
    let venue = Fields::plain(
      &self
        .fields
        .own(".//ltx:bib-related[@role='event'][@type='event']/ltx:bib-place"),
    );
    let date = Fields::plain(
      &self
        .fields
        .own(".//ltx:bib-related[@role='event']/ltx:bib-date"),
    )
    .map(|date| date_text(&date, false));
    self
      .p
      .print_marked("eventtitle", title, |p, content, tail| {
        p.nodes(content, tail)
      });
    self.p.new_unit();
    if venue.is_some() || date.is_some() {
      self.p.set_unit(SPACE);
      self.p.print("", |p| {
        p.text("(");
        p.print_plain("venue", venue, |p, text| p.text(text));
        p.set_unit_star(COMMA_SPACE);
        p.print_plain("eventdate", date, |p, text| p.text(text));
        p.text(")");
      });
    }
    self.p.new_unit();
  }

  // ---- name macros (biblatex.def:2734-3100)

  /// `\usebibmacro{author}`.
  fn author(&mut self) {
    let (names, others) = self.fields.names("author", false);
    self.p.print_names("author", &names, others);
  }

  /// Whether the entry's translators are its editors, name for name (`\ifnamesequal{editor}{translator}`).
  fn editors_are_translators(&self, hosted_editors: bool) -> bool {
    let (editors, editors_others) = self.fields.names("editor", hosted_editors);
    let (translators, translators_others) = self.fields.names("translator", false);
    !editors.is_empty()
      && !self.translator_cleared
      && editors_others == translators_others
      && editors.len() == translators.len()
      && editors
        .iter()
        .zip(&translators)
        .all(|(e, t)| e.given == t.given && e.family == t.family && e.suffix == t.suffix)
  }

  /// `\usebibmacro{editor+others}`: the names, "ed." / "eds." behind `\editortypedelim`; editors who are also the
  /// translators are "ed. and trans." (`editor+othersstrg`, biblatex.def:2911-2915).
  fn editor_plus_others(&mut self, with_translators: bool) {
    let (names, others) = self.fields.names("editor", false);
    if names.is_empty() {
      return;
    }
    let translators_too = with_translators && self.editors_are_translators(false);
    self.p.print_names("editor", &names, others);
    self.p.set_unit(COMMA_SPACE);
    let plural = names.len() > 1 || others;
    self.p.print("", |p| {
      p.bibstring(if plural { "eds" } else { "ed" }, true);
      if translators_too {
        p.add_space();
        p.text("and");
        p.add_space();
        p.bibstring("trans", true);
      }
    });
    self.editor_cleared = true;
    self.translator_cleared |= translators_too;
  }

  /// The name macro a driver starts with (biblatex.def:2734-2850): `author`, or `author/editor` where the driver takes
  /// the editors. The authors win.
  fn lead(&mut self, lead: Lead) {
    let (authors, _) = self.fields.names("author", false);
    if !authors.is_empty() {
      return self.author();
    }
    let (editors, _) = self.fields.names("editor", false);
    if !editors.is_empty() {
      match lead {
        Lead::Author => {},
        Lead::AuthorEditor => self.editor_plus_others(false),
        Lead::AuthorEditorOthers => self.editor_plus_others(true),
      }
    }
  }

  /// `\usebibmacro{byeditor+others}` (biblatex.def:3029): "ed. by A, B, and C", then `bytranslator+others`
  /// (`byeditor` of the manual driver: the editors alone).
  fn by_editor_plus_others(&mut self, translators: bool) {
    let (names, others) = self.fields.names("editor", self.hosted());
    if !names.is_empty() && !self.editor_cleared {
      // `byeditor+othersstrg` (biblatex.def:3062-3065): editors who are the translators too are "ed. and trans. by".
      let translators_too = translators && self.editors_are_translators(self.hosted());
      self.p.print("", |p| {
        p.bibstring("ed", true);
        if translators_too {
          p.add_space();
          p.text("and");
          p.add_space();
          p.bibstring("trans", true);
        }
        p.add_space();
        p.text("by");
      });
      self.translator_cleared |= translators_too;
      self.p.set_unit(SPACE);
      self.p.print_names("editor", &names, others);
      self.editor_cleared = true;
      self.p.new_unit();
    }
    if translators {
      self.by_translator_plus_others();
    }
  }

  /// `\usebibmacro{bytranslator+others}`: "trans. by A".
  fn by_translator_plus_others(&mut self) {
    let (names, others) = self.fields.names("translator", false);
    if !names.is_empty() && !self.translator_cleared {
      self.p.print("", |p| {
        p.bibstring("trans", true);
        p.add_space();
        p.text("by");
      });
      self.p.set_unit(SPACE);
      self.p.print_names("translator", &names, others);
      self.translator_cleared = true;
      self.p.new_unit();
    }
  }

  // ---- titles (biblatex.def:3159-3250)

  fn title_parts(&self, host: bool) -> (Option<Marked>, Vec<Marked>) {
    let (title, subtitles) = if host {
      (
        self.fields.in_host("ltx:bib-title"),
        self.fields.in_host("ltx:bib-subtitle"),
      )
    } else {
      (
        self.fields.own("ltx:bib-title"),
        self.fields.own("ltx:bib-subtitle"),
      )
    };
    let subtitles = subtitles
      .iter()
      .filter_map(|node| Fields::marked(std::slice::from_ref(node)))
      .collect();
    (Fields::marked(&title), subtitles)
  }

  /// `\usebibmacro{title}`.
  fn title(&mut self) {
    let (title, subtitles) = self.title_parts(false);
    let style = self.fields.title_style;
    self.p.print_title("title", title, subtitles, style);
  }

  /// `\usebibmacro{booktitle}`: the host's title.
  fn booktitle(&mut self) {
    let (mut title, subtitles) = self.title_parts(true);
    // `bookinbook` and `suppbook` have no host in the reader: their `booktitle` is plain data.
    if title.is_none() && self.fields.host.is_none() {
      title = Fields::marked(&self.fields.own("ltx:bib-data[@role='booktitle']"));
    }
    self
      .p
      .print_title("booktitle", title, subtitles, TitleStyle::Emph);
  }

  /// `\usebibmacro{journal}`, `\DeclareFieldFormat{journaltitle}{\mkbibemph{#1\isdot}}`.
  fn journal(&mut self) {
    let (title, _) = self.title_parts(true);
    self.p.print_marked("journal", title, |p, content, tail| {
      let mark = p.out.len();
      p.nodes(content, tail);
      p.isdot();
      let body = p.out.split_off(mark);
      p.out.push(emph(coalesce(body)));
    });
    self.p.new_unit();
  }

  /// `\usebibmacro{in:}`.
  fn in_(&mut self) {
    self.p.print("", |p| p.bibstring("in", false));
    self.p.print_unit(COLON_SPACE);
  }

  // ---- fields

  fn date(&mut self) {
    let date = self.fields.year().map(|date| date_text(&date, false));
    self.p.print_plain("date", date, |p, text| p.text(text));
  }

  /// `volume+number+eid` of a journal (standard.bbx:799-804).
  fn volume_number_eid(&mut self) {
    let volume = self.fields.part("volume", false);
    self.p.print_plain("volume", volume, |p, text| p.text(text));
    self.p.set_unit_star(DOT);
    let number = self.fields.part("number", false);
    self.p.print_plain("number", number, |p, text| p.text(text));
    self.p.set_unit(COMMA_SPACE);
    let eid = self.fields.data("eid");
    self.p.print_plain("eid", eid, |p, text| p.text(text));
  }

  /// `journal+issuetitle` (standard.bbx:784-797).
  fn journal_plus_issuetitle(&mut self) {
    self.journal();
    self.p.set_unit_star(SPACE);
    // The journal's series (biblatex.def:574-577): an integer is "3rd ser.", a bibstring key its string.
    if let Some(series) = self.fields.part("series", false) {
      self.p.new_unit();
      self.p.print_plain("series", Some(series), |p, text| {
        match text.parse::<u64>() {
          Ok(n) => {
            p.text(&ordinal(n));
            p.text("\u{a0}");
            p.bibstring("ser", true);
          },
          Err(_) => match text {
            "newseries" => p.bibstring("new ser", true),
            "oldseries" => p.bibstring("old ser", true),
            _ => p.text(text),
          },
        }
      });
      self.p.set_unit(SPACE);
    }
    self.volume_number_eid();
    self.p.set_unit(SPACE);
    // `issue+date`: `\printtext[parens]{\printfield{issue}\setunit*{\addspace}\usebibmacro{date}}\newunit`.
    // `\printtext` tests its code for blankness, not what it prints: an article with neither issue nor date gets
    // biblatex's empty "()" too.
    let issue = self.fields.data("issue");
    let date = self.fields.year().map(|date| date_text(&date, false));
    self.p.print("", |p| {
      p.text("(");
      p.print_plain("issue", issue, |p, text| p.text(text));
      p.set_unit_star(SPACE);
      p.print_plain("date", date, |p, text| p.text(text));
      p.text(")");
    });
    self.p.new_unit();
    self.p.set_unit(COLON_SPACE);
    // `\usebibmacro{issue}`: the issue title, nested under the journal by the reader, in emphasis (biblatex.def:562).
    let part = "ltx:bib-related[@role='part'][@type='issue']";
    let title = Fields::marked(&self.fields.in_host(&format!("{part}/ltx:bib-title")));
    let subtitles: Vec<Marked> = self
      .fields
      .in_host(&format!("{part}/ltx:bib-subtitle"))
      .iter()
      .filter_map(|node| Fields::marked(std::slice::from_ref(node)))
      .collect();
    self
      .p
      .print_title("issuetitle", title, subtitles, TitleStyle::Emph);
    self.p.new_unit();
  }

  /// `\printfield{pages}` with biblatex.def:570 `\mkpageprefix[bookpagination]`.
  fn pages(&mut self) {
    let pages = self.fields.part("pages", false);
    self.p.print_plain("pages", pages, |p, text| {
      if let Some(prefix) = pages_prefix(text) {
        p.bibstring(&prefix.trim_end_matches('.').to_lowercase(), true);
        p.text("\u{a0}");
      }
      p.text(text);
    });
  }

  /// `chapter+pages` (standard.bbx:887-894).
  fn chapter_plus_pages(&mut self) {
    let chapter = self.fields.part("chapter", false);
    self.p.print_plain("chapter", chapter, |p, text| {
      p.bibstring("chap", true);
      p.text("\u{a0}");
      p.text(text);
    });
    self.p.set_unit(COMMA_SPACE);
    let eid = self.fields.data("eid");
    self.p.print_plain("eid", eid, |p, text| p.text(text));
    self.p.set_unit(COMMA_SPACE);
    self.pages();
    self.p.new_unit();
  }

  /// `note+pages` (standard.bbx:895-900).
  fn note_plus_pages(&mut self) {
    self.note();
    self.p.set_unit(COMMA_SPACE);
    self.pages();
    self.p.new_unit();
  }

  /// `\printfield{note}`: `\DeclareFieldFormat{note}{#1\isdot}`.
  fn note(&mut self) {
    let note = Fields::plain(&self.fields.own("ltx:bib-note[@role='annotation']"));
    self.p.print_plain("note", note, |p, text| {
      p.text(text);
      p.isdot();
    });
  }

  /// `\printfield{howpublished}`.
  fn howpublished(&mut self) {
    let note = Fields::plain(&self.fields.own("ltx:bib-note[@role='publication']"));
    self
      .p
      .print_plain("howpublished", note, |p, text| p.text(text));
  }

  fn type_(&mut self) {
    let ty = Fields::plain(&self.fields.own("ltx:bib-type"));
    self
      .p
      .print_plain("type", ty, |p, text| match type_bibstring(text) {
        Some((string, dot)) => p.bibstring(string, dot),
        None => p.text(text),
      });
  }

  fn version(&mut self) {
    let version = self.fields.data("version");
    self.p.print_plain("version", version, |p, text| {
      p.bibstring("version", false);
      p.text("\u{a0}");
      p.text(text);
    });
  }

  /// `\printlist{language}` (biblatex.def:641-654). `\blx@clearlang` (biblatex.sty:7058-7078) clears a list of one item
  /// that has a `blx@rlm@` entry, which english.lbx:4 `\DeclareRedundantLanguages{english,american}{…}` makes for
  /// exactly `english`, `american`, `langenglish` and `langamerican` (every lookup is case-sensitive). The others are
  /// english.lbx's `lang<name>` strings (the `lang` may be written too) or as written.
  fn language(&mut self) {
    let nodes = self.fields.own("ltx:bib-language");
    let items: Vec<String> = Fields::plain(&nodes)
      .map(|text| {
        text
          .split(" and ")
          .map(|item| item.trim().to_string())
          .filter(|item| !item.is_empty())
          .collect()
      })
      .unwrap_or_default();
    if let [only] = items.as_slice()
      && matches!(
        only.as_str(),
        "english" | "american" | "langenglish" | "langamerican"
      )
    {
      return self.p.no_unit();
    }
    let strings: Vec<String> = items
      .iter()
      .map(|item| {
        let name = item.strip_prefix("lang").unwrap_or(item);
        LANGUAGE_NAMES
          .iter()
          .find(|(key, _)| *key == name)
          .map_or_else(|| item.clone(), |(_, string)| (*string).to_string())
      })
      .collect();
    self.p.print_list("language", &strings);
  }

  fn edition(&mut self) {
    let edition = if self.hosted() {
      Fields::plain(&self.fields.in_host("ltx:bib-edition"))
    } else {
      Fields::plain(&self.fields.own("ltx:bib-edition"))
    };
    self
      .p
      .print_plain("edition", edition, |p, text| match text.parse::<u64>() {
        Ok(n) => {
          p.text(&ordinal(n));
          p.text("\u{a0}");
          p.bibstring("ed", true);
        },
        Err(_) => {
          p.text(text);
          p.isdot();
        },
      });
  }

  /// `\iffieldundef{maintitle}{\printfield{volume}\printfield{part}}{}`: "Vol. 3".
  fn volume_part(&mut self) {
    let volume = self.scoped_part("volume");
    self.p.print_plain("volume", volume, |p, text| {
      p.bibstring("vol", true);
      p.text("\u{a0}");
      p.text(text);
    });
    let part = self.scoped_part("part");
    self.p.print_plain("part", part, |p, text| {
      p.text(".");
      p.text(text);
    });
  }

  /// `series+number` (standard.bbx:845-850).
  fn series_plus_number(&mut self) {
    let series = self.scoped_part("series");
    self.p.print_plain("series", series, |p, text| p.text(text));
    self.p.set_unit_star(SPACE);
    let number = self.scoped_part("number");
    self.p.print_plain("number", number, |p, text| p.text(text));
    self.p.new_unit();
  }

  /// `publisher+location+date`, `institution+location+date`, `organization+location+date` and `location+date`
  /// (standard.bbx:851-886): the list in front of the date decides the separator after the location.
  fn agent_location_date(&mut self, agent: Option<(&str, Vec<String>)>) {
    let location = self.scoped_list("bib-place");
    self.p.print_list("location", &location);
    let has_agent = agent.as_ref().is_some_and(|(_, items)| !items.is_empty());
    match (agent.is_some(), has_agent) {
      (true, true) => self.p.set_unit_star(COLON_SPACE),
      _ => self.p.set_unit_star(COMMA_SPACE),
    }
    if let Some((class, items)) = agent {
      self.p.print_list(class, &items);
      self.p.set_unit_star(COMMA_SPACE);
    }
    self.date();
    self.p.new_unit();
  }

  fn publisher_location_date(&mut self) {
    let publisher = self.scoped_list("bib-publisher");
    self.agent_location_date(Some(("publisher", publisher)));
  }

  fn institution_location_date(&mut self) {
    let institution = self.scoped_list("bib-organization");
    self.agent_location_date(Some(("institution", institution)));
  }

  fn organization_location_date(&mut self) {
    let organization = self.scoped_list("bib-organization");
    self.agent_location_date(Some(("organization", organization)));
  }

  fn location_date(&mut self) { self.agent_location_date(None); }

  /// `\printfield{isbn}` and its relatives, behind `\iftoggle{bbx:isbn}`: `\mkbibacro{ISBN}\addcolon\space #1`.
  fn identifier(&mut self, scheme: &str, acronym_text: &str) {
    let id = self.fields.identifier(scheme);
    self.p.print_plain(scheme, id, |p, text| {
      p.nodes(vec![acronym(acronym_text)], Some(acronym_text.to_string()));
      p.add_punct(P::Colon);
      p.add_space();
      p.text(text);
    });
  }

  /// `doi+eprint+url` (standard.bbx:901-913).
  fn doi_eprint_url(&mut self) {
    if self.p.layout.doi {
      if let Some(doi) = self.fields.identifier("doi") {
        self.p.print("doi", |p| {
          p.nodes(vec![acronym("DOI")], Some("DOI".to_string()));
          p.add_punct(P::Colon);
          p.add_space();
          p.nodes(
            vec![link(
              &crate::make_bibliography::doi_href(&doi),
              "doi ltx_bib_external",
              vec![NodeData::Text(doi.clone())],
            )],
            Some(doi.clone()),
          );
        });
      } else {
        self.p.no_unit();
      }
    }
    self.p.new_unit_block();
    if self.p.layout.eprint {
      self.eprint();
    }
    self.p.new_unit_block();
    // standard.bbx:23 `\ExecuteBibliographyOptions[online]{url=true}`: an `@online` entry shows its url whatever
    // `url` says.
    if self.p.layout.url || self.fields.driver == Driver::Online {
      self.url_plus_urldate();
    }
  }

  /// `\usebibmacro{eprint}` (biblatex.def:3286): `eprint:arxiv` of biblatex.def:526, "arXiv: 2301.01266 [math.AG]",
  /// the id and the class one link; any other `eprinttype` has its own format (:520-556): "JSTOR: 123", else
  /// "<type>: <value>".
  fn eprint(&mut self) {
    let arxiv = self.fields.identifier("arxiv");
    let class = self
      .fields
      .data("eprintclass")
      .or_else(|| self.fields.data("primaryclass"));
    let href = self
      .fields
      .own("ltx:bib-identifier[@scheme='arxiv']")
      .iter()
      .find_map(|node| node.get_attribute("href"))
      .unwrap_or_default();
    if arxiv.is_some() {
      return self.p.print_plain("eprint", arxiv, |p, id| {
        p.text("arXiv");
        p.add_punct(P::Colon);
        p.add_space();
        let mut inner = vec![NodeData::Text(id.to_string())];
        if let Some(class) = &class {
          inner.push(NodeData::Text(" ".to_string()));
          inner.push(wrap("ltx:text", &[("font", "typewriter")], vec![
            NodeData::Text(format!("[{class}]")),
          ]));
        }
        let tail = class
          .as_ref()
          .map_or(id.to_string(), |class| format!("[{class}]"));
        // The reader gives the id's address (`https://arxiv.org/abs/<id>`); without one it is the same.
        let href = if href.is_empty() {
          format!("https://arxiv.org/abs/{id}")
        } else {
          href.clone()
        };
        p.nodes(
          vec![link(&href, "arxiv ltx_bib_external", inner)],
          Some(tail),
        );
      });
    }
    // The reader files an eprint that is not arXiv's as `ltx:bib-links[@class='eprint']`, its type in `bib-data`.
    let Some(value) = Fields::plain(&self.fields.own("ltx:bib-links[@class='eprint']")) else {
      return self.p.no_unit();
    };
    let kind = self
      .fields
      .data("eprinttype")
      .or_else(|| self.fields.data("archiveprefix"));
    // biblatex.def:520-556: the types with a format of their own, else `<type>: <id>` (`eprint: <id>` without one)
    // and the class in parentheses.
    let (label, base): (String, Option<&str>) =
      match kind.as_deref().map(str::to_lowercase).as_deref() {
        Some("hdl") => ("HDL".to_string(), Some("https://hdl.handle.net/")),
        Some("jstor") => ("JSTOR".to_string(), Some("http://www.jstor.org/stable/")),
        Some("pubmed") => (
          "PMID".to_string(),
          Some("http://www.ncbi.nlm.nih.gov/pubmed/"),
        ),
        Some("googlebooks") => (
          "Google Books".to_string(),
          Some("http://books.google.com/books?id="),
        ),
        _ => (kind.clone().unwrap_or_else(|| "eprint".to_string()), None),
      };
    self.p.print_plain("eprint", Some(value), |p, value| {
      p.text(&label);
      p.add_punct(P::Colon);
      p.add_space();
      match base {
        Some(base) => {
          let href = format!("{base}{value}");
          p.nodes(
            vec![link(&href, "ltx_bib_external", vec![NodeData::Text(
              value.to_string(),
            )])],
            Some(value.to_string()),
          );
        },
        None => {
          p.text(value);
          if let Some(class) = &class {
            p.add_space();
            p.text(&format!("({class})"));
          }
        },
      }
    });
  }

  /// `url+urldate` (biblatex.def:3258).
  fn url_plus_urldate(&mut self) {
    let url = self
      .fields
      .own("ltx:bib-url")
      .into_iter()
      .find_map(|node| node.get_attribute("href"));
    self.p.print_plain("url", url, |p, href| {
      p.nodes(vec![acronym("URL")], Some("URL".to_string()));
      p.add_punct(P::Colon);
      p.add_space();
      p.nodes(
        vec![link(href, "ltx_bib_external", vec![NodeData::Text(
          href.to_string(),
        )])],
        Some(href.to_string()),
      );
    });
    if let Some(seen) = Fields::plain(&self.fields.own("ltx:bib-date[@role='accessed']")) {
      self.p.set_unit_star(SPACE);
      let text = date_text(&seen, true);
      self.p.print("urldate", |p| {
        p.text("(");
        p.bibstring("visited on", false);
        p.add_space();
        p.text(&text);
        p.text(")");
      });
    }
  }

  /// `addendum+pubstate`.
  fn addendum_plus_pubstate(&mut self) {
    let addendum = Fields::plain(&self.fields.own("ltx:bib-note[@role='addendum']"));
    self
      .p
      .print_plain("addendum", addendum, |p, text| p.text(text));
    self.p.new_unit_block();
    let pubstate = self.fields.data("pubstate");
    // `\DeclareFieldFormat{pubstate}{\ifbibstring{#1}{\bibstring{#1}}{#1}}` (biblatex.def:578), english.lbx:433-437.
    self
      .p
      .print_plain("pubstate", pubstate, |p, text| match text {
        "inpress" => p.bibstring("in press", false),
        "inpreparation" => p.bibstring("in preparation", false),
        "submitted" | "forthcoming" => p.bibstring(text, false),
        "prepublished" => p.bibstring("pre-published", false),
        _ => p.text(text),
      });
  }

  fn pagetotal(&mut self) {
    let total = self.fields.data("pagetotal");
    self.p.print_plain("pagetotal", total, |p, text| {
      if text.chars().all(|c| c.is_ascii_digit()) {
        p.text(text);
        p.text("\u{a0}");
        p.bibstring(if text == "1" { "p" } else { "pp" }, true);
      } else {
        p.text(text);
      }
    });
  }

  /// `\usebibmacro{pageref}`, `related`: not printed (the web page has no pages; no related entries).
  fn tail(&mut self) {
    self.addendum_plus_pubstate();
    self.p.set_unit(SPACE);
    self.p.new_block();
  }

  // ---- the drivers (standard.bbx:28-733)

  /// The head every driver shares: `\setunit{\printdelim{nametitledelim}}\newblock\usebibmacro{title}\newunit
  /// \printlist{language}\newunit\newblock\usebibmacro{byauthor}\newunit\newblock`.
  fn after_names_title(&mut self) {
    self.p.set_unit(NEWUNITPUNCT);
    self.p.new_block();
  }

  fn title_language(&mut self) {
    self.title();
    self.p.new_unit();
    self.language();
    self.p.new_unit_block();
    self.p.new_unit_block();
  }

  fn finish_entry(&mut self) {
    self.doi_eprint_url();
    self.p.new_unit_block();
    self.addendum_plus_pubstate();
  }

  fn article(&mut self) {
    self.lead(Lead::Author);
    self.after_names_title();
    self.title_language();
    self.by_translator_plus_others();
    self.p.new_unit_block();
    self.version();
    self.p.new_unit_block();
    self.in_();
    self.journal_plus_issuetitle();
    self.p.new_unit();
    self.by_editor_plus_others(true);
    self.p.new_unit();
    self.note_plus_pages();
    self.p.new_unit_block();
    if self.p.layout.isbn {
      self.identifier("issn", "ISSN");
    }
    self.p.new_unit_block();
    self.finish_entry();
  }

  fn book_like(&mut self, kind: Driver) {
    match kind {
      Driver::Collection | Driver::Proceedings => self.editor_plus_others(true),
      _ => self.lead(Lead::AuthorEditorOthers),
    }
    self.after_names_title();
    // maintitle+title: no `maintitle` in `ltx:bibentry`.
    self.title();
    self.p.new_unit();
    self.language();
    self.p.new_unit_block();
    self.p.new_unit_block();
    if kind == Driver::Proceedings {
      self.event_venue_date();
      self.p.new_unit_block();
    }
    self.by_editor_plus_others(true);
    self.p.new_unit_block();
    // standard.bbx:569-615: a proceedings volume has no edition.
    if kind != Driver::Proceedings {
      self.edition();
    }
    self.p.new_unit();
    self.volume_part();
    self.p.new_unit();
    self.p.new_unit_block();
    self.series_plus_number();
    self.p.new_unit_block();
    self.note();
    self.p.new_unit_block();
    if kind == Driver::Proceedings {
      let organization = self.scoped_list("bib-organization");
      self.p.print_list("organization", &organization);
      self.p.new_unit();
    }
    self.publisher_location_date();
    self.p.new_unit_block();
    self.chapter_plus_pages();
    self.p.new_unit();
    self.pagetotal();
    self.p.new_unit_block();
    if self.p.layout.isbn {
      self.identifier("isbn", "ISBN");
    }
    self.p.new_unit_block();
    self.finish_entry();
  }

  fn in_book_like(&mut self, kind: Driver) {
    self.lead(Lead::Author);
    self.after_names_title();
    self.title_language();
    if kind == Driver::Inproceedings {
      self.in_();
      self.booktitle();
      self.p.new_unit_block();
      self.event_venue_date();
      self.p.new_unit_block();
    } else {
      self.in_();
      self.booktitle();
      self.p.new_unit_block();
    }
    self.by_editor_plus_others(true);
    self.p.new_unit_block();
    if kind != Driver::Inproceedings {
      self.edition();
      self.p.new_unit();
    }
    self.volume_part();
    self.p.new_unit();
    self.p.new_unit_block();
    self.series_plus_number();
    self.p.new_unit_block();
    self.note();
    self.p.new_unit_block();
    if kind == Driver::Inproceedings {
      let organization = self.scoped_list("bib-organization");
      self.p.print_list("organization", &organization);
      self.p.new_unit();
    }
    self.publisher_location_date();
    self.p.new_unit_block();
    self.chapter_plus_pages();
    self.p.new_unit_block();
    if self.p.layout.isbn {
      self.identifier("isbn", "ISBN");
    }
    self.p.new_unit_block();
    self.finish_entry();
  }

  fn misc_like(&mut self, kind: Driver) {
    self.lead(if kind == Driver::Manual {
      Lead::AuthorEditor
    } else {
      Lead::AuthorEditorOthers
    });
    self.after_names_title();
    self.title();
    self.p.new_unit();
    self.language();
    self.p.new_unit_block();
    self.p.new_unit_block();
    self.by_editor_plus_others(kind != Driver::Manual);
    self.p.new_unit_block();
    match kind {
      Driver::Online => {
        self.version();
        self.p.new_unit();
        self.note();
        self.p.new_unit_block();
        let organization = self.scoped_list("bib-organization");
        self.p.print_list("organization", &organization);
        self.p.new_unit_block();
        self.date();
        self.p.new_unit_block();
      },
      Driver::Booklet => {
        self.howpublished();
        self.p.new_unit_block();
        self.type_();
        self.p.new_unit_block();
        self.note();
        self.p.new_unit_block();
        self.location_date();
        self.p.new_unit_block();
        self.chapter_plus_pages();
        self.p.new_unit();
        self.pagetotal();
        self.p.new_unit_block();
      },
      Driver::Manual => {
        self.edition();
        self.p.new_unit_block();
        self.series_plus_number();
        self.p.new_unit_block();
        self.type_();
        self.p.new_unit();
        self.version();
        self.p.new_unit();
        self.note();
        self.p.new_unit_block();
        let organization = self.scoped_list("bib-organization");
        self.p.print_list("organization", &organization);
        self.p.new_unit();
        self.publisher_location_date();
        self.p.new_unit_block();
        self.chapter_plus_pages();
        self.p.new_unit();
        self.pagetotal();
        self.p.new_unit_block();
        if self.p.layout.isbn {
          self.identifier("isbn", "ISBN");
        }
        self.p.new_unit_block();
      },
      _ => {
        self.howpublished();
        self.p.new_unit_block();
        self.type_();
        self.p.new_unit();
        self.version();
        self.p.new_unit();
        self.note();
        self.p.new_unit_block();
        self.organization_location_date();
        self.p.new_unit_block();
      },
    }
    self.finish_entry();
  }

  fn report_like(&mut self, kind: Driver) {
    self.lead(Lead::Author);
    self.after_names_title();
    self.title();
    self.p.new_unit();
    self.language();
    self.p.new_unit_block();
    self.p.new_unit_block();
    if kind == Driver::Thesis {
      self.note();
      self.p.new_unit_block();
      self.type_();
      self.p.new_unit();
    } else {
      self.type_();
      self.p.set_unit_star(SPACE);
      let number = self.fields.part("number", false);
      self.p.print_plain("number", number, |p, text| p.text(text));
      self.p.new_unit_block();
      self.version();
      self.p.new_unit();
      self.note();
      self.p.new_unit_block();
    }
    self.institution_location_date();
    self.p.new_unit_block();
    self.chapter_plus_pages();
    self.p.new_unit();
    self.pagetotal();
    self.p.new_unit_block();
    if self.p.layout.isbn {
      // `isrn` for a report, `isbn` for a thesis.
      self.identifier(
        if kind == Driver::Thesis {
          "isbn"
        } else {
          "isrn"
        },
        if kind == Driver::Thesis {
          "ISBN"
        } else {
          "ISRN"
        },
      );
    }
    self.p.new_unit_block();
    self.finish_entry();
  }

  fn unpublished(&mut self) {
    self.lead(Lead::Author);
    self.after_names_title();
    self.title_language();
    self.howpublished();
    self.p.new_unit_block();
    self.type_();
    self.p.new_unit_block();
    self.event_venue_date();
    self.p.new_unit_block();
    self.note();
    self.p.new_unit_block();
    self.location_date();
    self.p.new_unit_block();
    self.finish_entry();
  }

  fn run(&mut self) {
    match self.fields.driver {
      Driver::Article => self.article(),
      Driver::Book | Driver::Collection | Driver::Proceedings => {
        let kind = self.fields.driver;
        self.book_like(kind)
      },
      Driver::Inbook | Driver::Incollection | Driver::Inproceedings => {
        let kind = self.fields.driver;
        self.in_book_like(kind)
      },
      Driver::Thesis | Driver::Report => {
        let kind = self.fields.driver;
        self.report_like(kind)
      },
      Driver::Unpublished => self.unpublished(),
      kind @ (Driver::Booklet | Driver::Manual | Driver::Misc | Driver::Online) => {
        self.misc_like(kind)
      },
    }
  }
}

/// The bibliography entry as biblatex's standard drivers print it: the content of the one `ltx:bibblock` of the
/// entry's `ltx:bibitem`.
pub(crate) fn format_entry(layout: &Layout, bibentry: &Node, entry_type: &str) -> Vec<NodeData> {
  let fields = Fields::new(bibentry, entry_type);
  let mut run = Run {
    fields:             &fields,
    p:                  Printer::new(layout.clone()),
    editor_cleared:     false,
    translator_cleared: false,
  };
  run.run();
  run.p.finish()
}

#[cfg(test)]
mod tests {
  use super::*;

  fn text_of(nodes: &[NodeData]) -> String {
    nodes
      .iter()
      .map(|node| match node {
        NodeData::Text(text) => text.clone(),
        NodeData::Element { children, .. } => text_of(children),
        NodeData::XmlNode(node) => node.get_content(),
      })
      .collect()
  }

  fn printer() -> Printer {
    Printer::new(Layout::from_bibstyle("biblatex;layout=standard").expect("a standard layout"))
  }

  #[test]
  fn layout_reads_the_options_the_binding_records() {
    assert_eq!(Layout::from_bibstyle("biblatex"), None);
    assert_eq!(Layout::from_bibstyle("plain;layout=standard"), None);
    let layout =
      Layout::from_bibstyle("biblatex-giveninits;layout=standard;maxnames=5;minnames=9;isbn=false")
        .expect("a standard layout");
    // A maximum below the minimum is raised to it (biblatex.sty:15365-15377).
    assert_eq!((layout.max_names, layout.min_names), (9, 9));
    assert!(!layout.isbn && layout.doi && layout.url && layout.eprint && layout.given_initials);
  }

  /// biblatex.sty:2026-2035: no period after a mark, a comma after an abbreviation's dot but not after a sentence's.
  #[test]
  fn the_tracker_adds_a_mark_only_where_biblatex_does() {
    let mut p = printer();
    p.text("What is X?");
    p.add_punct(P::Period);
    assert_eq!(text_of(&p.out), "What is X?");
    let mut p = printer();
    p.text("Amer. J. Math.");
    p.isdot();
    p.add_punct(P::Comma);
    assert_eq!(text_of(&p.out), "Amer. J. Math.,");
    let mut p = printer();
    p.text("Birkhäuser Boston, Boston, MA.");
    p.add_punct(P::Comma);
    assert_eq!(text_of(&p.out), "Birkhäuser Boston, Boston, MA.");
    let mut p = printer();
    p.text("Title (2nd ed.)");
    p.add_punct(P::Period);
    assert_eq!(text_of(&p.out), "Title (2nd ed.)");
  }

  /// A unit's mark is printed in front of the next unit that prints, once, whatever was asked in between.
  #[test]
  fn units_are_separated_by_the_pending_mark() {
    let mut p = printer();
    p.print_plain("a", Some("One".to_string()), |p, t| p.text(t));
    p.new_unit();
    p.print_plain("b", None, |p, t| p.text(t));
    p.new_unit_block();
    p.print_plain("c", Some("Two".to_string()), |p, t| p.text(t));
    p.set_unit_star(COMMA_SPACE);
    p.print_plain("d", Some("Three".to_string()), |p, t| p.text(t));
    assert_eq!(text_of(&coalesce(p.out.clone())), "One. Two, Three");
    let mut p = printer();
    p.print_plain("a", Some("One".to_string()), |p, t| p.text(t));
    p.set_unit_star(COMMA_SPACE);
    p.print_plain("b", Some("Two".to_string()), |p, t| p.text(t));
    assert_eq!(text_of(&p.finish()), "One, Two.");
  }

  #[test]
  fn names_are_cut_to_minnames_past_maxnames() {
    let names = |count: usize| -> Vec<Name> {
      (0..count)
        .map(|i| Name {
          given:  Some(format!("G{i}")),
          family: format!("F{i}"),
          suffix: None,
        })
        .collect()
    };
    let mut layout =
      Layout::from_bibstyle("biblatex;layout=standard;maxnames=3").expect("a layout");
    layout.min_names = 2;
    let mut p = Printer::new(layout.clone());
    p.print_names("author", &names(3), false);
    assert_eq!(text_of(&p.out), "G0 F0, G1 F1, and G2 F2");
    let mut p = Printer::new(layout.clone());
    p.print_names("author", &names(2), false);
    assert_eq!(text_of(&p.out), "G0 F0 and G1 F1");
    let mut p = Printer::new(layout.clone());
    p.print_names("author", &names(4), false);
    assert_eq!(text_of(&p.out), "G0 F0, G1 F1, et al.");
    let mut p = Printer::new(layout);
    p.print_names("author", &names(1), true);
    assert_eq!(text_of(&p.out), "G0 F0 et al.");
  }

  #[test]
  fn formats_of_the_standard_fields() {
    assert_eq!(pages_prefix("5"), Some("p."));
    assert_eq!(pages_prefix("1337–1398"), Some("pp."));
    assert_eq!(pages_prefix("045, 13"), Some("pp."));
    assert_eq!(pages_prefix("vii-xii"), Some("pp."));
    assert_eq!(pages_prefix("e12345"), None);
    assert_eq!(ordinal(2), "2nd");
    assert_eq!(ordinal(11), "11th");
    assert_eq!(ordinal(23), "23rd");
    assert_eq!(date_text("2008", false), "2008");
    assert_eq!(date_text("1999-03", false), "Mar. 1999");
    assert_eq!(date_text("2025-01-06", false), "Jan.\u{a0}6, 2025");
    assert_eq!(date_text("1974/1975", false), "1974–1975");
    assert_eq!(date_text("2021-05-17", true), "05/17/2021");
    assert_eq!(initials("Jean-Pierre Marie"), "J.-P.\u{a0}M.");
    assert_eq!(initials("A.B."), "A.\u{a0}B.");
  }
}
