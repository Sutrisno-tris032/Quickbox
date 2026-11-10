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

_Backend akan berjalan di port `8080` untuk Web/API dan port `25` (atau port custom via env) untuk SMTP._

### Menjalankan Frontend (React + Vite)

```bash
cd frontend
bun install
bun run dev
```

_Frontend akan berjalan di `http://localhost:5173` dan otomatis mem-proxy panggilan `/api` ke backend `http://localhost:8080`._

---

## 🌐 Catatan: Pengujian Lokal vs Menerima Email dari Internet (Gmail, Outlook, dll.)

### Mengapa Email dari Gmail Belum Masuk ke Laptop/Localhost?
Jika Anda mencoba mengirim email dari **Gmail, Yahoo, atau Outlook** ke Quick-Box yang berjalan di komputer lokal, email tidak akan sampai karena dua hal mendasar dalam protokol email internet:

1. **Port 25 Wajib untuk Pertukaran Email Antar-Server (MTA to MTA):**
   - Seluruh server email internet (seperti Google Mail) **hanya mengirim ke TCP Port 25**.
   - Gmail tidak mendukung pengiriman ke port non-standar (seperti 2525 atau 8080).
   - Hampir seluruh penyedia internet rumahan/kantor (IndiHome, Biznet, FirstMedia, MyRepublic, dll.) memblokir Port 25 masuk dan keluar demi pencegahan penyebaran spam.
2. **Kebutuhan DNS MX Record Publik:**
   - Server Gmail membutuhkan rekaman **DNS MX (Mail Exchange)** publik untuk mencari alamat IP server tujuan dari domain penerima (misal: `test@domainanda.com`). Komputer lokal di balik NAT/router rumahan tidak memiliki IP publik statis atau DNS MX.

---

### 🚀 Menyiapkan Server Publik untuk Menerima Email dari Gmail

Untuk menjalankan Quick-Box sebagai layanan publik yang bisa menerima email langsung dari Gmail, ikuti langkah berikut:

1. **Sewa VPS (Virtual Private Server):**
   - Gunakan VPS dengan IP publik statis (contoh: Hetzner, DigitalOcean, Linode/Akamai, Vultr, atau AWS EC2).
   - **Penting:** Pastikan penyedia VPS tidak memblokir port 25 inbound (beberapa penyedia seperti AWS atau DigitalOcean memblokir port 25 secara default untuk akun baru dan memerlukan tiket permohonan pembukaan port 25).
2. **Konfigurasi DNS Domain Anda:**
   Tambahkan record berikut pada panel DNS domain Anda:
   | Tipe | Nama Host | Nilai / Target | Prioritas |
   |---|---|---|---|
   | **A** | `mail.domainanda.com` | `IP_PUBLIK_VPS` | - |
   | **MX** | `@` (atau `domainanda.com`) | `mail.domainanda.com` | `10` |
3. **Jalankan via Docker di VPS:**
   ```bash
   DOMAIN=domainanda.com SMTP_PORT=25 HTTP_PORT=8080 docker compose up -d
   ```
   *Setelah DNS terpropagasi, siapa pun dari Gmail dapat langsung mengirim email ke `apapun@domainanda.com` dan email akan masuk seketika.*

---

### 💻 Penggunaan untuk Pengujian Aplikasi Lokal (Development)

Untuk pengujian aplikasi yang sedang Anda bangun di laptop lokal tanpa VPS:
- Arahkan pengaturan SMTP client di aplikasi Anda (Laravel, Node.js/Nodemailer, Django, Go, Spring Boot, dll.) ke:
  - **SMTP Host:** `localhost` atau `127.0.0.1`
  - **SMTP Port:** `2525` *(atau `25` jika dijalankan dengan hak administrator)*
  - **Enkripsi (TLS/SSL):** `None` / `Disabled`
  - **Autentikasi (User/Pass):** Dikosongkan
- Setiap email verifikasi, OTP, dan pemberitahuan yang dikirim aplikasi Anda akan langsung ditangkap dan muncul seketika di Web UI **`http://localhost:8080`**.

---

## ⚙️ Variabel Lingkungan (Configuration)

| Variabel          | Default                | Penjelasan                                                      |
| ----------------- | ---------------------- | --------------------------------------------------------------- |
| `DOMAIN`          | `quickbox.dev`         | Domain utama layanan (email ke domain lain ditolak di SMTP 550) |
| `SMTP_PORT`       | `25`                   | Port listener penerima email SMTP                               |
| `ENABLE_SMTP`     | `true`                 | Set `false` untuk nonaktifkan SMTP listener (pakai Cloudflare Worker) |
| `HTTP_PORT`       | `8080`                 | Port HTTP Web UI & REST/SSE API                                 |
| `DATABASE_URL`    | `sqlite://quickbox.db` | URL koneksi basis data SQLite                                   |
| `ATTACHMENTS_DIR` | `./data/attachments`   | Direktori penyimpanan berkas lampiran                           |
| `FRONTEND_DIST`   | `./frontend/dist`      | Lokasi berkas static build React                                |
| `RETENTION_DAYS`  | `30`                   | Batas retensi email dan pelepasan kunci tidak aktif (hari)      |
| `JWT_SECRET`      | _(string acak)_        | Kunci rahasia HMAC penandatanganan token sesi inbox             |
| `INBOUND_SECRET`  | _(kosong = nonaktif)_  | Rahasia bersama untuk `POST /internal/inbound` (webhook penerima email dari Cloudflare Email Worker, dipakai saat SMTP langsung tidak memungkinkan misal di belakang CGNAT). Endpoint mengembalikan 404 selama variabel ini tidak diset. |

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
│   ├── cloudflare-setup.md          # 📖 Panduan lengkap self-host via Cloudflare Email Worker + Tunnel
│   └── superpowers/
│       ├── specs/                   # Dokumen desain arsitektur teknis
│       └── plans/                   # Rencana eksekusi implementasi TDD
├── cloudflare/
│   ├── email-worker.js              # Template Cloudflare Email Worker (forward SMTP → HTTP)
│   └── tunnel-config.yml            # Template konfigurasi Cloudflare Tunnel
└── PRD.md                           # Dokumen spesifikasi kebutuhan produk (PRD)
```

---

## 🏠 Self-Host di Home Server via Cloudflare (Tanpa VPS, Tanpa Port Forwarding)

Quick-Box mendukung penerimaan email langsung dari Gmail/Outlook di home server rumahan menggunakan **Cloudflare Email Routing + Email Worker + Cloudflare Tunnel** — tanpa memerlukan IP statis, tanpa port 25 terbuka di router rumah.

📖 **Baca panduan lengkapnya di [`docs/cloudflare-setup.md`](./docs/cloudflare-setup.md)**

---

## 📄 Lisensi

Didistribusikan di bawah lisensi **MIT License**.
