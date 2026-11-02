# fix-ci-portability-mk

Commit b407fb8 on fix/bughunt (not pushed). Added cfg_attr(not(linux), ignore) to bughunt_mk_03 in crates/mk-cli/tests/bughunt.rs. No other test in crates/*/tests/bughunt.rs uses timeout, ulimit or sh. No STOP.

Gate: nextest --workspace 399 run/399 passed; bughunt_mk_ 8/8; fmt and clippy clean.

0 Critical, 0 Important
