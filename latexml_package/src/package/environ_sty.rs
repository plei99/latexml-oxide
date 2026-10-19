//! environ.sty (v0.3, 2014/05/04) — environments whose body is collected into a macro.
//!
//! No Perl binding exists; native binding, OXIDIZED_DESIGN #588(l). `\NewEnviron{name}[n][default]{begin}[end]`
//! (environ.sty:143-219) defines an environment whose begin code collects the body up to its own
//! `\end{name}` (`\Collect@Body`, :98-131), stores it, spaces trimmed, in `\BODY` (the name
//! `\environbodyname` set when the environment was defined, :52-53), then runs `begin` with the
//! environment's arguments, read from the start of the collected body (`\env@<name>@save@env`,
//! `\env@<name>@process`, :197-213); `end` (default `\environfinalcode`, `\ignorespacesafterend`)
//! closes it. The collection is native here (`\lx@environ@collect`): it reads the tokens up to the
//! `\end` that matches, counting the `\begin`s and `\end`s outside braces as `\Collect@@Body` and
//! `\Push@Begins` do (:110-131), and hands the body on as one argument, then `\end{<name found>}`;
//! a mismatched name reaches the kernel's `\end`, which reports it as `\@checkend` does. The rest is
//! the package's own definitions.
//! Witnesses (errors in the default configuration, before → after): 2610.15762 (2 → 0), 2610.18953 (16 → 0).
use crate::prelude::*;

/// Read a body up to the `\end` that closes it: `\begin`/`\end` counted at brace depth 0, as
/// environ's delimited `#1\end#2` and `\Push@Begins#1\begin` read them. Returns the body and the
/// name argument of the closing `\end` (as tokens, braces stripped), or `None` at the end of input.
fn collect_body() -> Result<(Vec<Token>, Option<Vec<Token>>)> {
  let begin = T_CS!("\\begin");
  let end = T_CS!("\\end");
  let mut body = Vec::new();
  let mut depth = 0usize;
  let mut nested = 0usize;
  while let Some(token) = read_token()? {
    match token.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ => {},
    }
    if depth == 0 && token == begin {
      nested += 1;
    } else if depth == 0 && token == end {
      if nested == 0 {
        let name = read_arg(ExpansionLevel::Off)?;
        return Ok((body, Some(name.unlist())));
      }
      nested -= 1;
    }
    body.push(token);
  }
  Ok((body, None))
}

#[rustfmt::skip]
LoadDefinitions!({
  // environ.sty:51
  InputDefinitions!("trimspaces", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  // `\lx@environ@collect{name}{callback}`: `callback{body}\end{name}` (environ.sty:98-131).
  DefMacro!("\\lx@environ@collect{}{}", sub[(name, callback)] {
    let (body, found) = collect_body()?;
    let mut out = callback.unlist();
    out.push(T_BEGIN!());
    out.extend(body);
    out.push(T_END!());
    match found {
      Some(found) => {
        out.push(T_CS!("\\end"));
        out.push(T_BEGIN!());
        out.extend(found);
        out.push(T_END!());
      },
      None => {
        Error!("expected", "\\end", format!("Missing \\end{{{}}}: the environment's body runs to the end of the input", name.untex()));
      },
    }
    Ok(Tokens::new(out))
  });

  // environ.sty:52-57
  RawTeX!(r"\def\environbodyname#1{\def\env@BODY{#1}}
\environbodyname\BODY
\def\environfinalcode#1{\def\env@finalcode{#1}}
\environfinalcode{\ignorespacesafterend}
\def\longdef@c#1{\expandafter\long\expandafter\def\csname#1\endcsname}");
  // environ.sty:98-131: `\Collect@Body`, and amsmath's `\collect@body` when it is undefined
  // (:60-97), both collecting into `\@envbody` for the callback.
  RawTeX!(r"\unless\ifdefined\@envbody\newtoks\@envbody\fi
\long\def\Collect@Body#1{\expandafter\lx@environ@collect\expandafter{\@currenvir}{\lx@environ@callback#1}}
\long\def\lx@environ@callback#1#2{\global\@envbody{#2}#1{#2}}
\unless\ifdefined\collect@body\let\collect@body\Collect@Body\fi
\long\def\Addto@Envbody#1{\global\@envbody\expandafter{\the\@envbody#1}}");
  // environ.sty:143-219, as the package defines them.
  RawTeX!(r"\def\NewEnviron{\let\env@newenvironment\newenvironment\env@NewEnviron}
\def\RenewEnviron{\let\env@newenvironment\renewenvironment\env@NewEnviron}
\def\env@NewEnviron#1{\@ifnextchar[{\env@new@i{#1}}{\env@new@iii{#1}{}}}
\def\env@new@i#1[#2]{\@ifnextchar[{\env@new@ii{#1}[#2]}{\env@new@iii{#1}{[#2]}}}
\def\env@new@ii#1[#2][#3]{\env@new@iii{#1}{[#2][#3]}}
\long\def\env@new@iii#1#2#3{\@temptokena={\env@new{#1}{#2}{#3}}\@ifnextchar[{\the\@temptokena}{\expandafter\the\expandafter\@temptokena\expandafter[\env@finalcode]}}
\long\def\env@new#1#2#3[#4]{%
  \longdef@c{env@#1@BODY\expandafter}\expandafter{\env@BODY}%
  \env@newenvironment{#1}{\expandafter\Collect@Body\csname env@#1@parse\endcsname}{#4}%
  \longdef@c{env@#1@parse}##1{\csname env@#1@save@env\endcsname##1\env@nil\csname env@#1@process\endcsname##1\env@nil}%
  \expandafter\let\csname env@#1@save@env\endcsname\relax
  \expandafter\let\csname env@#1@process\endcsname\relax
  \expandafter\newcommand\csname env@#1@save@env\endcsname#2{\expandafter\expandafter\expandafter\env@save\csname env@#1@BODY\endcsname}%
  \expandafter\newcommand\csname env@#1@process\endcsname#2{#3\env@ignore}}
\long\def\env@save#1#2\env@nil{\edef#1{\unexpanded\expandafter\expandafter\expandafter{\trim@spaces{#2}}}}
\long\def\env@ignore#1\env@nil{}");
});
