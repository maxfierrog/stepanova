# 0005: Links are portals

Date: 2026-11-21. Status: proposed.

## Context

The core of Stepanova is the selection: a part of document A, shown in document
B, that leads back to A. The [glyph model](0004-glyph-model.md) defines how a
link is addressed (a path ending in a selector) and how a glyph marks a
selection. This record decides what a link looks like at each end, and what
happens to it when its source changes or disappears.

## Decision

1. **A link is a portal with two ends.** The destination shows the selected
   content with a **backlink** indicator. The source region carries a
   **forward-link** indicator (drawn by the source glyph's `mark`). Clicking
   either indicator goes through the portal to the other end.

2. **What the destination shows depends on how the selection was made:**
   - a **highlight** is text, so it is shown as a quotation with a citation of
     its source. It stays text: copyable, searchable, and readable by the agent;
   - a **region selection** (rectangle or lasso) is shown as an image of what
     was there.

3. **The content is a snapshot.** It is captured when the selection is made and
   does not change when the source is edited later; the note keeps what was
   seen. The link still follows its path to the source, resolving to the
   deepest glyph it can reach.

4. **A changed source is marked.** When the source glyph changes after the
   snapshot was taken, every backlink to it gains a "source changed" marker,
   even if the change is not visible in the snapshot.

5. **Deleting the source never deletes material.** If the source glyph or any
   of its ancestors is deleted, every backlink to it keeps its snapshot and the
   annotations attached to it, gains a tombstone marker, and cannot be
   followed until it is reviewed. Because content is a snapshot, deletion does not cascade through
   chains of links.

6. **Changes and tombstones are reviewed the same way.** The reader either
   **re-does** the selection (on the changed source, or anywhere else when the
   source is gone), or **confirms** the existing snapshot as it is. Either one
   clears the marker. Every marked backlink in the library is listed in one
   central review place, so links can be kept consistent without hunting for
   them.

7. **One source, many destinations.** When a region has been used in several
   places, clicking its forward-link indicator shows a short list of the
   destinations, each named by its document.

8. **Indicators can be hidden.** A single view toggle shows or hides every
   backlink and forward-link indicator, so content can be read cleanly. While
   visible, indicators are clickable.

## Consequences

- Storage keeps links in both directions, so a source knows its forward links
  and a change or deletion can find every backlink to mark.
- Each snapshot records the version of the source glyph it was taken from,
  so a change is detected by comparing versions rather than images.
- Indicators are small, local marks, so toggling them is cheap on e-paper
  ([0003](0003-design-philosophy.md), principle 8).
- The look of quotations, citations, indicators, and tombstones is left to the
  visual language records.
