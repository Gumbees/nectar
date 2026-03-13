# Nectar — Media Server

## Project Overview
Nectar is a modern media server built from scratch in Rust (backend) + Solid.js (PWA frontend). Inspired by Jellyfin but designed to fix its pain points: broken mobile video, slow transcoding, SQLite limitations, no semantic search.

## Architecture
- **server/** — Rust (Axum) API server. PostgreSQL via SQLx. NATS for job dispatch.
- **transcoder/** — Stateless Rust worker that pulls jobs from NATS and runs FFmpeg with hardware acceleration.
- **web/** — Solid.js TypeScript PWA. Vite build. HLS.js for adaptive streaming.
- **docker/** — Docker Compose stack with pgvector PostgreSQL, NATS, nginx reverse proxy.

## Key Design Decisions
- PostgreSQL + pgvector (not SQLite) — supports large libraries and vector similarity search
- NATS for transcoder queue — lightweight, supports JetStream for persistence
- Stateless transcoder workers — can run on any machine (Orange Pi, GPU server, etc.)
- Direct play preferred — only transcode when client can't handle the codec
- PWA with proper `<video>` fullscreen — works with iPad/Android screen mirroring

## Tech Stack
- Rust 2024 edition, Axum 0.8, SQLx 0.8, Tokio
- Solid.js 1.9, TypeScript, Vite, HLS.js
- PostgreSQL 17 with pgvector extension
- NATS 2.x with JetStream
- FFmpeg for all transcoding

## Conventions
- Rust: snake_case, modules per domain (api/, services/, media/)
- TypeScript: camelCase, components in PascalCase
- API routes: `/api/v1/{resource}`
- Environment config: `NECTAR__` prefix for server, `NECTAR_TRANSCODER__` for workers
