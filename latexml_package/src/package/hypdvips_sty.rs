//! hypdvips.sty (v3.03, 2019/07/31) — hyperref extensions for the dvips driver.
//!
//! No Perl binding exists; native binding, OXIDIZED_DESIGN #588(h). The package extends hyperref's
//! `hdvips` driver with PostScript `pdfmark`s: file attachments (`\attachfile`, `\embedfile`,
//! `\file`), links into other and attached PDFs (`\goto`, `\gotoparent`, `\runattachment`), named
//! destinations (`\odest`), page labels, open actions, bookmark styles and annotation colours, and it
//! patches the footnote, `\begin`/`\end`, `\abstract` and ToC internals to emit those marks
//! (hypdvips.sty:1980-2306, :3073-3518). It refuses to load without hyperref, or with a driver other
//! than `hdvips` (:26-42). This converter has no output driver: the driver test is taken as passed
//! (a dvips run selects `hdvips` by default, hyperref.sty:4343, which is how the SIAM classes that
//! ship it load it), the hyperref test is kept, and the commands keep what is observable in a
//! document: an attachment is a link to its file showing the `text=` material (or the attachment
//! icon), `\file` its file name, underlined in typewriter (:2874-2884), `\goto[dest=…]{text}` a link
//! to that destination (other targets: the text), `\gotoparent`/`\runattachment` their text,
//! `\odest` an anchor, `\listofattachments` its heading (:3468-3474). The PDF-only settings are read
//! into the macros the package keeps them in. The internal patches are not installed: they only
//! emit `pdfmark`s, except `\end`'s trailing `\ignorespaces` (:1980-1993), which the driver path adds
//! and the converter's `\end` does not have.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // hypdvips.sty:26-42: hyperref first.
  if !lookup_bool("hyperref.sty_loaded") {
    RawTeX!(r"\PackageError{hypdvips}{The hyperref package is not loaded!\MessageBreak Please insert \string\usepackage{hyperref} in your code\MessageBreak before loading hypdvips}\@ehd");
    return Ok(());
  }
  // hypdvips.sty:43-48, :70
  RequirePackage!("atveryend");
  RequirePackage!("xcolor");
  RequirePackage!("xkeyval");
  RequirePackage!("bookmark");
  RawTeX!(r"\@ifpackageloaded{caption}{}{\RequirePackage[all]{hypcap}}
\hypersetup{breaklinks=true}");

  // hypdvips.sty:240-387: the options, booleans of the `[pp]{options}` family (an unknown one is a
  // warning, :352).
  RawTeX!(r"\def\lx@pp@bool#1{\define@boolkey+[pp]{options}[pp@]{#1}[true]{}{\PackageWarning{hypdvips}{Illegal option `#1 = ##1' discarded.\MessageBreak}}}
\@for\lx@pp@name:={autoauthor,autotitle,detailedbr,emptypagelabels,evenboxes,fish,flip,fullbookmarks,hypertnotes,javascript,landscape,loabr,lofbr,lotbr,mac,mirror,nlwarning,openmessage,quadpoints,showdests,smallfootnotes,tocbr,twoside}\do{\expandafter\lx@pp@bool\expandafter{\lx@pp@name}}
\define@key[pp]{options}{JavaScript}[true]{\setkeys[pp]{options}{javascript=#1}}
\define@key[pp]{options}{Mac}[true]{\setkeys[pp]{options}{mac=#1}}
\define@key[pp]{options}{german}[]{}
\define@key[pp]{options}{autocrop}[true]{\PackageWarning{hypdvips}{Obsolete package option `autocrop = #1' discarded.\MessageBreak}}
\define@key[pp]{options}{detailedpagebr}[true]{\PackageWarning{hypdvips}{Obsolete package option `detailedpagebr = #1' discarded.\MessageBreak}}
\DeclareOptionX*{\PackageWarning{hypdvips}{Unknown option `\CurrentOption' discarded.\MessageBreak}}
\ProcessOptionsX*[pp]<options>");

  // The key families of the commands (:2552-2606, :2887-2896, :3260-3264): only `text` (attachfile)
  // and `dest` (goto) change what is typeset.
  RawTeX!(r"\def\lx@pp@nokeys#1#2{\@for\lx@pp@name:=#2\do{\edef\lx@pp@x{\noexpand\define@key[pp]{#1}{\lx@pp@name}[]{}}\lx@pp@x}}
\lx@pp@nokeys{attachfile}{appearance,anncreated,annmodified,author,checksum,color,created,creator,description,date,filetype,flags,icon,iconfilename,mimetype,modified,name,opacity,overprint,print,rect,resourcefork,size,subject,timezone,zoom}
\define@key[pp]{attachfile}{text}{\def\pp@fa@caption{#1}}
\lx@pp@nokeys{embedfile}{checksum,created,creator,description,filetype,mimetype,modified,name,resourcefork,size}
\define@key[pp]{goto}{dest}{\def\pp@go@destination{#1}}
\define@key[pp]{goto}{filename}{\def\pp@go@filename{#1}}
\define@key[pp]{goto}{id}[]{\def\pp@go@id{#1}}
\define@key[pp]{goto}{newwindow}[true]{}
\define@key[pp]{goto}{target}{\def\pp@go@target{#1}}");

  // hypdvips.sty:3734-3753 (`\providecommand`: a definition already made stays).
  RawTeX!(r"\providecommand{\attachfile}[2][]{\begingroup\def\pp@fa@caption{\lx@pp@icon}\setkeys[pp]{attachfile}{#1}\href{#2}{\pp@fa@caption}\endgroup}
\def\lx@pp@icon{📎}
\providecommand{\embedfile}[2][]{\begingroup\setkeys[pp]{embedfile}{#1}\endgroup}
\newcommand{\pp@strippathdef}[2]{\def\lx@pp@rest{#2}\def\lx@pp@last{}\expandafter\lx@pp@strip\detokenize{#2}/\@nil\let#1\lx@pp@last}
\def\lx@pp@strip#1/#2\@nil{\def\lx@pp@last{#1}\ifx\relax#2\relax\else\lx@pp@strip#2\@nil\fi}
\providecommand{\file}[2]{\pp@strippathdef\pp@strippedpath{#1}\href{#1}{\@ifundefined{textcolor}{\underline{\ttfamily\pp@strippedpath}}{\textcolor{\@embeddedcolor}{\ttfamily\underline{\pp@strippedpath}}}}}
\providecommand{\goto}[2][]{\begingroup\let\pp@go@destination\@undefined\let\pp@go@filename\@undefined\let\pp@go@id\@undefined\let\pp@go@target\@undefined
  \setkeys[pp]{goto}{#1}\@ifundefined{pp@go@filename}{\@ifundefined{pp@go@id}{\@ifundefined{pp@go@target}{\@ifundefined{pp@go@destination}{#2}{\hyperlink{\pp@go@destination}{#2}}}{#2}}{#2}}{#2}\endgroup}
\providecommand{\gotoparent}[2]{#2}
\providecommand{\runattachment}[2]{#2}
\providecommand{\odest}[2]{\hypertarget{#1}{}}
\providecommand{\loaformat}[4]{{\ttfamily\pp@strippathdef\pp@string{#3}\pp@string}}
\def\pp@loastring{List of File Attachments}
\newcommand{\listofattachments}{\section*{\pp@loastring}}
\providecommand{\pagelabel}[2][]{}
\providecommand{\openaction}[1]{\def\pp@openaction{#1}}
\providecommand{\bmstyle}[2]{\bookmarkdefinestyle{#1}{#2}}
\providecommand{\backrefbordercolor}[1]{\gdef\@backrefbordercolor{#1}}
\providecommand{\backrefcolor}[1]{\gdef\@backrefcolor{#1}}
\providecommand{\embeddedbordercolor}[1]{\gdef\@embeddedbordercolor{#1}}
\providecommand{\embeddedcolor}[1]{\gdef\@embeddedcolor{#1}}
\providecommand{\footnotebordercolor}[1]{\gdef\@footnotebordercolor{#1}}
\providecommand{\footnotecolor}[1]{\gdef\@footnotecolor{#1}}
\providecommand{\tablenotebordercolor}[1]{\gdef\@tablenotebordercolor{#1}}
\providecommand{\tablenotecolor}[1]{\gdef\@tablenotecolor{#1}}
\providecommand{\evenboxesstring}[1]{\gdef\pp@ABC{#1}}
\providecommand{\point}{}
\providecommand{\currentpoint}[1]{}
\providecommand{\debug}[1]{\PackageWarning{DEBUG}{#1}}
\edef\@embeddedcolor{\@runcolor}");
});
