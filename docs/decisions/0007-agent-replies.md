# 0007: Asking the agent

Date: 2026-11-21. Status: proposed.

## Context

The agent is a tool of production ([0003](0003-design-philosophy.md)): what it
says has to land as material in the document where the question was asked, not
in a separate chat. This record decides how a question is asked and where and
how the reply appears. What the agent can reach, and what it sees of
handwriting, are left for later records.

## Decision

1. **Questions are handwritten.** Typing on the Type Folio keyboard is a
   stretch goal.

2. **The reply's place is chosen before it exists.** When a question is asked,
   the reader is shown where the reply will appear, right below the question,
   and can move that place. Its width is set by that place; its length is not
   known in advance, so it grows downward.

3. **The reply appears once, when it is done.** It is placed where it was
   promised, on top of everything else, with a transparent background. It is
   not streamed in piece by piece, which suits e-paper: one refresh of one
   region.

4. **The reply is a glyph of rendered HTML.** Its text is selectable text, not
   one image, so it can be highlighted, linked, and quoted like any other
   text. Math and code are rendered properly.

## Consequences

- The reply glyph needs its own renderer and `select`/`mark`
  ([0004](0004-glyph-model.md)), with text kept as text. Laying out the HTML,
  and rendering math and code, can happen on the server, as long as the result
  keeps a text layer.
- Open: what the agent can reach (the selections included with the question,
  the whole library, the web), and what it sees of handwriting.
- Open: what selecting rendered math copies (for example its LaTeX source).
