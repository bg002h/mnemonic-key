//! Integration tests for `mk gui-schema`.
//!
//! Realizes Section C.2 of the `mnemonic-gui` v0.2 plan: validates the SPEC §7
//! JSON contract that `mnemonic-gui`'s schema-mirror gate consumes.

use std::process::Command;

use assert_cmd::cargo::CommandCargoExt;
use serde_json::Value;

/// `mk gui-schema` exits 0 and produces stdout that parses as JSON.
#[test]
fn gui_schema_exits_zero_and_emits_json() {
    let mut cmd = Command::cargo_bin("mk").expect("mk binary");
    let out = cmd
        .arg("gui-schema")
        .output()
        .expect("invoke mk gui-schema");
    assert!(out.status.success(), "mk gui-schema failed: {:?}", out);
    let v: Value = serde_json::from_slice(&out.stdout).expect("stdout parses as JSON");
    assert!(v.is_object(), "top-level must be a JSON object");
}

/// `version == 1` and `cli == "mk"` (SPEC §7 invariants).
#[test]
fn gui_schema_envelope_pins_version_and_cli() {
    let mut cmd = Command::cargo_bin("mk").expect("mk binary");
    let out = cmd
        .arg("gui-schema")
        .output()
        .expect("invoke mk gui-schema");
    assert!(out.status.success());
    let v: Value = serde_json::from_slice(&out.stdout).expect("JSON");
    assert_eq!(v["version"], Value::from(1u32), "version must be 1");
    assert_eq!(v["cli"], Value::from("mk"), "cli must be 'mk'");
    assert!(
        v["subcommands"].is_array(),
        "subcommands must be a JSON array"
    );
}

/// `encode` exposes `--xpub` and `--origin-path` as required text flags.
#[test]
fn gui_schema_encode_xpub_and_origin_path_required() {
    let mut cmd = Command::cargo_bin("mk").expect("mk binary");
    let out = cmd
        .arg("gui-schema")
        .output()
        .expect("invoke mk gui-schema");
    assert!(out.status.success());
    let v: Value = serde_json::from_slice(&out.stdout).expect("JSON");
    let subs = v["subcommands"].as_array().expect("subcommands array");
    let encode = subs
        .iter()
        .find(|s| s["name"] == "encode")
        .expect("encode subcommand present");

    let flags = encode["flags"].as_array().expect("encode flags");
    let xpub = flags
        .iter()
        .find(|f| f["name"] == "--xpub")
        .expect("--xpub flag present");
    assert_eq!(
        xpub["required"],
        Value::from(true),
        "--xpub must be required"
    );
    assert_eq!(xpub["kind"], Value::from("text"), "--xpub kind=text");

    let origin_path = flags
        .iter()
        .find(|f| f["name"] == "--origin-path")
        .expect("--origin-path flag present");
    assert_eq!(
        origin_path["required"],
        Value::from(true),
        "--origin-path must be required"
    );
    assert_eq!(
        origin_path["kind"],
        Value::from("text"),
        "--origin-path kind=text"
    );
}

/// `gui-schema` does NOT list itself or `help` (avoid GUI exposing reflection
/// subcommand or auto-generated help).
#[test]
fn gui_schema_excludes_self_and_help() {
    let mut cmd = Command::cargo_bin("mk").expect("mk binary");
    let out = cmd
        .arg("gui-schema")
        .output()
        .expect("invoke mk gui-schema");
    assert!(out.status.success());
    let v: Value = serde_json::from_slice(&out.stdout).expect("JSON");
    let subs = v["subcommands"].as_array().expect("subcommands array");
    let names: Vec<&str> = subs
        .iter()
        .map(|s| s["name"].as_str().expect("name string"))
        .collect();
    assert!(
        !names.contains(&"gui-schema"),
        "gui-schema must not list itself; got {names:?}"
    );
    assert!(
        !names.contains(&"help"),
        "help must not appear; got {names:?}"
    );
}

/// Every v0.1 subcommand surfaces (encode/decode/inspect/verify/vectors).
#[test]
fn gui_schema_lists_all_five_v0_1_subcommands() {
    let mut cmd = Command::cargo_bin("mk").expect("mk binary");
    let out = cmd
        .arg("gui-schema")
        .output()
        .expect("invoke mk gui-schema");
    assert!(out.status.success());
    let v: Value = serde_json::from_slice(&out.stdout).expect("JSON");
    let subs = v["subcommands"].as_array().expect("subcommands array");
    let names: Vec<&str> = subs
        .iter()
        .map(|s| s["name"].as_str().expect("name string"))
        .collect();
    for expected in ["encode", "decode", "inspect", "verify", "vectors"] {
        assert!(
            names.contains(&expected),
            "expected subcommand {expected:?} in {names:?}"
        );
    }
}

/// Spot-check kind classification: `--json` (boolean), `--out` on vectors (path).
#[test]
fn gui_schema_classifies_boolean_and_path_kinds() {
    let mut cmd = Command::cargo_bin("mk").expect("mk binary");
    let out = cmd
        .arg("gui-schema")
        .output()
        .expect("invoke mk gui-schema");
    assert!(out.status.success());
    let v: Value = serde_json::from_slice(&out.stdout).expect("JSON");
    let subs = v["subcommands"].as_array().expect("subcommands array");

    let decode = subs
        .iter()
        .find(|s| s["name"] == "decode")
        .expect("decode present");
    let decode_json = decode["flags"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["name"] == "--json")
        .expect("decode --json present");
    assert_eq!(decode_json["kind"], Value::from("boolean"));

    let vectors = subs
        .iter()
        .find(|s| s["name"] == "vectors")
        .expect("vectors present");
    let vectors_out = vectors["flags"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["name"] == "--out")
        .expect("vectors --out present");
    assert_eq!(vectors_out["kind"], Value::from("path"));
}

/// Positional repeating flag detected (e.g., `decode <mk1_strings>...`).
#[test]
fn gui_schema_detects_repeating_positional() {
    let mut cmd = Command::cargo_bin("mk").expect("mk binary");
    let out = cmd
        .arg("gui-schema")
        .output()
        .expect("invoke mk gui-schema");
    assert!(out.status.success());
    let v: Value = serde_json::from_slice(&out.stdout).expect("JSON");
    let subs = v["subcommands"].as_array().expect("subcommands array");
    let decode = subs
        .iter()
        .find(|s| s["name"] == "decode")
        .expect("decode present");
    let positionals = decode["positionals"].as_array().expect("positionals array");
    assert_eq!(positionals.len(), 1, "decode has one positional");
    assert_eq!(positionals[0]["name"], Value::from("mk1_strings"));
    assert_eq!(
        positionals[0]["repeating"],
        Value::from(true),
        "mk1_strings is repeating"
    );
}
