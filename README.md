# RustWho

A secure, lightning-fast WHOIS, IP, and ASN lookup web utility built in Rust using **Yew** (WebAssembly frontend) and **Axum + Tokio** (asynchronous backend).

---

## ⚡ Quick Start (Time-To-First-Run)

### Running via Docker Compose
1. Ensure a `docker-compose.yml` file is configured in your project root:
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
      - APPRISE_URL=
      - APPRISE_MESSAGE=WHOIS Lookup for {query} ({query_type})
```
2. Spin up the container:
```bash
docker compose up -d
```
3. Open your browser and navigate to `http://localhost:4404`.

---

## 🛠️ Local Development

### 1. Prerequisites
Ensure you have the Rust toolchain installed. Add the WebAssembly target and install the **Trunk** WASM bundler:
```bash
# Add WebAssembly target
rustup target add wasm32-unknown-unknown

# Install Trunk
wget -qO- "https://github.com/trunk-rs/trunk/releases/download/v0.21.14/trunk-x86_64-unknown-linux-gnu.tar.gz" | tar -xzf- -C /usr/local/bin
```

### 2. Run the Application
1. **Frontend**: Start the Yew development server (runs with hot-reloading at `http://localhost:8080`):
   ```bash
   cd frontend
   trunk serve
   ```
2. **Backend**: Start the Axum API server (listens on `http://localhost:4404`):
   ```bash
   cd backend
   cargo run
   ```

---

## 📋 Environment Configuration

| Variable | Description | Default | Required |
| :--- | :--- | :--- | :--- |
| `PORT` | Local host port mapping for the Axum backend | `4404` | Optional |
| `SITE_TITLE` | Custom title rendered in the navigation header | `RustWho` | Optional |
| `ALLOWED_ORIGINS` | Comma-separated HTTP request origins (CORS filter) | `*` | Optional |
| `RUSTWHO_PIN` | Optional 4-10 digit PIN to lock access to the utility | None | Optional |
| `APPRISE_URL` | Apprise API webhook URL (e.g. Discord, Telegram) | None | Optional |
| `APPRISE_MESSAGE` | Custom webhook alert message template | `WHOIS Lookup for {query} ({query_type})` | Optional |

---

## 📂 Repository File Tree

```
RustWho/
├── .env
├── .env.example
├── .gitattributes
├── .gitignore
├── Cargo.lock
├── Cargo.toml (Workspace configuration)
├── Dockerfile (Multi-stage build)
├── LICENSE
├── README.md
├── docker-compose.yml
├── backend/
│   ├── Cargo.toml (Axum backend metadata)
│   └── src/
│       └── main.rs (Server logic & TCP query resolution)
└── frontend/
    ├── Cargo.toml (Yew dependencies)
    ├── index.html (WASM compilation entry)
    ├── Assets/
    │   ├── login.css (PIN layout styling)
    │   ├── styles.css (Variable themes & print stylesheets)
    │   ├── service-worker.js (Offline PWA caching)
    │   └── assets/
    │       ├── logo.png (Red branding icon)
    │       └── logo.svg (Red vector markup)
    └── src/
        ├── main.rs (State management & lookup rendering)
        ├── header.rs (Title, language & theme controller)
        ├── i18n.rs (Multi-language translations database)
        ├── storage.rs (LocalStorage interface wrapper)
        └── types.rs (Global definitions)
```

---

## 🛡️ License

Distributed under the MIT License. See [LICENSE](LICENSE) for more details.
