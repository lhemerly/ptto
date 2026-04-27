# Contributing to ```ptto```

We welcome contributions, but be warned: We say "No" to 99% of feature requests.

```ptto``` exists because modern infrastructure is bloated. We will not rebuild Kubernetes.

# Our Core Opinions (Do not violate these)

- Single Node Only: If a PR requires a load balancer or an external managed database, it will be closed.
- The Single Binary Rule: We deploy compiled binaries. We do not install Node.js, Python, or Ruby runtimes on the server. The server remains pristine Linux.
- SQLite Only: We do not support external Postgres or MySQL. SQLite is the only persistent state.
- Zero Configuration: If a user has to write a configuration file (like a Dockerfile or a YAML script) to make your feature work, the feature is broken. ptto figures it out, or ptto fails.

# Development Setup

The ```ptto``` CLI is built in Rust to ensure a fast, cross-platform, single-binary distribution for the tool itself.

1. Ensure you have cargo installed.
2. We highly recommend using Nix for a reproducible dev environment (see flake.nix).
3. Run tests with cargo test.
4. Please run cargo fmt and cargo clippy before submitting a PR.
