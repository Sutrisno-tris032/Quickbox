# PRD: Layanan Inbox Email Sementara untuk Developer (Open Source) - Quick-Box

> **Status:** Draf 2 (Diselaraskan dengan Keputusan Arsitektur & Roadmap), 2026-11-09.  
> **Nama proyek:** **Quick-Box** (Quickbox) [Keputusan].  
> **Cara membaca dokumen ini:** setiap keputusan diberi label:
> - **[Keputusan]** sudah diputuskan oleh pemilik proyek.
> - **[Usulan]** pilihan awal yang diusulkan, boleh diubah lewat diskusi tim.
> - **[Terjawab]** poin diskusi/pertanyaan terbuka yang telah disepakati solusinya.

---

## 1. Ringkasan

Proyek ini adalah layanan **inbox email sementara** yang bisa dipakai siapa saja tanpa mendaftar, mirip Yopmail. Pengguna cukup mengetik nama alamat apa saja di domain layanan, lalu langsung melihat email yang masuk ke alamat itu.

Pengguna utamanya adalah **developer dan QA yang menguji aplikasi**: misalnya mendaftar akun di aplikasi yang sedang dikembangkan, lalu memeriksa email verifikasi, reset kata sandi, kode OTP, atau notifikasi yang dikirim aplikasi tersebut.

Layanan ini **dijalankan sendiri oleh pemilik proyek sebagai layanan publik di satu domain utama** [Keputusan], dan kodenya dibuka sebagai open source.

## 2. Latar belakang dan masalah

Pemilik proyek memakai Yopmail untuk menguji email dari aplikasi. Cara kerjanya sudah cukup baik, tetapi ada keterbatasan yang ingin diatasi [Keputusan]:

1. Tidak dirancang untuk komunikasi dua arah (hanya menerima untuk MVP; balasan menyusul).
2. Masa simpan email singkat (Yopmail 8 hari vs Quick-Box 30 hari).
3. Tidak ada kata sandi, sehingga siapa pun yang tahu alamat bisa membaca inbox publik.

Selain itu, pemilik ingin memiliki versi sendiri yang kodenya terbuka, berbasis **Rust** untuk kehandalan, efisiensi resource, dan keamanan memori tingkat tinggi, serta **React** di antarmuka web [Keputusan].

## 3. Tujuan, bukan tujuan, dan anti-tujuan

### Tujuan
- Developer bisa menguji alur email dengan cepat: ketik alamat, kirim email dari aplikasi, lihat hasilnya secara realtime.
- Pengalaman pakai setara atau lebih nyaman daripada Yopmail, tanpa langkah daftar akun.
- Inbox bisa dikunci dengan kata sandi dan disimpan lebih lama (30 hari secara default).
- Deteksi otomatis kode OTP (4–8 digit) dan link verifikasi dengan tombol salin satu-klik.
- Kode terbuka (*open source*) dan mudah dipasang orang lain di server mereka sendiri dalam satu container Docker (*single-process monolith*).

### Bukan tujuan (untuk versi pertama)
- **Mengirim atau membalas email** (menyusul via relay SMTP di tahap lanjutan).
- API publik dan fitur tes otomatis/CI (menyusul di v2).
- Menjadi layanan email pribadi jangka panjang atau pengganti Gmail.
- Aplikasi mobile native.

### Anti-tujuan (hal yang membuat proyek ini gagal walau secara teknis berjalan)
- Dipakai sebagai alat spam atau penipuan, atau domainnya masuk daftar hitam.
- Email pengguna yang terkunci bisa dibaca orang lain (kebocoran privasi).
- Terlalu merepotkan sehingga developer kembali ke Yopmail (misalnya memaksa login).
- Email berbahaya (script, pelacak) mengeksekusi sesuatu di browser pengguna.
- Perebutan nama inbox populer (*permanent squatting*) oleh pengguna nakal.

## 4. Pengguna dan kasus penggunaan

**Pengguna utama:** developer dan penguji (QA) yang menguji aplikasi, termasuk mencoba aplikasi milik orang lain.

| # | Kasus penggunaan |
|---|---|
| U1 | Developer mendaftar di aplikasi dengan `budi-test@domain` lalu membuka inbox untuk melihat email verifikasi. |
| U2 | Developer mencari kode OTP 6 digit atau link verifikasi di email dan menyalinnya dengan satu klik. |
| U3 | Developer ingin inbox ujinya tidak bisa dibaca orang lain, sehingga mengunci inbox dengan kata sandi. |
| U4 | Developer membuka kembali inbox yang sama beberapa hari kemudian dan email lama masih ada (retensi 30 hari). |
| U5 | Developer memeriksa HTML, versi teks, dan header (SPF/DKIM) sebuah email untuk men-debug template. |
| U6 | Developer memeriksa dan mengunduh lampiran yang dikirim aplikasinya. |

