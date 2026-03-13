# Nectar

A modern media server built in Rust with a Solid.js PWA frontend.

## Architecture

```
nectar/
├── server/       Rust API server (Axum, SQLx, PostgreSQL)
├── transcoder/   Stateless FFmpeg worker nodes (NATS queue)
├── web/          Solid.js PWA frontend
├── docker/       Docker Compose stack
└── proto/        Shared API types
```

## Features

- **Rust backend** — Axum async server, PostgreSQL with pgvector
- **Solid.js PWA** — installable, offline-capable, proper fullscreen video on iPad/Android
- **Distributed transcoding** — stateless workers connected via NATS, supports NVIDIA, Intel QSV, AMD AMF, VA-API, V4L2 (Orange Pi/ARM)
- **Hardware-accelerated trickplay** — GPU-generated preview thumbnails
- **Vector search** — find media by description using Ollama or OpenAI embeddings
- **OIDC/OAuth** — SSO support alongside local auth
- **PostgreSQL** — designed for large libraries with pgvector similarity search
- **Direct play first** — codec negotiation to avoid unnecessary transcoding
- **HDR tone mapping** — automatic HDR to SDR when needed

## Quick Start

```bash
cp .env.example .env
# Edit .env with your paths and passwords

cd docker
docker compose up -d
```

## Development

```bash
# Backend
cargo run --package nectar-server

# Frontend
cd web && npm install && npm run dev

# Transcoder worker
cargo run --package nectar-transcoder
```

## License

MIT
