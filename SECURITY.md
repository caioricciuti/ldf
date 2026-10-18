# Security

## Reporting

Report a vulnerability privately through GitHub's "Report a vulnerability"
button on the Security tab of this repository. Do not open a public issue
for it. You will get an acknowledgement within a week.

## Scope

The parts of this repository that process untrusted input:

* `crates/ldf`: the decoder reads text that may come from anywhere. It is
  written without `unsafe`, parses with bounded recursion over the line
  structure, and treats every malformed input as an error with a line
  number rather than a panic. A panic on any input is a bug; report it.
* `crates/ldf-mcp`: the proxy forwards JSON-RPC between a client and a
  downstream server and rewrites tool results; the ClickHouse mode runs
  SQL that the model wrote, always with `readonly=1`, against the server
  you point it at, with the password taken from an environment variable
  only. It makes no other network connection.
* `crates/ldf-data` readers (JSON, JSONL, CSV, Parquet) take files you
  choose.

The evaluation scripts under `eval/` call paid APIs only when run with
`--confirm-spend`; keys never enter the repository.

## Supply chain

Every crate is pinned to an exact version in `Cargo.toml`; `Cargo.lock` is
committed; versions younger than seven days are not taken. The website's two
npm packages are pinned and installed from a frozen lockfile with lifecycle
scripts disabled. The CI workflow uses one external action, pinned to a
commit hash.
