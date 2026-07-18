<h1 align="center">MyCodex</h1>

<p align="center">
  An opinionated Codex fork focused on a better terminal workflow, stronger session ergonomics, and quality-of-life features for people who live inside the TUI.
</p>

<p align="center">
  <img src="./.github/codex-cli-splash.png" alt="MyCodex splash" width="80%" />
</p>

<p align="center">
  Forked from <a href="https://github.com/openai/codex">openai/codex</a>.
  This repository keeps the upstream foundation, then layers custom UX and workflow improvements on top.
</p>

---

## Quick Start

Choose one of these paths:

- Build from source:

  ```shell
  cd codex-rs
  cargo run --bin codex
  ```

- Run through the repo `justfile`:

  ```shell
  cd codex-rs
  just codex
  ```

If you prefer packaged upstream installs, the original Codex CLI options are still documented in the
[OpenAI Codex docs](https://developers.openai.com/codex).

## What MyCodex Changes

MyCodex is not a rewrite. It is a maintained fork that keeps pace with upstream Codex while improving the day-to-day terminal experience for local agent work.

The main areas of focus in this fork are:

- Better visibility into what the agent is doing.
- Better recovery when work is interrupted by limits or account changes.
- Better local tooling for long-running interactive sessions.
- Better customization of the TUI surface.

## Features

> For the latest fork-specific work, check the commit history and repository releases.

### Status Line and Session Visibility

MyCodex expands the TUI status surfaces so important session context stays visible while you work:

- configurable status line items
- optional theme-derived status line colors
- live session timer
- stronger workspace and activity visibility

This makes it easier to understand the current model, workspace, runtime state, and session duration at a glance.

### Prompt Queue Management

MyCodex adds stronger control over queued follow-up prompts when a turn cannot continue immediately:

- inspect and manage queued prompts with `/queue`
- pause queued auto-send behavior
- resume later without losing follow-up work
- keep working even when usage limits interrupt the current flow

This is especially useful when you are iterating quickly and do not want rate limits or temporary pauses to break momentum.

### Session Recap

MyCodex adds a local session recap flow so the TUI can summarize what happened during a working session:

- review decisions, completed work, problems, and changed files
- trigger a recap manually with `/recap`
- benefit from automatic recap snapshots during longer sessions

This gives you a fast local summary without depending on external note-taking.

### Background Process Management

Long-running terminal work is easier to manage in this fork:

- inspect managed background processes with `/processes` or `/ps`
- stop background terminals explicitly when needed
- surface process state inside the TUI instead of treating background work as invisible

### Theme and UI Customization

MyCodex adds more control over how the TUI looks and feels:

- semantic UI theme support
- syntax theme selection
- configurable status line composition
- terminal title customization

The goal is simple: make the interface easier to scan during long sessions without turning it into noise.

### Safer Account and Recovery Flow

This fork includes workflow improvements around auth changes and interrupted sessions:

- safer `auth.json` reload behavior
- better account-switch pickup boundaries
- automatic recovery-oriented resumption after usage-limit interruptions

These changes are aimed at reducing restarts and manual recovery when you switch accounts or hit plan limits mid-session.

## Upstream Strategy

This fork is maintained by re-applying MyCodex changes onto fresh upstream Codex updates instead of carrying a long-lived merge history forever.

That keeps the codebase closer to current upstream behavior while preserving fork-specific improvements.

## Repository Layout

- `codex-rs/`: Rust implementation and TUI
- `codex-cli/`: CLI packaging entrypoint
- `sdk/`: language SDKs
- `docs/`: repository and contributor documentation
- `tools/`: maintenance and lint tooling

## Development

Install dependencies and run from source:

```shell
cd codex-rs
just install
just codex
```

Useful commands:

```shell
cd codex-rs
just fmt
just test -p codex-tui
just test -p codex-core
```

## Documentation

- [OpenAI Codex documentation](https://developers.openai.com/codex)
- [Repository contributing guide](./docs/contributing.md)
- [Install and build notes](./docs/install.md)
- [codex-rs README](./codex-rs/README.md)

## Credits

- Upstream project: [openai/codex](https://github.com/openai/codex)
- Inspiration for README positioning and feature-first presentation: [Loongphy/codext](https://github.com/Loongphy/codext)

## License

This repository is licensed under the [Apache-2.0 License](./LICENSE).
