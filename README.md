# RustWho - Blazing Fast WHOIS & IP Lookup

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
| `PORT` | Local host port mapping for the backend. | `4404` |
| `SITE_TITLE` | Custom title rendered in the navigation header. | `RustWho` |
| `ALLOWED_ORIGINS` | Comma-separated HTTP request origins (CORS filter). | `*` |
| `RUSTWHO_PIN` | Optional 4-10 digit PIN to lock access to the utility. | None |
