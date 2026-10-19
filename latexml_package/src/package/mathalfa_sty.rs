//! mathalfa.sty — the old name of mathalpha: `\RequirePackageWithOptions{mathalpha}`
//! (mathalfa.sty:1-2). Native binding, OXIDIZED_DESIGN #588(j); see mathalpha_sty.rs.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  require_package_with_options("mathalpha")?;
});
