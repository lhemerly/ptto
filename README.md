# 🥔 ptto V0.2.0

[![codecov](https://codecov.io/github/lhemerly/ptto/graph/badge.svg?token=O6W62RLDY4)](https://codecov.io/github/lhemerly/ptto)

> The world doesn't need another distributed PaaS. It needs a potato.

`ptto` is a highly opinionated, zero-dashboard CLI for deploying single-binary web apps to one VPS.

No Kubernetes. No Docker Compose sprawl. No cloud control panel.

## Current capabilities

Today, `ptto` focuses on a Go single-binary workflow with multi-tenant VPS support:

- Build target: `GOOS=linux GOARCH=amd64`
- Multi-tenancy: isolated apps on a single VPS with Caddy import directory (`/etc/caddy/apps/*.caddy`) and directory isolation (`/opt/ptto/apps/<app>/`)
- Runtime strategy: native blue-green process swap managed over SSH
- Edge proxy + TLS: `Caddy` (Let's Encrypt automated SSL)
- Persistence: tenant-isolated SQLite at `/opt/ptto/apps/<app>/data/database.sqlite` (or `/opt/ptto/data/database.sqlite` for default single app)
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
# optional multi-tenant app name (defaults to "ptto-app")
app = "my-service"
# optional
ssh_key = "~/.ssh/id_ed25519"
source = "./cmd/server" # defaults to "."
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
# ptto deploy --app my-service --domain your-app.com --target root@203.0.113.10
```

## Commands

### Deploy lifecycle

- `ptto init [target] [--dry-run]`
  - Installs/validates Caddy + goaccess, initializes Caddy app import directory, and enables Caddy on the target host.
- `ptto deploy [--app <name>] [--domain <domain>] [--target <user@host>] [--artifact <path>] [--source <path>] [--dry-run]`
  - Builds your Go app for Linux amd64.
  - Copies artifact to remote host over SSH/SCP.
  - Uploads a new release binary into isolated directory (`/opt/ptto/apps/<app>/bin/`) and launches it on a random open localhost port.
  - Generates `/etc/caddy/apps/<app>.caddy`, validates configuration, reloads Caddy gracefully, and terminates the previous process.

### Operations

- `ptto logs [service] [--app <name>] [--target <user@host>]`
  - Streams `journalctl` logs (defaults to resolved app name or `ptto-app`).
- `ptto top [--target <user@host>]`
  - Opens `htop`, `btop`, or `top` on the remote host.
- `ptto traffic [--target <user@host>]`
  - Streams Caddy access logs into `goaccess` in your terminal.

### Database

- `ptto db [--app <name>] [--target <user@host>] shell`
- `ptto db [--app <name>] [--target <user@host>] pull [local_path]`
- `ptto db [--app <name>] [--target <user@host>] push [local_path]`

### Utility

- `ptto generate-key`
  - Placeholder hook for future CI/CD deploy-key workflows.

## Behavior notes

- `host`, `domain`, `app`, and optional `ssh_key` and `source` are read from `.ptto.toml` when command flags are omitted.
- Domain validation rejects whitespace/control characters and malformed hostnames.
- Application name validation permits ASCII letters, digits, hyphens, and underscores up to 64 characters.
- `--dry-run` shows planned build/remote actions without executing remote mutations.

## Examples

```bash
# bootstrap with explicit target
ptto init root@203.0.113.10 --dry-run

# deploy using config defaults
ptto deploy --dry-run

# deploy specific tenant app
ptto deploy --app auth-service --domain auth.example.com --dry-run

# tail custom service logs
ptto logs --app auth-service --target root@203.0.113.10

# pull production sqlite db for a tenant app
ptto db pull ./auth.sqlite --app auth-service --target root@203.0.113.10
```

## Disclaimer

`ptto` is intentionally opinionated and currently optimized for Ubuntu/Debian-like targets with `apt-get`.
