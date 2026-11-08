# Quick-Box 📬

> **Layanan Inbox Email Sementara untuk Developer (Open Source & Self-Hostable)**  
> Terinspirasi dari kesederhanaan Yopmail, diperkaya dengan keamanan kata sandi, retensi 30 hari, ekstraksi OTP instan, dan arsitektur monolitik modern berbasis **Rust** dan **React**.

---

## 📌 Status Proyek Saat Ini

- [x] **PRD & Kebutuhan Inti**: Disepakati (Lihat [docs/PRD.md](docs/PRD.md))
- [x] **Brainstorming Arsitektur**: Opsi Monolitik Asinkron Terpilih (Tokio + Axum + Embedded SMTP + SQLite + React)
- [x] **Desain Teknis & Spesifikasi**: Selesai di-review (Lihat [docs/superpowers/specs/2026-11-04-quick-box-design.md](docs/superpowers/specs/2026-11-04-quick-box-design.md))
- [x] **Referensi UI**: Disimpan di [docs/design/reference-ui.jpg](docs/design/reference-ui.jpg) (Desain 3-pane modern: Sidebar, List, Message Reader)
- [ ] **Fase Implementasi**: Siap dieksekusi bertahap (Fase 1 s/d Fase 8)

---

## 🏗️ Gambaran Arsitektur

```
[ Pengirim Email ] ──SMTP:25──▶ [ Listener SMTP (Tokio) ] ──▶ [ SQLite (WAL) + Disk Attachments ]
                                         │                                      │
[ Browser (React) ] ◀──HTTP + SSE───────▶ [ API Server (Axum) ] ◀──────────────┘
                                                 │
                                 [ Background Worker (Retensi 30 Hari) ]
```

### Stack Teknologi
- **Backend**: Rust (Tokio async runtime, Axum web framework, `mail-parser` untuk MIME, `sqlx` untuk SQLite).
- **Frontend**: React 19 + Vite + TypeScript + Tailwind CSS (Desain modern 3-pane ala Swiftlet, Dark/Light mode).
- **Database**: SQLite (mode Write-Ahead Logging / WAL) untuk performa cepat dan portabilitas tanpa database eksternal.
- **Distribusi**: Satu binary terpadu / satu kontainer Docker multi-stage (konsumsi RAM < 50 MB).

---

## 🛣️ Peta Jalan Pengembangan (8 Fase)

Pengerjaan dirancang bertahap agar rapi dan tidak membebani context window:

1. **Fase 1: Project Scaffolding & Core Architecture**  
   Setup Rust Cargo workspace (`quickbox-core`, `quickbox-smtp`, `quickbox-api`, `quickbox-app`), frontend Vite/React, dan skema SQLite.
2. **Fase 2: Asynchronous SMTP Ingestion & MIME Parsing**  
   Server SMTP TCP port 25, parser MIME email multi-part, deteksi regex OTP (4–8 digit) & tautan verifikasi.
3. **Fase 3: REST & Realtime API (Axum)**  
   Endpoint inbox, pesan, lampiran, lock/unlock kata sandi (Argon2id), dan stream SSE realtime (`/api/inboxes/{name}/events`).
4. **Fase 4: Retention Worker & Abuse Mitigation**  
   Worker berkala untuk menghapus email > 30 hari, melepas kunci inbox tidak aktif, dan pembatasan laju (*rate limiting*).
5. **Fase 5: React Frontend Scaffolding & Design System**  
   Layout 3-panel responsif (Sidebar, Inbox List, Message View), tema Gelap/Terang, dan komponen UI.
6. **Fase 6: Integrasi Inbox & Realtime Sync**  
   Koneksi SSE realtime, fitur ganti inbox cepat, riwayat inbox lokal (`localStorage`), dan dialog kunci sandi.
7. **Fase 7: Message Viewer, Sandboxed Iframe & Tooling OTP**  
   Iframe sandbox aman (blokir script & pelacak luar), banner OTP 1-klik salin, tab Teks/HTML/Headers, dan unduh lampiran.
8. **Fase 8: Containerization & Deployment Documentation**  
   Dockerfile multi-stage, `docker-compose.yml`, panduan DNS MX, dan dokumentasi produksi.

---

## 📂 Struktur Berkas Dokumentasi

```
quick-box/
├── README.md                                             # Ringkasan proyek & status siap lanjut
├── docs/
│   ├── PRD.md                                            # Dokumen kebutuhan produk lengkap
│   ├── design/
│   │   └── reference-ui.jpg                              # Tangkapan layar referensi tampilan email client
│   └── superpowers/
│       └── specs/
│           └── 2026-11-04-quick-box-design.md            # Desain teknis detail, skema SQL, API & roadmap
```

---

## 🚀 Cara Melanjutkan di Sesi Berikutnya

Ketika Anda siap melanjutkan pengerjaan proyek ini, cukup berikan perintah:
> *"Lanjutkan implementasi Quick-Box mulai dari Fase 1 (Project Scaffolding & Core Architecture) sesuai dokumen spesifikasi teknis."*

