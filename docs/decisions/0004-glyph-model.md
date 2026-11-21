# 0004: Everything is a glyph

Date: 2026-11-21. Status: proposed.

## Context

Stepanova produces material: ink, text, selections cut from sources, and the
links between them ([0003](0003-design-philosophy.md)). That material has to
be reproducible on top of media it does not own (PDFs, books, images), stored
apart from that media, addressable from anywhere, and cheap to keep in memory
on a small device. Render-oriented software (vector editors, browsers, game
engines) solves the rendering half of this with a retained tree of drawable
nodes; Stepanova extends that tree to addressing and caching.

## Decision

1. **Everything on screen is a glyph.** A glyph is a drawable node with
   metadata that lets it be reproduced exactly. Ink, text, images, and pages of
   media are all glyphs, layered over and under one another. A media glyph is
   opaque to rendering but keeps its semantics: copyable text, an outline,
   internal links.

2. **A document is a root glyph.** Every document has exactly one root, and the
   root's kind decides how the document is viewed:
   - **Canvas:** a transparent, infinite grid with 64-bit integer
     coordinates. A new note is an empty canvas, which renders as white.
   - **PDF** and **EPUB:** paginated roots. They drive the viewport (turn the
     page, jump from the outline, search), and other glyphs anchor to their
     pages.

   PDFs and EPUBs are always roots, never children. Bringing part of one into
   another document is done through a selection, not by placing the file.

   An EPUB accepts highlights and selections but no handwriting. Ink is
   geometric and an EPUB has no fixed geometry to anchor it to. Writing about a
   book happens in a note, which links back to the book through a selection.

3. **Pagination is a function.** A paginated root is its content typeset with
   stored layout parameters, so its pages are deterministic and reproducible.
   A PDF is already typeset. An EPUB is structured text with no inherent pages
   (a zip of XHTML chapters, CSS, metadata, and navigation), so its layout
   parameters are part of the document's state.

4. **A link is a path.** It is a sequence of segments resolved one glyph at a
   time, from the root down:
   - every segment before the last names a child glyph **by identity**, so
     moving or editing a glyph never breaks links that pass through it;
   - the last segment is a **selector**, interpreted only by the leaf glyph.

   The path is the selection's provenance: the document, then the location in
   it, recursively. Prompts carry it to the agent as is. If a path stops
   resolving partway, it resolves to the deepest glyph it reached.

5. **Each glyph type implements a small contract:**
   - resolve a child by identity;
   - a transform into its parent's coordinates;
   - `select(selector)`, which returns the region the selection occupies. The
     viewport centers on that region and zooms to fit it proportionally;
   - `mark(selector)`, which draws an indicator for the selection that suits
     how it was made (an outline for a lasso, highlighting for a text range);
   - a distance, either its own or inherited from its parent (see 8).

   A selection that spans several children belongs to their nearest common
   ancestor. For example, a text range across two pages is a selector on the
   paginated root.

6. **The tree is attachment, and layering is sibling order.** A glyph's parent
   is what it moves with and is anchored to; that changes only when a glyph is
   deliberately moved onto another one. Attachment can nest to any depth: ink
   written over an image on a canvas moves with the image, and a group moves
   its members as one. Z-order is the order of a parent's
   children, so layering never changes a link path:
   - **up one and down one** step past the next sibling above or below that
     overlaps the glyph, skipping siblings that do not;
   - **front and back** move it to the end or the start of its siblings.

   Each parent keeps a spatial index of its children so overlap queries stay
   cheap however many siblings there are.

7. **Storage is separate from media.** Glyph data has a stable file format,
   stored apart from the immutable media it annotates. Every bit of markup
   state can be reconstructed from that file plus the media.

8. **Caching follows distance.** Each glyph defines what "near" means inside
   itself or inherits that from its parent: Euclidean distance on a canvas, and
   page distance weighted by reading direction in a paginated root. Following
   a link jumps to its target in one tap, so the targets of visible links are
   near too. Cached layers (decoded media, rasterized pixels, semantic data)
   are kept and evicted by distance, weighed against what each costs to
   rebuild. This follows the practice of browser tile managers and game asset
   streaming.

## Consequences

- Anchoring over a paginated root is geometric, against a specific
  typesetting. Selectors over text also carry content anchors (like EPUB's
  Canonical Fragment Identifiers) so that they survive typesetting again.
- The unit of every coordinate is TeX's scaled point (1 sp = 1/65536 pt,
  about 5.4 nm). It is finer than any input can reach: zooming in about 3000
  times at the pen digitizer's precision (about 16 µm) still does not reach
  it. Its 64-bit range spans about 50 million km. It maps exactly onto points,
  the unit of PDFs and typography, and it does not depend on the device, so the
  same document measures the same on any display. Integers keep coordinates
  exact and reproducible, and pixels only appear when a glyph is rendered.
- Marks drawn during navigation may use quality greys. Marks drawn live while
  selecting must stay thin, 1-bit, and local ([0003](0003-design-philosophy.md),
  principle 8).
- The file format, the selector kinds, and the cache budget each get their own
  record once there is evidence to decide them.