## 5. Pembeda dari Yopmail dan layanan sejenis

| Fitur | Yopmail dan sejenisnya | Quick-Box |
|---|---|---|
| Kata sandi inbox | Umumnya tidak ada | Ada, opsional (Argon2id) |
| Masa simpan | Singkat (Yopmail 8 hari) | 30 hari default, dapat diatur [Keputusan] |
| Kode sumber | Tertutup | Open source (Rust + React) |
| Self-host | Tidak bisa | Bisa (Satu image Docker monolitik) |
| Deteksi OTP/tautan | Tidak ada | Ada (Ekstraksi regex instan + tombol salin) |
| Balasan (tahap berikutnya) | Tidak ada | Direncanakan (via relay SMTP) |

---

## 6. Ruang lingkup versi pertama (MVP)

Prioritas: **P0** wajib, **P1** sebaiknya ada, **P2** boleh menyusul.

### Fitur

| ID | Prioritas | Kebutuhan |
|---|---|---|
| FR-01 | P0 | Server SMTP menerima email untuk `apa-saja@domain` tanpa pendaftaran. |
| FR-02 | P0 | Web inbox: pengguna mengetik nama alamat lalu melihat daftar email. |
| FR-03 | P0 | Tampilan 3 panel modern (Sidebar navigasi, daftar inbox di tengah, pembaca pesan di kanan) mengikuti acuan desain dan prototipe. |
| FR-04 | P0 | Email baru muncul otomatis tanpa memuat ulang halaman via Server-Sent Events (SSE) realtime. |
| FR-05 | P0 | Isi email ditampilkan dalam tab **HTML Preview**, **Plain Text**, dan **Raw Headers**. HTML dirender di iframe ber-sandbox dengan blokir gambar eksternal default. |
| FR-06 | P0 | Kunci inbox dengan kata sandi (Argon2id). Inbox terkunci hanya bisa dibuka dengan sandi tersebut. |
| FR-07 | P0 | Retensi: email dihapus otomatis setelah 30 hari. Kunci pada inbox yang tidak aktif 30 hari otomatis terlepas. |
| FR-08 | P0 | Pembatasan penyalahgunaan: batas ukuran email (10 MB), batas jumlah per inbox (200), batas laju permintaan per IP. |
| FR-09 | P1 | Deteksi otomatis kode OTP (4–8 digit) dan link verifikasi dengan tombol Salin satu-klik. |
| FR-10 | P1 | Tampilkan dan unduh lampiran berkas. |
| FR-11 | P1 | Cari email (pengirim, subjek, isi), favorit, tandai belum dibaca, hapus dan pulihkan. |
| FR-12 | P1 | Mode gelap (*dark mode*) dan tata letak satu kolom responsif di layar sempit/mobile. |
| FR-13 | P1 | Pemasangan dengan Docker multi-stage (satu perintah) di VPS kecil (RAM < 50MB). |
| FR-14 | P2 | Daftar "inbox saya / recent inboxes" untuk berpindah cepat antar-inbox yang pernah dibuka (disimpan di browser via `localStorage`). |

### Tombol yang tampil tetapi nonaktif
**Tulis**, **Balas**, **Terkirim**, dan **Draf** tampil dengan label "segera hadir" [Keputusan; pilihan awal untuk MVP].

---

## 7. Perilaku inti yang harus jelas

**Membuka inbox.** Mengetik nama alamat membuat inbox secara otomatis saat email pertama masuk atau saat dibuka. Inbox yang belum dikunci terbuka untuk siapa pun yang tahu namanya (sama seperti Yopmail). Penjelasan ini tertera jelas di antarmuka.

**Mengunci inbox.** Pemilik pertama yang menetapkan kata sandi menjadi pemilik kunci. Inbox terkunci hanya bisa dibuka dengan sandi tersebut. Kata sandi disimpan sebagai hash Argon2id [Keputusan].
- *Pencegahan Pembajakan Pesan Lama:* Saat suatu inbox publik pertama kali dikunci dengan kata sandi, email publik lama yang sudah ada di dalamnya otomatis dibersihkan (dihapus) sehingga penyerang tidak bisa mengunci inbox orang lain untuk mengintip kode verifikasi sebelumnya [Keputusan].
- *Pelepasan Kunci Mandiri:* Pengguna yang sedang membuka sesi kunci dapat menghapus kata sandi (`DELETE /api/inboxes/{name}/lock`) untuk mengembalikan inbox ke status publik [Keputusan].
- *Tidak ada pemulihan lupa sandi di MVP.* Hal ini dinyatakan jelas di UI.

