<h1 align="center">
  <img src="assets/icon.png" width="48" height="48" valign="middle"> Trace
</h1>

<p align="center">
  <b>High-performance ASN, IP, WHOIS, and network intelligence diagnostic dashboard written in Rust.</b>
</p>

---

### Instant One-Line Install (Docker Container)

Run the official zero-dependency container on port 8080:

```bash
docker run -d --name trace -p 8080:8080 -v /mnt/user/appdata/trace:/config ghcr.io/studio2201/trace:latest
```

Open your browser to `http://localhost:8080` to access the IP & ASN intelligence dashboard.

---

### One-Line Install (Native Package Manager)

On Debian, Ubuntu, Fedora, or RHEL:

```bash
curl -fsSL https://studio2201.github.io/packages/install.sh | sudo bash
```

---

### Unraid NAS Deployment

Deploy via the official Unraid Template:

1. Copy [`trace.xml`](trace.xml) to your Unraid flash drive under `/boot/config/plugins/dockerMan/templates-user/`.
2. Open **Docker** -> **Add Container** -> Select **trace** from the template dropdown.
3. Click **Apply**.

---

### Environment Configuration

The backend service can be customized using the following environment variables:

| Variable | Description | Default |
| :--- | :--- | :---: |
| `TRACE_PORT` | Network port the web server binds to | `8080` |
| `TRACE_PIN` | Security PIN required for application access | *(Disabled)* |
| `TRACE_SITE_TITLE` | Custom branding title for web dashboard | `Trace` |
| `TRACE_ALLOWED_ORIGINS` | CORS allowed origins list (comma-separated) | `*` |
| `TRUST_PROXY` | Honor reverse proxy headers (`X-Forwarded-For`) | `false` |
| `TRUSTED_PROXY_IPS` | Comma-separated CIDR list of trusted reverse proxies | *(None)* |
| `LOG_LEVEL` | Tracing filter (`error`, `warn`, `info`, `debug`) | `info` |

---

### Administration CLI & TUI Dashboard

Every container and package includes a built-in administration utility (`trace`).

Launch interactive TUI dashboard:
```bash
docker exec -it trace trace tui
```

System diagnostics and self-healing check:
```bash
docker exec -it trace trace doctor
```

CLI Command Reference:
- `trace tui` — Interactive terminal user interface.
- `trace doctor` — Diagnoses network connectivity, DNS resolvers, and rate limits.
- `trace status` — Displays service configuration and security settings.
- `trace data stats` — Shows lookup metrics and cache stats.

---

### Architecture & Security

- **Axum Web Backend**: High-concurrency async HTTP runtime built on Tokio.
- **Yew WebAssembly Frontend**: Client-side single-page app compiled to WASM.
- **Upstream Rate Limiting**: Throttles PeeringDB, RIPE Stat, and RDAP APIs to prevent IP bans.
- **Fail-Closed Security PIN Authentication**: Rate-limited brute force protection with automatic lockout timers.

---

### License

Distributed under the Apache 2.0 License. See [LICENSE](LICENSE) for details.

---

<p align="center">
  <a href="https://github.com/studio2201/trace">
    <img src="assets/trace-header.jpg" alt="studio2201 banner" width="100%">
  </a>
</p>
