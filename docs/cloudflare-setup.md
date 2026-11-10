# Panduan Lengkap: Self-Host Quick-Box di Home Server via Cloudflare

Panduan ini menjelaskan cara menjalankan Quick-Box di **server rumahan (home server)** di balik koneksi internet biasa (IndiHome, Biznet, MyRepublic, dll.) menggunakan **Cloudflare Email Routing + Email Worker + Cloudflare Tunnel** — tanpa memerlukan IP publik statis, tanpa port forwarding router, dan tanpa perlu port 25 terbuka di sisi rumah.

---

## 🔄 Cara Kerja Arsitektur

```
[ Gmail / Outlook / Siapapun ]
         │
         │  SMTP port 25 (koneksi internet biasa)
         ▼
[ Cloudflare MX Server (IP publik Cloudflare) ]
         │
         │  Trigger Email Worker (edge function)
         ▼
[ Cloudflare Email Worker (email-worker.js) ]
         │
         │  HTTPS POST ke tunnel
         │  - Content-Type: message/rfc822
         │  - X-Quickbox-Secret: <token rahasia>
         │  - X-Quickbox-Rcpt: budi@domainanda.com
         ▼
[ Cloudflare Tunnel (cloudflared daemon di home server) ]
         │
         │  HTTP ke localhost
         ▼
[ Quick-Box → POST /internal/inbound ]
         │
         ├──▶ SQLite: simpan email, lampiran, OTP
         └──▶ SSE broadcast → browser realtime
```

**Keunggulan pendekatan ini:**
- ✅ Tidak butuh IP statis / port forwarding router
- ✅ Tidak butuh VPS berbayar
- ✅ Reputasi pengirim terjaga (MX Cloudflare yang bersih di mata Gmail)
- ✅ Port 25 tidak perlu dibuka di router rumah sama sekali
- ✅ Semua traffic terenkripsi TLS end-to-end

---

## 📋 Prasyarat

- Domain yang nameserver-nya sudah dipindahkan ke Cloudflare
- Akun Cloudflare (paket Free sudah cukup)
- Home server (Raspberry Pi, PC lama, NAS, dll.) dengan Linux/Windows/macOS
- Quick-Box sudah di-build dan siap dijalankan

---

## Langkah 1 — Aktifkan Cloudflare Email Routing

1. Masuk ke **Cloudflare Dashboard** → pilih domain Anda.
2. Di menu kiri pilih **Email** → **Email Routing**.
3. Klik **Enable Email Routing**.
4. Cloudflare akan meminta Anda menambahkan beberapa DNS record MX secara otomatis — klik **Add Records**.

> **Catatan:** Setelah langkah ini, Cloudflare menjadi server MX publik untuk domain Anda. Record MX sudah ditambahkan otomatis.

---

## Langkah 2 — Buat Cloudflare Tunnel

Tunnel ini yang menghubungkan Cloudflare ke home server Anda melalui koneksi keluar (tidak butuh port forwarding).

### a. Install `cloudflared`

**Linux (Debian/Ubuntu/Raspberry Pi OS):**
```bash
curl -L https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-arm64.deb -o cloudflared.deb
sudo dpkg -i cloudflared.deb
# Untuk x86_64: ganti arm64 dengan amd64
```

**Windows:**
```powershell
winget install --id Cloudflare.cloudflared
```

### b. Login dan Buat Tunnel

```bash
# 1. Login ke akun Cloudflare (akan membuka browser)
cloudflared tunnel login

# 2. Buat tunnel dengan nama "quickbox"
cloudflared tunnel create quickbox
# Catat UUID yang muncul, contoh: a1b2c3d4-e5f6-...

# 3. Buat DNS record CNAME untuk tunnel
cloudflared tunnel route dns quickbox quickbox.domainanda.com
```

### c. Konfigurasi Tunnel

Salin file template [`cloudflare/tunnel-config.yml`](./tunnel-config.yml) dari repositori ini:

```bash
mkdir -p ~/.cloudflared
cp cloudflare/tunnel-config.yml ~/.cloudflared/config.yml
```

Edit `~/.cloudflared/config.yml` dan ganti:
- `<GANTI-DENGAN-UUID-TUNNEL-ANDA>` → UUID dari langkah sebelumnya
- `quickbox.domainanda.com` → subdomain Anda yang sebenarnya

### d. Jalankan Tunnel sebagai Service (Otomatis Aktif saat Boot)

**Linux (systemd):**
```bash
sudo cloudflared service install
sudo systemctl enable cloudflared
sudo systemctl start cloudflared
sudo systemctl status cloudflared
```

**Windows:**
```powershell
# Jalankan sebagai Administrator
cloudflared service install
```

---

## Langkah 3 — Buat Cloudflare Email Worker

### a. Buat Worker Baru

1. Di Cloudflare Dashboard → **Workers & Pages** → **Create Application** → **Create Worker**.
2. Beri nama, misalnya: `quickbox-email-forwarder`.
3. Klik **Deploy**, lalu klik **Edit Code**.
4. Hapus semua kode default dan tempel seluruh isi file [`cloudflare/email-worker.js`](./email-worker.js) dari repositori ini.
5. Klik **Save and Deploy**.

### b. Tambahkan Environment Variables dan Secrets

Di halaman Worker Anda → **Settings** → **Variables and Secrets**:

| Nama | Tipe | Nilai |
|---|---|---|
| `QUICKBOX_URL` | Environment Variable | `https://quickbox.domainanda.com` |
| `INBOUND_SECRET` | Secret (terenkripsi) | String acak panjang, contoh: `aBcDeFgH1234567890xYzQrStUvWxYz` |

> **Penting:** Nilai `INBOUND_SECRET` di Worker harus **sama persis** dengan environment variable `INBOUND_SECRET` pada server Quick-Box (lihat Langkah 4).

