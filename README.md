# RustWho - Blazing Fast WHOIS & IP Lookup

<p align="center">
  <img src="https://raw.githubusercontent.com/UberMetroid/RustWho/main/frontend/Assets/assets/logo.png" alt="RustWho Logo" width="128" height="128">
</p>

RustWho is a clean, secure, and lightning-fast WHOIS, IP, and ASN lookup web utility built in Rust.

---

## 🐳 Container Installation

### Option 1: Docker Compose (Recommended)

1. Create a `docker-compose.yml` file:

```yaml
version: '3'
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

2. Run the container:

```bash
docker compose up -d
```

3. Open your browser and navigate to `http://localhost:4404`.

### Option 2: Docker CLI

Run the following command to start the container:

```bash
docker run -d \
  --name rustwho \
  --restart unless-stopped \
  -p 4404:4404 \
  -e RUSTWHO_PIN=1234 \
  ubermetroid/rustwho:latest
```

---

## 📋 Configuration Options

Configure these settings inside your Docker Compose environment or container environment variables:

| Variable | Description | Default |
| :--- | :--- | :--- |
| `PORT` | The port number the backend HTTP server will bind to inside the container. | `4404` |
| `SITE_TITLE` | Custom website title rendered in navigation headers, browser tabs, and PWA manifest. *(Supports fallback `RUSTRUSTWHO_TITLE`)* | `RustWho` |
| `BASE_URL` | Application base URL. Essential when deploying behind reverse proxies to ensure redirect and websocket links are resolved correctly. | `http://localhost:4404` |
| `ALLOWED_ORIGINS` | Comma-separated list of allowed HTTP request origins (CORS filter). Use `*` to allow all origins. | `*` |
| `RUSTWHO_PIN` | Optional 4–10 digit PIN (numerical only) to lock access to the interface. Leave empty for public mode. | None |
| `TZ` | Timezone for the container processes and logs. | `UTC` |
