# Fold: architect answers S1-S6 into FIX doc (fix/bughunt, commit d4a6147)

Source: architect commit 83c0c20. Only `design/FIX_bughunt_mk.md` changed.
No test or production file changed, so cargo was not run and no fmt check
was needed.

- S1 (folded): §8 rows 39, 47, 499, 500 at l.210-213 (row 31 at l.209 unchanged,
  already backticked). §10 mark at l.236.
- S2 (HELD, not folded): no change. The §10 closing text is clear, but the
  brief's "FIX §8 drops the marker sentence" has no exact referent. The only
  marker wording in §8 is l.195 ("and the unreleased marker are STOPs S1 and
  S2"), which would need a wording choice to fix. Coordinator to decide.
- S3 (HELD, not folded): no change. The report says "FIX §3 or §8, a new
  'help' row". §3 has no table, and §8 is the manual table, so the anchor is
  ambiguous. The pointer is at l.89 (§3.2). Coordinator to choose the anchor.
- S4 (folded): §3.3 pointer replaced by the two `///` texts at l.109-119.
  §10 mark at l.259.
- S5 (folded, optional row not added): accept the lowercased payload. §10 mark
  at l.266. The optional §4 row-05 uppercase case is not folded.
- S6 (folded): clause appended to §2 row 5 result cell at l.52, the data-part
  row. The report's "row 5" also matches the HRP row at l.53, but the clause
  concerns the data part, so I used l.52. §10 mark at l.272.

Counts: 6 STOPs; 4 folded (S1, S4, S5, S6); 2 held (S2, S3); 0 test files
changed; 0 Critical, 0 Important.