**Pencegahan Perebutan Nama (Anti-Squatting) & Retensi.** 
- Waktu aktivitas pengguna web (`last_user_activity_at`) dicatat terpisah dari waktu email masuk (`last_received_at`).
- Jika sebuah inbox terkunci tidak pernah dibuka atau diakses oleh pengguna web selama 30 hari berturut-turut, kunci sandi otomatis dibatalkan (*auto-release*), seluruh email lama dibersihkan, dan alamat inbox kembali menjadi publik. Mengirim email otomatis (ping/spam) tidak akan memperpanjang masa kunci [Terjawab].

**Keamanan isi email.** Isi HTML dirender secara eksklusif di dalam `iframe` dengan atribut `sandbox="allow-popups"` (tanpa `allow-scripts` dan tanpa `allow-same-origin`). Gambar dan pelacak jarak jauh diblokir secara default dengan opsi tombol "Muat Gambar Luar".

---

## 8. Persyaratan non-fungsional

| Area | Persyaratan |
|---|---|
| Kinerja | Email yang diterima muncul di inbox yang sedang dibuka dalam waktu kurang dari 2 detik. |
| Ukuran & Resource | Batas email 10 MB per pesan; konsumsi RAM server monolitik di bawah 50 MB. |
| Keamanan | Hash sandi Argon2id, iframe sandbox anti-XSS, rate limiting per IP, Content Security Policy (CSP). |
| Privasi | Tanpa pelacak pihak ketiga; log server tidak mencatat isi tubuh email. |
| Ketersediaan & Portabilitas | Satu binary mandiri hasil kompilasi Rust dan satu image Docker multi-stage. |
| Aksesibilitas | Navigasi keyboard, kontras memadai, dukungan penuh tema Gelap & Terang. |
| Bahasa | Antarmuka berbahasa Indonesia terlebih dahulu, struktur teks disiapkan untuk i18n. |

---

## 9. Arsitektur dan teknologi

| Komponen | Pilihan | Status |
|---|---|---|
| Backend | **Rust** (Server SMTP + API Axum + SSE realtime + worker retensi dalam satu proses monolitik) | [Keputusan Pemilik Proyek] |
| Frontend | **React** (Vite + TypeScript + Tailwind CSS) | [Keputusan Pemilik Proyek] |
| Library SMTP & MIME | Tokio TCP listener (RFC 5321) + `mail-parser` | [Keputusan] |
| Database | **SQLite** (mode Write-Ahead Logging / WAL) via `sqlx` | [Keputusan] |
| Realtime | **Server-Sent Events (SSE)** via Axum (sinyal pemicu ringan `new_email`, isi diambil terotentikasi) | [Keputusan] |
| Penyimpanan lampiran | Disk lokal (`/data/attachments/`) | [Keputusan] |
| Pemasangan | Docker / Docker Compose (Multi-stage build) | [Keputusan] |

```
Pengirim email ──SMTP:25──▶ [ Listener SMTP Tokio (Rust) ] ──▶ [ SQLite (WAL) + Disk Lampiran ]
                                           │                                        │
Browser (React) ◀──HTTP REST + SSE────────▶ [ Server API Axum (Rust) ] ◀────────────┘
                                                   │
                                   [ Background Worker (Retensi 30 Hari) ]
```

### Garis besar API
- `GET /api/inboxes/{name}/status`: informasi status kunci, jumlah pesan, dan unread.
- `POST /api/inboxes/{name}/lock`: mengunci inbox dengan kata sandi (Argon2id).
- `POST /api/inboxes/{name}/unlock`: memverifikasi sandi dan mengembalikan token sesi.
- `DELETE /api/inboxes/{name}/lock`: menghapus kunci sandi dan mengembalikan inbox ke publik.
- `GET /api/inboxes/{name}/messages`: daftar email (membutuhkan token jika terkunci).
- `GET /api/inboxes/{name}/messages/{id}`: detail pesan lengkap (HTML, teks, header, lampiran).
- `DELETE /api/inboxes/{name}/messages/{id}`: hapus pesan / pindahkan ke tempat sampah.
- `GET /api/inboxes/{name}/attachments/{id}`: unduh berkas lampiran.
- `GET /api/inboxes/{name}/events`: aliran Server-Sent Events (SSE) notifikasi pesan baru realtime.

---

## 10. Infrastruktur dan DNS

- **Domain:** Single domain terkonfigurasi via environment variable `DOMAIN`. MX record mengarah ke IP publik server. Email untuk domain asing ditolak dengan SMTP `550 5.1.1 Mailbox Unrecognized`.
- **VPS & Port 25 Inbound:**
  - Penerimaan email langsung dari penyedia publik (Gmail, Outlook, Yahoo) **wajib menggunakan port TCP 25** dan record DNS MX publik yang valid. Penyedia email internet tidak mendukung pengiriman antar-MTA ke port kustom.
  - Mayoritas ISP rumahan/kantor memblokir port 25 masuk dan keluar untuk mencegah spam. Karena itu, penerimaan email dari Gmail membutuhkan VPS publik (Hetzner, Linode, DigitalOcean, dsb.) dengan port 25 inbound yang aktif.
  - Untuk pengembangan lokal (*local development*), server dapat dijalankan di port alternatif (misal `2525`) di mana aplikasi penguji diarahkan ke `localhost:2525` tanpa otentikasi/TLS.
