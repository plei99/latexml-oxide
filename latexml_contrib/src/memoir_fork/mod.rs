//! Fork-only native bindings for memoir (plei99/latexml-oxide; `fork/MEMOIR_ROADMAP.md`, decision D3).
//!
//! `memoir_cls.rs` loads the real `memoir.cls` raw and then calls [`load_definitions`], so everything here overrides
//! the raw class. Each submodule owns one area of the commands the fork's memoir documents use, defined natively so
//! that its effect reaches the HTML (or is a documented no-op where HTML has no counterpart). Upstream merges conflict
//! on the one call in `memoir_cls.rs` at most.
use latexml_package::prelude::*;

mod chapters;
mod layout;
mod text;

LoadDefinitions!({
  // The fork's presentation (chapter styles, the `\sourceatright` float), required whenever memoir is loaded.
  RequireResource!("ltx-memoir.css");
  layout::load_definitions()?;
  chapters::load_definitions()?;
  text::load_definitions()?;
});
