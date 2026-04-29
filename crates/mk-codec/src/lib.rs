//! `mk-codec` — reference implementation of the **Mnemonic Key (MK)** backup format.
//!
//! Status: **design-stage skeleton, no implementation yet.** All public
//! functions return `todo!()` or panic. The crate exists so the design
//! surface is concretely visible alongside the spec and BIP draft, and
//! so the repo structure is in place when implementation work begins.
//!
//! See for the design surface:
//!
//! - `design/SPEC_mk_v0_1.md` — wire-format spec
//! - `design/DECISIONS.md` — rolling decisions log
//! - `bip/bip-mnemonic-key.mediawiki` — BIP draft skeleton
//!
//! See for related sibling project:
//!
//! - [`bg002h/descriptor-mnemonic`](https://github.com/bg002h/descriptor-mnemonic) — the MD policy-template format and its `md-codec` reference implementation. MK is designed to engrave alongside MD policy cards for foreign-xpub multisig recovery.
//!
//! # Eventual factoring
//!
//! Per `design/DECISIONS.md` D-13, this crate initially **forks** the
//! BCH primitives from the sibling `md-codec` once those modules are
//! needed; the shared codex32-derived plumbing extracts to a third
//! crate (likely a new sibling repo `mc-codex32`) once both formats
//! are implementation-validated. For now the scaffold doesn't have
//! those modules at all — they land when there's actual encoding
//! logic to put in them.

#![cfg_attr(not(test), deny(missing_docs))]

pub mod error;
pub mod key_card;

pub use error::Error;
pub use key_card::{KeyCard, decode, encode};
