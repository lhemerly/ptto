# Roadmap

The goal is to get from zero to a reliable, production-ready "potato" stack as fast as possible, without feature creep.

# Phase 1: Patient Zero (Go + SQLite) - Current

[ ] CLI Scaffolding: Setup the Rust CLI structure (clap, tokio).

[ ] SSH Engine: Implement secure SSH execution and file transfer from local to remote using libssh2 or system ssh.

[ ] Server Init: Automate installation of Caddy on the remote VPS.

[ ] Go Compiler Wrapper: Automate GOOS=linux GOARCH=amd64 go build.

[ ] Systemd Management: Auto-generate and reload systemd service files for the Go binary.

[ ] Caddy Routing: Auto-generate Caddyfile for reverse proxying port 80/443 to the binary's internal port.

# Phase 2: Resilience (The Potato Survives)

[ ] Automated Backups: CLI configures a systemd timer on the server to sync the SQLite file to an S3/R2 bucket via a lightweight sync tool.

[ ] Zero-Downtime Deploys: Implement a blue-green swap on the server. Spin up the new binary on a random open port, update Caddy, gracefully kill the old process.

[ ] Log Tailing: ptto logs command to stream journalctl logs directly to the local terminal.

# Phase 3: Language Expansion

[ ] Rust Server Support: Expand the build step to support compiling Rust web servers (Axum/Actix) via cargo build --target x86_64-unknown-linux-musl.

[ ] Zig / C / Nim: Support any web framework that compiles to a statically linked Linux binary.
