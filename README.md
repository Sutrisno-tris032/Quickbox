# Quick-Box 📬

> **Layanan Inbox Email Sementara untuk Developer (Open Source & Self-Hostable)**  
> Terinspirasi dari kesederhanaan Yopmail, diperkaya dengan keamanan kunci kata sandi (Argon2id), masa simpan 30 hari, ekstraksi OTP instan, dan arsitektur monolitik modern berbasis **Rust** dan **React**.

---

## ✨ Fitur Utama

- ⚡ **Tanpa Pendaftaran**: Ketik nama alamat apa saja (contoh: `budi-test@domain.com`) dan langsung terima email secara realtime.
- 🔒 **Kunci Kata Sandi Opsional**: Kunci inbox pengujian Anda dengan kata sandi (Argon2id). Saat dikunci, email publik sebelumnya otomatis dibersihkan demi privasi.
- 🕒 **Retensi 30 Hari & Anti-Squatting**: Email disimpan selama 30 hari. Kunci pada inbox yang tidak pernah dibuka pengguna web selama 30 hari otomatis terlepas sehingga alamat dapat digunakan kembali.
- 🔑 **Deteksi Otomatis Kode OTP**: Kode verifikasi 4–8 digit langsung terdeteksi dengan tombol salin satu-klik.
- 🛡️ **Viewer Terisolasi Aman**: Email HTML dirender di dalam `iframe` dengan sandbox ketat (`allow-popups` tanpa `allow-scripts` atau `allow-same-origin`) dan pelacak gambar luar diblokir secara default.
- 🚀 **Single-Process Monolith**: Server SMTP (port 25), Web API (Axum port 8080), worker retensi berkala, dan UI React berjalan dalam **satu proses / satu kontainer Docker** (penggunaan RAM < 50 MB).

---

## 🏗️ Arsitektur Sistem

```
[ Pengirim Email ] ──SMTP:25──▶ [ Listener SMTP Tokio (Rust) ] ──▶ [ SQLite (WAL) + Disk Lampiran ]
                                           │                                        │
[ Browser (React) ] ◀──HTTP REST + SSE────▶ [ Server API Axum (Rust) ] ◀────────────┘
                                                   │
                                   [ Background Worker (Retensi 30 Hari) ]
```

---

## 🚀 Menjalankan dengan Docker (Paling Mudah)

```bash
# 1. Clone repository
git clone https://github.com/Sutrisno-tris032/Quickbox.git
cd Quickbox

# 2. Jalankan satu perintah
docker compose up -d
```

Buka peramban Anda di **`http://localhost:8080`**.

---

## 💻 Pengembangan Lokal (Development)

### Prasyarat
- **Rust Toolchain**: `rustup` & `cargo` (v1.80+)
- **Bun**: v1.2+ (atau Node.js 20+)

### Menjalankan Backend (Rust)
```bash
# Jalankan seluruh unit & integration test
cargo test

# Jalankan server monolith backend
cargo run -p quickbox-app
```
*Backend akan berjalan di port `8080` untuk Web/API dan port `25` (atau port custom via env) untuk SMTP.*

### Menjalankan Frontend (React + Vite)
```bash
cd frontend
bun install
bun run dev
```
*Frontend akan berjalan di `http://localhost:5173` dan otomatis mem-proxy panggilan `/api` ke backend `http://localhost:8080`.*

---

## ⚙️ Variabel Lingkungan (Configuration)

| Variabel | Default | Penjelasan |
|---|---|---|
| `DOMAIN` | `quickbox.dev` | Domain utama layanan (email ke domain lain ditolak di SMTP 550) |
| `SMTP_PORT` | `25` | Port listener penerima email SMTP |
| `HTTP_PORT` | `8080` | Port HTTP Web UI & REST/SSE API |
| `DATABASE_URL` | `sqlite://quickbox.db` | URL koneksi basis data SQLite |
| `ATTACHMENTS_DIR` | `./data/attachments` | Direktori penyimpanan berkas lampiran |
| `FRONTEND_DIST` | `./frontend/dist` | Lokasi berkas static build React |
| `RETENTION_DAYS` | `30` | Batas retensi email dan pelepasan kunci tidak aktif (hari) |
| `JWT_SECRET` | *(string acak)* | Kunci rahasia HMAC penandatanganan token sesi inbox |

---

## 📁 Struktur Repositori

```
quick-box/
├── Cargo.toml                       # Rust multi-crate workspace
├── Dockerfile                       # Multi-stage production build (Bun + Rust + Debian)
├── docker-compose.yml               # File orkestrasi Docker siap pakai
├── crates/
│   ├── quickbox-core/               # Model data, skema SQLite, migrasi, auth Argon2id, worker retensi
│   ├── quickbox-smtp/               # Inbound SMTP RFC 5321 listener, MIME parser, regex OTP
│   ├── quickbox-api/                # Axum REST API, middleware auth, Server-Sent Events (SSE)
│   └── quickbox-app/                # Entrypoint executable server monolitik terpadu
├── frontend/                        # React 19 + TypeScript + Tailwind CSS (Vite)
│   ├── src/
│   │   ├── components/layout/       # 3-pane responsive layout shell
│   │   ├── components/inbox/        # Sidebar navigasi, MessageList, Modal kunci sandi
│   │   ├── components/viewer/       # MessageDetail, OtpBanner, Iframe sandbox, Lampiran
│   │   └── hooks/                   # useInbox & useRealtimeEvents (SSE)
├── docs/
│   ├── design/                      # Aset prototipe interaktif & gambar referensi UI
│   └── superpowers/
│       ├── specs/                   # Dokumen desain arsitektur teknis
│       └── plans/                   # Rencana eksekusi implementasi TDD
└── PRD.md                           # Dokumen spesifikasi kebutuhan produk (PRD)
```

---

## 📄 Lisensi
Didistribusikan di bawah lisensi **MIT License**.
