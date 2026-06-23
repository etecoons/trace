# RustWho

A clean, secure, and lightning-fast WHOIS, IP, and ASN lookup web utility built in Rust.

---

## ⚡ Quick Start & Deployment

### Running via Docker Compose

1. Create a `docker-compose.yml` file in your directory:

```yaml
services:
  rustwho:
    image: ubermetroid/rustwho:latest
    container_name: rustwho
    restart: unless-stopped
    ports:
      - 4404:4404
    environment:
      - PORT=4404
      - SITE_TITLE=RustWho
      - ALLOWED_ORIGINS=*
      - RUSTWHO_PIN=1234
```

2. Spin up the container:

```bash
docker compose up -d
```

3. Open your browser and navigate to `http://localhost:4404`.

---

## 📋 Environment Configuration

| Variable | Description | Default | Required |
| :--- | :--- | :--- | :--- |
| `PORT` | Local host port mapping for the backend | `4404` | Optional |
| `SITE_TITLE` | Custom title rendered in the navigation header | `RustWho` | Optional |
| `ALLOWED_ORIGINS` | Comma-separated HTTP request origins (CORS filter) | `*` | Optional |
| `RUSTWHO_PIN` | Optional 4-10 digit PIN to lock access to the utility | None | Optional |

---

## 🛡️ License

Distributed under the MIT License. See `LICENSE` for more details.