- **Reverse Proxy:** Port 80/443 digunakan untuk antarmuka web dan API melalui reverse proxy bawaan atau Caddy/Nginx.
- **Relay Outbound (Tahap Berikutnya):** Pengiriman email (post-MVP) menggunakan relay (seperti Amazon SES) di port 587 untuk menjaga reputasi IP.

---

## 11. Desain UI & Prototipe

Acuan visual dan interaksi ada di folder `docs/design/`:
1. `docs/design/reference-ui.jpg`: Desain referensi klien email modern 3-panel (Sidebar navigasi, daftar email, dan pembaca detail).
2. `docs/design/prototype-inbox.html`: Prototipe interaktif mandiri yang memperagakan tata letak, alur kunci sandi, ekstraksi OTP, dan tab rendering.

---

## 12. Peta Jalan Pengembangan Bertahap (8 Fase)

Pengerjaan dipecah menjadi 8 fase terstruktur (rincian lengkap di `docs/superpowers/specs/2026-11-04-quick-box-design.md`):

1. **Fase 1: Project Scaffolding & Core Architecture** — Setup Rust Cargo workspace (`quickbox-core`, `quickbox-smtp`, `quickbox-api`, `quickbox-app`), frontend Vite/React, dan skema SQLite WAL.
2. **Fase 2: Asynchronous SMTP Ingestion & MIME Parsing** — Listener SMTP Tokio port 25, parser MIME `mail-parser`, ekstraksi regex OTP & link verifikasi.
3. **Fase 3: REST & Realtime API (Axum)** — Endpoint inbox, pesan, autentikasi sesi kunci sandi, dan aliran SSE.
4. **Fase 4: Retention Worker & Abuse Mitigation** — Worker pembersihan 30 hari, auto-release kunci inbox tidak aktif, dan rate limiting per IP.
5. **Fase 5: React Frontend Scaffolding & Design System** — Layout 3-panel responsif (Tailwind CSS, tema gelap/terang, komponen UI).
6. **Fase 6: Integrasi Inbox & Realtime Sync** — Integrasi SSE frontend, riwayat inbox lokal, dialog modal kunci sandi.
7. **Fase 7: Message Viewer, Sandboxed Iframe & Tooling OTP** — Iframe sandbox aman, blokir pelacak luar, tombol salin OTP 1-klik, tab HTML/Teks/Header.
8. **Fase 8: Containerization & Deployment Documentation** — Multi-stage Dockerfile, docker-compose.yml, panduan konfigurasi DNS MX.

---

## 13. Resolusi Pertanyaan Terbuka

1. **Nama proyek:** **Quick-Box** (Quickbox). [Terjawab]
2. **Perebutan nama inbox:** Auto-release kunci setelah 30 hari tanpa aktivitas pengguna web (`last_user_activity_at`). [Terjawab]
3. **Lupa kata sandi:** Tidak ada pemulihan di MVP. Jika lupa sandi, inbox akan ter-reset otomatis menjadi publik setelah 30 hari tidak aktif. [Terjawab]
4. **Lisensi open source:** MIT License. [Usulan Disepakati]
5. **Batas nyata:** Ukuran email 10 MB, maksimal 200 email per inbox. [Terjawab]
6. **Arsitektur:** Monolitik proses tunggal di Rust (`tokio` + `axum` + SQLite) dan frontend React. [Terjawab]

---

## 14. Struktur Proyek

```
quick-box/
├── Cargo.toml                       # Rust Workspace definition
├── crates/
│   ├── quickbox-core/               # Model data, skema SQLite, konfigurasi
│   ├── quickbox-smtp/               # Listener SMTP Tokio & MIME parser
│   ├── quickbox-api/                # Axum REST API & SSE handler
│   └── quickbox-app/                # Main entrypoint executable
├── frontend/                        # React 19 + TypeScript + Tailwind CSS (Vite)
├── docs/
│   ├── design/
│   │   ├── prototype-inbox.html     # Prototipe interaktif
│   │   └── reference-ui.jpg         # Referensi visual klien email
│   └── superpowers/
│       └── specs/
│           └── 2026-11-04-quick-box-design.md # Desain teknis detail
├── PRD.md                           # Dokumen PRD ini
└── README.md                        # Panduan instalasi dan pengembangan
```
