//! futs font encoding: the Fourier ornaments font (`\FourierOrns`, fourier-orns.sty:29), selected by
//! `latexml_contrib`'s `fourier_orns_sty` through `\lx@fontencoding{futs}`. Perl has no `futs.fontmap.ltxml`
//! (`futs` is only a serif family, Common/Font.pm:145). Each slot is the name fourier-orns.sty:33-72 gives it, as
//! the nearest Unicode ornament: the aldines are the `\grappe`/`\leftgrappe`/`\rightgrappe` glyphs the package's own
//! TU branch uses (fourier-orns.sty:80-82 `\char"2766`/`"2619`/`"2767`); the decorative curls
//! (`\decofourright`…`\decothreeleft`), `\textthing` and the small aldines take the floral heart, the old
//! pilcrows `¶`, `\decosix` the white four-pointed star. Slots the package never uses stay undecoded. The
//! encoding is not attached to the `futs` family, which is also Fourier's Utopia text family. Guard:
//! `native_notes_bindings::fourier_orns_print_their_ornaments`.
use crate::prelude::*;

LoadDefinitions!({
  // slot (the character `\FourierOrns` is followed by) => ornament
  #[rustfmt::skip]
  let slots: &[(char, char)] = &[
    ('0', '\u{26D4}'),  // \noway
    ('1', '\u{26A0}'),  // \warning
    ('2', '\u{2621}'),  // \caution
    ('3', '\u{00B6}'), ('4', '\u{00B6}'), ('5', '\u{00B6}'), // \oldpilcrowone…three
    ('6', '\u{00B6}'), ('7', '\u{00B6}'), ('8', '\u{00B6}'), // \oldpilcrowfour…six
    ('9', '\u{1F4A3}'), // \bomb
    ('A', '\u{2619}'),  // \aldineleft
    ('B', '\u{2767}'),  // \aldineright
    ('C', '\u{2766}'),  // \aldine
    ('D', '\u{2766}'),  // \aldinesmall
    ('E', '\u{2740}'), ('F', '\u{2740}'), // \floweroneleft, \floweroneright
    ('G', '\u{1F343}'), ('H', '\u{1F343}'), ('I', '\u{1F343}'), ('J', '\u{1F343}'), // \leafNE, \leafNW, \leafSE, \leafSW
    ('K', '\u{2767}'),  // \leafright
    ('L', '\u{2619}'),  // \leafleft
    ('M', '\u{261C}'),  // \lefthand
    ('N', '\u{261E}'),  // \righthand
    ('O', '\u{2766}'), ('P', '\u{2766}'), ('Q', '\u{2766}'), ('R', '\u{2766}'), // \decofourright…\decothreeleft
    ('S', '\u{2694}'), ('T', '\u{2694}'), // \textxswdown, \textxswup
    ('U', '\u{2756}'),  // \decoone
    ('V', '\u{2727}'),  // \decosix
    ('W', '\u{269C}'),  // \decotwo
    ('X', '\u{2726}'),  // \starredbullet
    ('Y', '\u{2639}'),  // \grimace
    ('Z', '\u{2766}'),  // \textthing
    ('c', '\u{2766}'),  // \aldinesmallup
    ('d', '\u{2766}'),  // \aldinesmallrevert
    ('e', '\u{20AC}'),  // \eurologo
    ('m', '\u{261A}'),  // \leftblackhand
    ('n', '\u{261B}'),  // \rightblackhand
  ];
  let mut table: Vec<Option<char>> = vec![None; 0x80];
  for &(slot, glyph) in slots {
    table[slot as usize] = Some(glyph);
  }
  DeclareFontMap!("futs", Rc::from(table));
});
