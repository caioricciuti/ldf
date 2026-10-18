//! `ldf-mcp`: a Model Context Protocol proxy over stdio.
//!
//! It launches a downstream MCP server as a child process, forwards every
//! JSON-RPC message in both directions unchanged, and rewrites exactly three
//! things on the way back to the client:
//!
//! * `initialize` results get a short LDF primer appended to `instructions`;
//! * `tools/list` results get one sentence appended to each tool description;
//! * `tools/call` results whose text content is a JSON dataset are re-encoded
//!   as LDF when that is shorter.
//!
//! Nothing else is touched, so the model keeps calling the downstream tools
//! exactly as before and receives cheaper data. `ldf-mcp clickhouse` is a
//! server of its own with three read-only tools over ClickHouse's HTTP
//! interface, and `ldf-mcp files` one over a directory of data files; both
//! return LDF directly. The protocol surface used is
//! the stdio transport of MCP 2025-06-18: newline-delimited JSON-RPC
//! messages, logging on stderr only.

pub mod clickhouse;
pub mod fake;
pub mod files;
pub mod proxy;
pub mod rewrite;
pub mod rpc;
