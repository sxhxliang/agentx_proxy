# Repository Guidelines

## Project Structure & Module Organization
- Root `Cargo.toml` defines a Rust workspace with binaries: `agents/` (public proxy server), `agentc/` (edge client), and `common/` (shared protocol utilities).
- Each crate keeps source under `src/`; integration assets live beside binaries (e.g., `agentc/src/router.rs`, `common/src/http.rs`) so add new modules in the matching crate.
- Operational assets are tracked under `scripts/` (startup helpers), `docs/DEPLOYMENT.md`, and `nginx/agents.conf`; update these when you change ports, routing, or deployment expectations.

## Build, Test, and Development Commands
- `cargo check --workspace` validates compilation quickly and should back every PR.
- `cargo fmt --all` and `cargo clippy --all-targets --all-features` keep formatting and linting consistent; fix or justify warnings before review.
- `cargo test --workspace` runs unit and integration tests across crates; scope to a crate with `cargo test -p agentc`.
- Use `cargo run -p agents -- --control-port 17001 ...` or the scripts `./scripts/start_agents.sh` and `./scripts/start_agentc.sh` for end-to-end manual runs after a `cargo build --release`.

## Coding Style & Naming Conventions
- Follow default Rust 2021 style: 4-space indentation, snake_case for modules/functions, PascalCase for types, and SCREAMING_SNAKE_CASE for constants.
- Prefer async-friendly patterns already in place (Tokio, DashMap); reuse helpers from `common` rather than reimplementing serializers or socket code.
- Keep new config or CLI flags centralized in `agentc/src/config.rs` or the corresponding Clap parser, and document env-var fallbacks in the scripts.

## Testing Guidelines
- Add targeted `#[tokio::test]` or standard unit tests beside the code they cover; name modules `mod tests` and describe the behavior under test.
- Use `tests/` directories for cross-crate flows (e.g., handshake between `agents` and `agentc`); when doing so, spin up lightweight async servers rather than hitting real ports.
- Aim to cover error paths (timeouts, registration failures) and update scripts to mention any new manual validation.

## Commit & Pull Request Guidelines
- Existing history is minimal (`init`), so keep commits small, in imperative mood, and scoped (e.g., `feat(agentc): add proxy metrics`).
- Reference related issues in the body, summarize behavioral impact, and paste the command output that validates the change.
- Pull requests must describe the scenario, list config adjustments, and provide follow-up actions for operators (docs, scripts, or nginx updates).
- Include screenshots or log excerpts when changes affect runtime observability or startup scripts, especially under `scripts/` or `docs/`.
