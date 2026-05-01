# Roadmap

The goal is to get from zero to a reliable, production-ready "potato" stack as fast as possible, without feature creep.

# Phase 1: Patient Zero (Go + SQLite) - Current

[x] CLI Scaffolding: Setup the Rust CLI structure (clap, tokio).

[x] CI Baseline: Add GitHub Actions checks for fmt, clippy, build, and tests on PRs.

[x] SSH Engine: Implement secure SSH execution and file transfer from local to remote using libssh2 or system ssh.

[x] Server Init: Automate installation of Caddy on the remote VPS.

[x] Go Compiler Wrapper: Automate GOOS=linux GOARCH=amd64 go build.

[x] Systemd Management: Auto-generate and reload systemd service files for the Go binary.

[x] Caddy Routing: Auto-generate Caddyfile for reverse proxying port 80/443 to the binary's internal port.

Phase 2: The Terminal Dashboard (Local DX)

We have deployment. Now we need visibility without web-bloat.

[x] Config Parser: Implement parsing for the .ptto.toml local file to remove the need for CLI flags on every run.

[x] DB Management Suite: Implement ptto db shell, ptto db pull, and ptto db push using native SSH wrappers.

[x] Telemetry Wrappers: Implement ptto logs, ptto top, and ptto traffic (automating goaccess installation via init).

Phase 3: Resilience & Automation

Making the potato indestructible.

[ ] Automated Backups: CLI configures a systemd timer on the server to sync the SQLite WAL/file to an S3/R2 bucket (e.g., via Litestream or rclone).

[ ] Zero-Downtime Deploys: Implement a blue-green swap on the server. Spin up the new binary on a random open port, update Caddy, gracefully kill the old process.

[ ] CI/CD Hook: Create the ptto generate-key workflow and official GitHub Action for headless deployments.

Phase 4: Expansion (The 1-to-Many)

Once the 1-VPS-to-1-App model is flawless, we expand.

[ ] Multi-Tenancy: Upgrade the server architecture to support multiple .ptto.toml projects on a single potato using Caddy import directories and systemd namespace isolation.

[ ] Language Expansion: Expand the build step to support compiling Rust web servers (Axum/Actix) via cargo build --target x86_64-unknown-linux-musl.