### c. Hubungkan Worker ke Email Routing

1. Kembali ke **Email** → **Email Routing** → **Routing Rules**.
2. Di bagian **Catch-all address**, klik **Edit**.
3. Ubah action menjadi **Send to a Worker**.
4. Pilih worker `quickbox-email-forwarder` yang baru dibuat.
5. Klik **Save**.

---

## Langkah 4 — Konfigurasi dan Jalankan Quick-Box

Buat file `.env` di direktori root Quick-Box:

```dotenv
# Domain utama layanan email
DOMAIN=domainanda.com

# SMTP listener lokal (untuk pengujian lokal, tidak wajib dipakai saat Cloudflare aktif)
# Set ke "false" untuk menonaktifkan SMTP listener jika tidak diperlukan
ENABLE_SMTP=false
SMTP_PORT=2525

# Port HTTP Web UI & API (sama dengan target di tunnel-config.yml)
HTTP_PORT=8080

# Database & penyimpanan
DATABASE_URL=sqlite:///opt/quickbox/data/quickbox.db
ATTACHMENTS_DIR=/opt/quickbox/data/attachments

# Keamanan (WAJIB diganti dengan nilai acak yang kuat)
JWT_SECRET=GANTI_DENGAN_STRING_ACAK_PANJANG_MINIMAL_32_KARAKTER
INBOUND_SECRET=GANTI_SAMA_DENGAN_YANG_DI_CLOUDFLARE_WORKER

# Retensi email (hari)
RETENTION_DAYS=30
```

Jalankan Quick-Box:

```bash
# Dengan Cargo (development)
cargo run -p quickbox-app

# Dengan Docker (production — direkomendasikan)
docker compose up -d
```

---

## Langkah 5 — Uji Coba End-to-End

### a. Verifikasi Tunnel Aktif

```bash
cloudflared tunnel info quickbox
# Pastikan status: "healthy"
```

### b. Uji API Quick-Box via Tunnel

```bash
curl https://quickbox.domainanda.com/api/inboxes/test/status
# Harus muncul: {"name":"test","is_locked":false,...}
```

### c. Kirim Email Uji dari Gmail

1. Buka **Gmail** di browser.
2. Tulis email baru ke: `test@domainanda.com`
3. Subjek: `Halo Quick-Box! OTP kamu adalah 123456`
4. Klik **Send**.

### d. Cek Hasilnya di Web UI Quick-Box

1. Buka browser → `https://quickbox.domainanda.com`
2. Ketik `test` di kotak nama inbox.
3. Email dari Gmail seharusnya muncul dalam hitungan detik, lengkap dengan banner OTP yang menampilkan kode `123456`.

---

## ⚙️ Variabel Lingkungan Tambahan (Spesifik Cloudflare)

| Variabel | Default | Penjelasan |
|---|---|---|
| `INBOUND_SECRET` | *(kosong)* | Token rahasia bersama antara Cloudflare Worker dan server. Jika kosong, endpoint `/internal/inbound` mengembalikan `404 Not Found` (nonaktif). |
| `ENABLE_SMTP` | `true` | Set `false` untuk menonaktifkan SMTP TCP listener (port 25/2525). Berguna jika email 100% diterima via Cloudflare Worker sehingga port SMTP tidak perlu dibuka. |

---

## 🔒 Pertimbangan Keamanan

- **`INBOUND_SECRET` harus kuat dan acak** — minimal 32 karakter campuran huruf, angka, dan simbol. Gunakan `openssl rand -base64 32` untuk membuatnya.
- Secret dibandingkan dengan fungsi **constant-time** di server (bukan `==` biasa) untuk mencegah timing attack.
- Endpoint `/internal/inbound` mengembalikan **401 Unauthorized** jika secret salah/tidak ada, **404 Not Found** jika `INBOUND_SECRET` belum diset sama sekali di server, **422 Unprocessable Entity** jika header `X-Quickbox-Rcpt` hilang atau domainnya bukan `DOMAIN` yang dikonfigurasi, dan **413 Payload Too Large** jika body email lebih dari 10 MB.
- Cloudflare Tunnel hanya mengizinkan koneksi masuk dari jaringan Cloudflare — tidak ada port yang perlu dibuka di router rumah.
- Gunakan **Secret** (bukan plain Environment Variable) untuk `INBOUND_SECRET` di dashboard Cloudflare Worker agar tidak terekspos di log.

---

## 🐛 Pemecahan Masalah (Troubleshooting)

| Gejala | Kemungkinan Penyebab | Solusi |
|---|---|---|
| Email dari Gmail tidak masuk | Tunnel tidak aktif | `systemctl status cloudflared` |
| Respons `404` di log Worker | `INBOUND_SECRET` belum diset di server | Set env var `INBOUND_SECRET` di server lalu restart |
| Respons `401` di log Worker | `INBOUND_SECRET` tidak cocok | Periksa nilai secret di Worker dan `.env` server sama persis |
| Respons `422` di log Worker | Header `X-Quickbox-Rcpt` hilang, atau domain penerima bukan `DOMAIN` yang dikonfigurasi | Pastikan `DOMAIN` di server sama dengan domain Cloudflare Email Routing Anda |
| Tunnel aktif tapi API tidak bisa diakses | Port 8080 Quick-Box tidak berjalan | `cargo run -p quickbox-app` atau cek log Docker |
| Catch-all tidak berjalan | Email Routing routing rule salah | Cek **Email Routing** → **Routing Rules** → Catch-all harus ke Worker |
| SSE realtime terputus-putus | Timeout tunnel terlalu pendek | Periksa `keepAliveTimeout` di `tunnel-config.yml` |

