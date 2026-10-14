# Beamer → browser slide decks: options (parked 2026-10-14)

Status: brainstorming, no approach chosen yet. Come back here before writing a spec.

## What we know

- **Today.** `beamer_cls.rs` turns each frame into an `ltx:subsection` shown once, in its final state:
  - `\pause` and `\onslide` are no-ops;
  - `\uncover` and `\visible` pass their body through, and `\invisible` drops it;
  - `\alt` always takes its first branch;
  - `\only` filters by mode spec only;
  - `\frametitle` is a bold paragraph and `\framesubtitle` is dropped;
  - themes are ignored, apart from the `\setbeamercolor` colour model.
- **Perl LaTeXML.** `beamer.cls.ltxml` makes each frame an `ltx:slidesequence` with one `ltx:slide` per overlay step, digesting the body again for each step. Covered or invisible content is an inline block with the class `ltx_covered`, `ltx_invisible` or `ltx_alert`. Neither Perl's stock XSLT/CSS nor ours styles slides, so a frame with n pauses comes out as n stacked copies.
- **Typical use** (the decks in the notes corpus, about 20 frames each):
  - overlays: `\pause`, `\onslide<…>`, a few `\only` and `itemize[<+->]`, and one overlay inside a TikZ picture;
  - content: TikZ and tikz-cd pictures, `fragile` frames;
  - themes: metropolis and Darmstadt.

## Decided so far

- **Purpose:** presenting live from the browser, with keyboard stepping, a presenter view and a fixed 16:9 or 4:3 frame. A no-JS fallback to a readable page is secondary.
- **Look:** one clean house style for every deck. `\usetheme` is ignored; `\setbeamercolor` colours may tint it.
- **Divergence:** a hard fork is fine. Nothing has to stay upstreamable or Perl-faithful.
- **Self-contained binary:** any JS or CSS is embedded, as the rest of the resources are.

## Approaches

Every approach starts the same way: the beamer binding stops flattening and emits a slide model. Each frame becomes a slide with its title, and each overlay-bearing element carries the set of steps it shows on, without digesting the body again. The approaches differ in what renders that model.

### A. Our own small stepper (recommended so far)

A new `LaTeXML-slides` XSLT emits `<section class="slide">` elements with `data-step` sets on elements. Embedded CSS and about 200 lines of JS handle:
- fixed-aspect scaling;
- arrow keys and click to step through overlays, then frames;
- `#frame/step` URLs;
- a presenter window synced with the main one, showing the current and next slide and a timer;
- printing one page per frame.

- **Covered content** (`\uncover`, `\onslide`) keeps its space and is hidden. `\only` content takes no space when off.
- **Pros:** maps exactly onto beamer's overlay sets, including non-monotonic ones like `\only<2>` and `\onslide<1-3>`; no dependency; small.
- **Cons:** the presenter view and print layout are ours to build.
- **TikZ overlays:** the pgf/SVG driver tags the wrapped drawing (`\onslide`/`\only` inside a picture, `visible on=<…>`) with its step set.

### B. Vendored reveal.js

The same slide model, emitted as reveal.js markup, with beamer steps as reveal "fragments".

- **Pros:** presenter view, speaker notes, print-to-PDF, plugins and transitions for free.
- **Cons:**
  - fragments model "appear, in order". `\only<2>`, `\onslide<1-3>` (content that disappears again) and `\alt` need custom fragment classes and fight its model;
  - about 1 MB embedded;
  - reveal's CSS clashes with the `ltx_*` styles.

### C. Re-digest per overlay (Perl's model)

One complete HTML slide per overlay step, so stepping is paging.

- **Pros:** simplest JS. Exactly right for anything, TikZ overlays included, since TeX produces each state.
- **Cons:** multiplies slides and digestion time (a deck with 20 pauses doubles); no per-element transitions; a large output.

## Open questions for when we resume

- Speaker notes: `\note` isn't used in the corpus decks; skip it, or support it for the presenter view?
- Output shape: one self-contained HTML file per deck (SVGs inlined), or the usual directory?
- How to select it: automatically for `\documentclass{beamer}`, or a flag (`--format=slides`)?
- Frame features to cover first: blocks, columns, theorem environments, `\titlepage`, section divider slides, `allowframebreaks`.
