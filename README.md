# Software Factory — macOS Native App

Full Rust, zero runtime dependencies. AppKit via `objc2`, no Python, no swift-bridge.

## Architecture

```
sf-macos/crates/
├── sf-core/     # AgentExecutor, LlmClient (Ollama/Azure/MiniMax), PatternEngine, ToolRunner
├── sf-db/       # Local SQLite (rusqlite bundled), migrations, CRUD
├── sf-remote/   # RemoteInstance (HTTP API), SSE consumer, OAuth2 PKCE, Keychain
└── sf-app/      # AppKit UI via objc2 — NSWindow, views, NSStatusItem
```

## LLM Providers

| Provider | Local/Remote | Notes |
|----------|-------------|-------|
| Ollama | Local (localhost:11434) | OpenAI-compatible `/v1/` |
| Azure OpenAI | Remote | `max_completion_tokens` (not `max_tokens`) |
| MiniMax | Remote | Strips `<think>` blocks, no temperature param |

## Minimal Crates (≤ 10)

`tokio`, `reqwest`, `serde`/`serde_json`, `rusqlite` (bundled), `objc2`/`objc2-app-kit`, `sha2`, `base64`, `async-trait`, `futures-util`

## OAuth2 PKCE

Implemented manually (RFC 7636) — no `oauth2` crate.  
`pkce_challenge(verifier) = base64url(SHA256(verifier))`

## Reasoning Model

Source: arXiv:2603.01896 — semi-formal agentic reasoning.  
Each LLM turn: **Premises** (system + history) → **Trace** (tool calls) → **Verdict** (final text).

## Build

```bash
cargo build --target aarch64-apple-darwin
cargo test -p sf-db -p sf-core -p sf-remote
```

## Modes

| Mode | LLM | Storage |
|------|-----|---------|
| Local | Ollama | SQLite embedded |
| Remote OVH | Azure (server-side) | PostgreSQL + SSE |
| Remote Azure | Azure (server-side) | PostgreSQL + SSE |
