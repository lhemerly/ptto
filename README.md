🥔 # ptto (The "Just Deploy" Manifesto)
The world doesn't need another distributed PaaS. It needs a potato.

Modern web deployment is a trap. We've traded the simplicity of a Linux server for the cognitive overload of distributed YAML files, three different cloud dashboards, and the constant anxiety of usage-based billing.

```ptto``` is a highly opinionated, zero-dashboard CLI tool for deploying web applications to a single VPS (a "potato").

No Kubernetes. No Docker-compose hell. No cloud provider lock-in. No Vercel.

## The MVP Stack (Patient Zero)

Right now, ```ptto``` only deploys the most brutally efficient, indestructible stack known to web development: The Single Binary.
- Language: GoFrontend: HTMX (Server-Side HTML Rendering)
- Database: Embedded SQLite
- Proxy/SSL: Caddy

If you want microservices or a thick React SPA client, go pay AWS. If you want a stateful, high-performance app deployed in 3 seconds, use ```ptto```.

## Current baseline (implemented)

This repository now includes:
- A Rust CLI baseline using `clap` + `tokio`.
- Initial subcommands (`init`, `deploy`, `logs`, `generate-key`) with an SSH/scp execution engine (dry-run friendly).
- A PR-focused CI workflow that runs `fmt`, `clippy`, `build`, and `test` automatically.

## How it works

1. You buy a $5 VPS (Ubuntu/Debian).
2. You run the CLI on your local machine.
   ```
   # Setup the server (installs Caddy, prepares systemd)
     ptto init root@your-server-ip
   # Compile your Go app locally and deploy it
     ptto deploy --domain your-app.com
   ```

### What ```ptto``` actually does during deploy:
1. Compiles: Cross-compiles your Go web app locally (GOOS=linux GOARCH=amd64 go build).
2. Transfers: scps the single binary to the server.
3. Injects: Sets up a persistent SQLite directory and injects DATABASE_URL via systemd environment variables.
4. Secures: Generates a Caddyfile and automatically provisions Let's Encrypt SSL.
5. Restarts: Reloads the systemd service. Your new code is live.

## Development

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

## The "One-Time" Hook (CI/CD)

Want to deploy on ```git push```?Run ```ptto generate-key```. Put the string in your GitHub Repository Secrets. Use our official GitHub Action. Zero dashboards required.
