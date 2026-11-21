# 0006: The device is a cache of the server

Date: 2026-11-21. Status: proposed.

## Context

The device has limited storage, and a library can be larger than it. Documents
therefore have to leave the device and come back, and they must come back
exactly as they left: glyph data is reproducible by design
([0004](0004-glyph-model.md)), and portals detect changes to their sources by
version ([0005](0005-portals.md)). New documents arrive from both sides: media
imported into the library, and notes written on the device, possibly offline.

## Decision

1. **The server is the source of truth.** The device is a cache of it. Editing
   offline is allowed, but a document is edited from one device at a time.

2. **Each document has three tiers of data:**
   - **metadata:** id, title, place in the library, the link index in both
     directions, versions, and outline. It is small and never evicted, so an
     evicted document still appears in the library, links into it still
     resolve to the document, and the review list stays complete;
   - **content:** the original media bytes and the glyph data (ink, selections,
     snapshots, layout parameters). Evicting it leaves a stub that is fetched
     again when opened;
   - **derived:** rendered tiles, decoded pages, extracted text, and search
     indexes. They are rebuilt on demand and may be dropped at any time.

3. **Content is addressed by hash.** Every media file and every version of a
   document's glyph data is an immutable object, named by the hash of its
   bytes. A version is a hash, which is also what a snapshot records about its
   source.

4. **Only acknowledged objects are evicted.** An object may leave the device
   only after the server has confirmed it by hash. Unsynced edits are pinned
   until then, so being offline limits what can be evicted but never loses
   work. A fetched object is verified against its hash, which proves the
   round trip.

5. **The encoding is canonical.** The file format has exactly one encoding for
   each state, so the same glyphs always produce the same bytes and the same
   hash. Decoding then encoding, and encoding then decoding, are both the
   identity, and this is tested.

6. **Media is never converted.** An imported PDF or EPUB is kept as its
   original bytes. Its text, outline, page geometry, and search index are
   derived data: deterministic functions of the media's hash and the
   extractor's version. Either side can compute them, so a new document needs
   no preparation by the server before the device can cache it. The server
   computes them anyway, since the agent needs the text for retrieval.

7. **Notes are born on the device.** A new note is an empty canvas root, pinned
   on the device until the server acknowledges it.

## Consequences

- Two devices editing the same document concurrently are out of scope. Because
  each version names its parent, the server can detect it and refuse the
  second one rather than lose either; merging is not designed.
- The file format, the hash function, and the eviction policy are left to
  their own records. Eviction is expected to follow the cache distance from
  [0004](0004-glyph-model.md).
