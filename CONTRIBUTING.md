# Contributing

LDF is a research project with a product attached: every rule of the format
exists because a measurement asked for it, and every number in the docs
comes from a file the benchmark wrote. Contributions that follow the same
discipline are welcome.

## Before you start

* Read `AGENTS.md` for the architecture, the commands and the conventions.
* Read `site/src/content/docs/docs/spec.md` before touching the encoder or the decoder,
  and `site/src/content/docs/docs/design.md` before proposing a syntax change. A
  syntax change is a measurement first and a pull request second.
* Open an issue for anything bigger than a fix, so the approach is agreed
  before the work.

## The bar

* `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`
  and `cargo test --release` are green. CI runs the same three plus the
  small round trip.
* A behaviour change to the format comes with conformance fixtures under
  `crates/ldf/tests/fixtures` and a spec change in the same pull request.
* A performance claim comes with the `ldf-bench` output that supports it.
  Do not edit numbers in `reports/` by hand; copy the files the tool
  wrote.
* No new external crate without a discussion. Every dependency is pinned
  to an exact version and the lockfile is committed. Anything published in
  the last seven days does not go in.
* No API calls from Rust code. The evaluation scripts under `eval/` are
  standard-library Python, spend-gated behind `--confirm-spend`, and read
  keys from the environment or a gitignored `.env`.

## Commits and pull requests

* Conventional commits: `feat(ldf): ...`, `fix(ldf-mcp): ...`,
  `docs(eval): ...`, `perf(ldf): ...`.
* Small pull requests. One change, its tests and its docs.
* Dependency changes never ride inside an unrelated diff; say them in the
  description, at the top.
* Commits on `main` are signed. A pull request is squashed or rebased by
  the maintainer; do not force-push a branch that has been reviewed.

## Reporting a problem

Use the issue templates. For a wrong encoding or decoding, attach the
smallest input that shows it and the output you expected; the round-trip
level you expected (byte, semantic, semantic-unordered) matters. For a
security problem, see `SECURITY.md`.
