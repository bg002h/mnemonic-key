//! Placeholder integration test for `mk-codec`.
//!
//! Skeleton only — gated behind `#[ignore]` since `encode`/`decode`
//! aren't implemented yet. Will become the canonical round-trip test
//! once the implementation lands.

#[test]
#[ignore = "mk-codec encode/decode not yet implemented"]
fn round_trip_single_xpub_one_policy_id_stub() {
    // Future shape:
    //   1. Build a KeyCard with one Policy ID stub, a known xpub,
    //      and a standard origin path.
    //   2. encode → Vec<String>
    //   3. decode → KeyCard
    //   4. Assert byte-identical round-trip.
    //
    // Blocked on: encode + decode implementation. See
    // design/SPEC_mk_v0_1.md for the wire-format spec.
}
