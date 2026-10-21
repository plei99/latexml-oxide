use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: lscape.sty.ltxml
  // Conceivably could be useful to add "pagination markers"
  // (empty elements that are normally ignored) at beginning and end
  DefEnvironment!("{landscape}", "#body",
    // The begin code opens a `\begingroup` (a semi_simple_group) of its own: lscape.sty:38-42 `\begingroup`.
    tex_group => { TexGroup::SemiSimple },
    // lscape.sty's end code runs `\clearpage`, which ends the paragraph the body left open (the mode-less
    // environment's body is `restricted_horizontal`, where nothing else ends it).
    before_digest_end => {
      // not when a plain `\endgroup` closed the environment's group: the end code never ran
      if environment_ended_by_group() { vec![] } else { vec![digest(Tokens!(T_CS!("\\par")))?] }
    });
});
