# 🥔 ptto (The "Just Deploy" Manifesto)

[![codecov](https://codecov.io/gh/ptto/ptto/branch/main/graph/badge.svg)](https://codecov.io/gh/ptto/ptto)

> The world doesn't need another distributed PaaS. It needs a potato.

Modern web deployment is a trap. We've traded the simplicity of a Linux server for the cognitive overload of distributed YAML files, three different cloud dashboards, and the constant anxiety of usage-based billing.

`ptto` is a highly opinionated, zero-dashboard CLI tool for deploying web applications to a single VPS (a "potato").

No Kubernetes. No Docker-compose hell. No cloud provider lock-in. No Vercel.

## The MVP Stack (Patient Zero)

Right now, `ptto` only deploys the most brutally efficient, indestructible stack known to web development: **The Single Binary.**

* **Language**: Go

* **Frontend**: HTMX (Server-Side HTML Rendering)

* **Database**: Embedded SQLite

* **Auth**: Native Passkeys (WebAuthn)

* **Proxy/SSL**: Caddy

If you want microservices or a thick React SPA client, go pay AWS. If you want a stateful, high-performance app deployed in 3 seconds, use `ptto`.

## How it works

1. You buy a \$5 VPS (Ubuntu/Debian).

2. You initialize the project locally.

```
# Start a new project with Passkeys, SQLite, and HTMX pre-configured
ptto new my-app
cd my-app

# Connect it to your VPS
ptto init

```

This creates a tiny `.ptto.toml` file in your directory. No complex YAML.

```
host = "root@203.0.113.10"
domain = "your-app.com"

```

3. You deploy.

```
ptto deploy

```

### What `ptto` actually does during `deploy`:

1. **Compiles**: Cross-compiles your Go web app locally (`GOOS=linux GOARCH=amd64 go build`).

2. **Transfers**: `scp`s the single binary to the server using your local OS credentials.

3. **Injects**: Sets up a persistent SQLite directory and injects `DATABASE_URL` via systemd environment variables.

4. **Secures**: Generates a Caddyfile and automatically provisions Let's Encrypt SSL.

5. **Restarts**: Reloads the `systemd` service. Your new code is live.

## Management (The Terminal Dashboard)

`ptto` refuses to build web dashboards. Instead, it uses its encrypted SSH bridge to pipe your server's native telemetry and data directly to your local terminal.

* `ptto logs` - Streams `systemd-journald` logs live to your terminal.

* `ptto db shell` - Drops you into a remote `sqlite3` interactive prompt for your production database.

* `ptto db pull` / `ptto db push` - Safely syncs the `database.sqlite` file to your local machine for GUI editing.

* `ptto top` - Streams server CPU/RAM usage (`htop` / `bottom`).

* `ptto traffic` - Pipes Caddy access logs into a real-time terminal analytics dashboard (`goaccess`).
