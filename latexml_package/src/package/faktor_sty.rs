//! faktor.sty (v0.1b, 2006/04/05) — quotient structures `G/H` with a raised numerator and a
//! lowered denominator.
//!
//! No Perl binding exists; native binding, OXIDIZED_DESIGN #588(e). `\faktor[#1]{#2}{#3}`
//! (faktor.sty:33-37) typesets, in the current math style, `#1{#2}` raised by half its height,
//! a `\diagup` slash kerned in by `\mkern-5mu`/`\mkern-4mu`, and `#1{#3}` lowered by half its
//! height (`\faktor@impl`, :39-48). Here it is one application of a quotient operator (U+2571, the
//! `\diagup` glyph) to the two parts, the parts offset as the package moves them, as nicefrac's
//! inline form is built (nicefrac_sty.rs). `#1` is a prefix to each part (`\faktor[\mathbb]{Z}{n}`),
//! so it is applied before the parts are digested.
//! Witnesses (errors in the default configuration, before → after): 2610.20478 (1 → 0), 2610.21953 (1 → 0), 2610.10804.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // faktor.sty:33 `\DeclareRobustCommand*{\faktor}[3][]`. The parts are passed twice: as
  // written, for the reversion, and with the prefix applied, to typeset.
  DefMacro!("\\faktor[]{}{}", "\\lx@faktor{#1}{#2}{#3}{#1{#2}}{#1{#3}}", robust => true);
  // faktor.sty:43-47: the numerator is raised and the denominator lowered by half their heights
  // (about 0.35em for a capital letter); the slash sits between them.
  DefConstructor!("\\lx@faktor Undigested Undigested Undigested {}{}",
    "<ltx:XMApp>\
       <ltx:XMTok role='MULOP' meaning='quotient' name='faktor'>\u{2571}</ltx:XMTok>\
       <ltx:XMArg yoffset='0.35em'>#4</ltx:XMArg>\
       <ltx:XMArg yoffset='-0.35em'>#5</ltx:XMArg>\
     </ltx:XMApp>",
    reversion => sub[_whatsit, args] {
      // `\faktor[#1]{#2}{#3}` as written, the bracket only when the prefix was given.
      let mut tks = vec![T_CS!("\\faktor")];
      if let Some(prefix) = &args[0] {
        let prefix = prefix.revert()?;
        if !prefix.is_empty() {
          tks.push(T_OTHER!("["));
          tks.extend(prefix.unlist());
          tks.push(T_OTHER!("]"));
        }
      }
      for part in &args[1..3] {
        tks.push(T_BEGIN!());
        if let Some(part) = part {
          tks.extend(part.revert()?.unlist());
        }
        tks.push(T_END!());
      }
      Ok(Tokens::new(tks))
    },
    require_math => true,
    after_construct => sub[document] {
      // The slash is a symbol, upright in the current font (`\diagup` is an amssymb `\mathrel`
      // symbol), as nicefrac's slash is set (nicefrac_sty.rs).
      let node = document.get_node().clone();
      for mut tok in document.findnodes("descendant::ltx:XMTok[@name='faktor']", Some(&node)) {
        let slash_font = document.get_node_font(&tok).specialize("\u{2571}");
        document.set_node_font(&mut tok, &slash_font)?;
      }
    });
});
