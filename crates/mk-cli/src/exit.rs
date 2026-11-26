//! The constellation exit codes (`SPEC_constellation_exit_codes.md` §3, in
//! mnemonic-engrave's `design/`).
//!
//! **A local copy.** This mirrors the `exit` constants and bit-set of
//! `mnemonic-io-lib` until that crate is published with them (spec decision
//! 4); `numbers_match_the_spec` pins every number to §3 so the two cannot
//! drift silently. Classification stays here, in `error.rs`: the shared crate
//! holds the numbers, each binary decides which class an outcome is.
//!
//! Codes 1 to 63 are sums of the six bits below, each sum unique. 64 and 101
//! never combine.

/// Everything requested was done and every check passed.
pub const OK: u8 = 0;
/// The input is well-formed and a comparison against something else came out
/// no (a verify mismatch, not a member, an expectation not met).
pub const NO: u8 = 1;
/// An input is wrong for this verb whatever the flags (bad checksum, wrong
/// HRP or kind, a newer format, empty input, a malformed value the tool reads).
pub const INVALID: u8 = 2;
/// The tool checked part of the result and could not confirm it (a correction
/// with no oracle).
pub const UNVERIFIED: u8 = 4;
/// The tool declined under a safety or policy rule that a flag, network,
/// channel or different destination would waive, or a network mismatch.
pub const REFUSED: u8 = 8;
/// A checksum correction was applied to an input and the corrected value used.
pub const REPAIRED: u8 = 16;
/// Something needed from the system failed or is missing (an input file, an
/// output that cannot be written, stdout).
pub const IO: u8 = 32;
/// Never combines: the command line alone is wrong; nothing ran.
pub const USAGE: u8 = 64;
/// Never combines: a bug, or a failure no class covers (Rust's panic code).
pub const INTERNAL: u8 = 101;
/// The operator stopped it (at a prompt, or Ctrl-C = 128 + SIGINT).
pub const STOPPED: u8 = 130;

/// The root `--help` legend (spec R11), word for word.
pub const HELP_LEGEND: &str = "Exit status: 0 = done, every check passed. 64 = bad command line, \
nothing ran. 101 = internal error. 130 = stopped by the operator. Other codes from 128 up = killed \
by a signal. Otherwise the sum (1 to 63) of: 1 a check said no; 2 an input is invalid; 4 verify the \
result before trusting it; 8 refused (network mismatch, secret on the command line, unsafe output); \
16 a correction was applied; 32 a file, device or helper failed.";

/// A set of the six combinable bits (1..=32).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Bits(u8);

impl Bits {
    /// The empty set (exit 0).
    pub const fn empty() -> Self {
        Bits(0)
    }

    /// The set holding one bit (or a sum of bits).
    pub const fn of(bits: u8) -> Self {
        Bits(bits & 63)
    }

    /// Whether every bit of `bits` is set.
    pub const fn contains(self, bits: u8) -> bool {
        self.0 & bits == bits
    }

    /// The exit status: the sum of the bits.
    pub const fn to_exit_code(self) -> u8 {
        self.0
    }
}

impl std::ops::BitOr<u8> for Bits {
    type Output = Bits;
    fn bitor(self, rhs: u8) -> Bits {
        Bits(self.0 | (rhs & 63))
    }
}

impl std::ops::BitOr for Bits {
    type Output = Bits;
    fn bitor(self, rhs: Bits) -> Bits {
        Bits(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign<u8> for Bits {
    fn bitor_assign(&mut self, rhs: u8) {
        self.0 |= rhs & 63;
    }
}

/// Add `bits` to an exit status, unless the status is one that never
/// combines (0 combines; 64, 101 and anything above 63 are kept as they are).
pub const fn add(code: u8, bits: u8) -> u8 {
    if code > 63 { code } else { code | (bits & 63) }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Spec §3: the numbers, pinned. A change here is a change to the
    /// constellation contract and to mnemonic-io-lib's copy.
    #[test]
    fn numbers_match_the_spec() {
        assert_eq!(
            [
                OK, NO, INVALID, UNVERIFIED, REFUSED, REPAIRED, IO, USAGE, INTERNAL, STOPPED
            ],
            [0, 1, 2, 4, 8, 16, 32, 64, 101, 130]
        );
    }

    #[test]
    fn bits_sum_and_contain() {
        let b = Bits::empty() | REPAIRED | UNVERIFIED;
        assert_eq!(b.to_exit_code(), 20);
        assert!(b.contains(REPAIRED) && b.contains(UNVERIFIED) && !b.contains(INVALID));
        assert_eq!((b | Bits::of(INVALID)).to_exit_code(), 22);
        assert_eq!(Bits::empty().to_exit_code(), OK);
    }

    #[test]
    fn usage_and_internal_never_combine() {
        assert_eq!(add(USAGE, IO), USAGE);
        assert_eq!(add(INTERNAL, IO), INTERNAL);
        assert_eq!(add(OK, IO), 32);
        assert_eq!(add(REPAIRED, IO), 48);
    }

    #[test]
    fn the_legend_is_the_specs() {
        assert!(HELP_LEGEND.starts_with("Exit status: 0 = done, every check passed."));
        assert!(HELP_LEGEND.ends_with("32 a file, device or helper failed."));
    }
}
