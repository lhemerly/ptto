# 🥔 ptto V0.1.2

[![codecov](https://codecov.io/github/lhemerly/ptto/graph/badge.svg?token=O6W62RLDY4)](https://codecov.io/github/lhemerly/ptto)

> The world doesn't need another distributed PaaS. It needs a potato.

`ptto` is a highly opinionated, zero-dashboard CLI for deploying single-binary web apps to one VPS.

No Kubernetes. No Docker Compose sprawl. No cloud control panel.

## Current capabilities (MVP)

Today, `ptto` focuses on a Go single-binary workflow:

- Build target: `GOOS=linux GOARCH=amd64`
- Runtime strategy: native blue-green process swap managed over SSH
- Edge proxy + TLS: `Caddy` (Let's Encrypt)
- Persistence: remote SQLite file at `/opt/ptto/data/database.sqlite`
- Ops UX: SSH-native logs, process dashboard, and access-log traffic analytics

## Installation

```bash
cargo install --path .
```

Or run without installing:

```bash
cargo run -- <COMMAND>
```

## Quick start

1. Create a `.ptto.toml` in your app directory:

```toml
host = "root@203.0.113.10"
domain = "your-app.com"
# optional
ssh_key = "~/.ssh/id_ed25519"
```

2. Prepare the VPS once:

```bash
ptto init
# or: ptto init root@203.0.113.10
```

3. Deploy:

```bash
ptto deploy
# or explicitly:
# ptto deploy --domain your-app.com --target root@203.0.113.10
```

## Commands

### Deploy lifecycle

- `ptto init [target] [--dry-run]`
  - Installs/validates Caddy + goaccess and enables Caddy on the target host.
- `ptto deploy [--domain <domain>] [--target <user@host>] [--artifact <path>] [--source <path>] [--dry-run]`
  - Builds your Go app for Linux amd64.
  - Copies artifact to remote host over SSH/SCP.
  - Uploads a new release binary and launches it on a random open localhost port.
  - Swaps Caddy upstream to the new port and gracefully terminates the previous process.

### Operations

- `ptto logs [service] [--target <user@host>]`
  - Streams `journalctl` logs (default service: `ptto-app`).
- `ptto top [--target <user@host>]`
  - Opens `htop`, `btop`, or `top` on the remote host.
- `ptto traffic [--target <user@host>]`
  - Streams Caddy access logs into `goaccess` in your terminal.

### Database

- `ptto db shell [--target <user@host>]`
- `ptto db pull [local_path] [--target <user@host>]`
- `ptto db push [local_path] [--target <user@host>]`

### Utility

- `ptto generate-key`
  - Placeholder hook for future CI/CD deploy-key workflows.

## Behavior notes

- `host`, `domain`, and optional `ssh_key` are read from `.ptto.toml` when command flags are omitted.
- Domain validation rejects whitespace/control characters.
- `--dry-run` shows planned build/remote actions without executing remote mutations.

## Examples

```bash
# bootstrap with explicit target
ptto init root@203.0.113.10 --dry-run

# deploy using config defaults
ptto deploy --dry-run

# tail custom service logs
ptto logs my-app --target root@203.0.113.10

# pull production sqlite db
ptto db pull ./prod.sqlite --target root@203.0.113.10
```

## Disclaimer

`ptto` is intentionally opinionated and currently optimized for Ubuntu/Debian-like targets with `apt-get`.
