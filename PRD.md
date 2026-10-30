# PRD: Layanan Inbox Email Sementara untuk Developer (Open Source)

> **Status:** Draf 1, 2026-10-30. Nama proyek: *belum ditentukan* (sementara disebut "proyek ini").
> **Cara membaca dokumen ini:** setiap keputusan diberi label.
> - **[Keputusan]** sudah diputuskan oleh pemilik proyek.
> - **[Usulan]** pilihan awal yang diusulkan, boleh diubah lewat diskusi tim.
> - **[Terbuka]** belum diputuskan, lihat bagian 15.

---

## 1. Ringkasan

Proyek ini adalah layanan **inbox email sementara** yang bisa dipakai siapa saja tanpa mendaftar, mirip Yopmail. Pengguna cukup mengetik nama alamat apa saja di domain layanan, lalu langsung melihat email yang masuk ke alamat itu.

Pengguna utamanya adalah **developer yang menguji aplikasi**: misalnya mendaftar akun di aplikasi yang sedang dikembangkan, lalu memeriksa email verifikasi, reset kata sandi, atau notifikasi yang dikirim aplikasi tersebut.

Layanan ini **dijalankan sendiri oleh pemilik proyek sebagai layanan publik di satu domain** [Keputusan], dan kodenya dibuka sebagai open source.

## 2. Latar belakang dan masalah

Pemilik proyek memakai Yopmail untuk menguji email dari aplikasi. Cara kerjanya sudah cukup baik, tetapi ada keterbatasan yang ingin diatasi [Keputusan]:

1. Tidak dirancang untuk komunikasi dua arah (hanya menerima).
2. Masa simpan email singkat (8 hari).
3. Tidak ada kata sandi, sehingga siapa pun yang tahu alamat bisa membaca inbox.

Selain itu, pemilik ingin memiliki versi sendiri yang kodenya terbuka dan bisa dikembangkan bersama komunitas.

## 3. Tujuan, bukan tujuan, dan anti-tujuan

### Tujuan
- Developer bisa menguji alur email dengan cepat: ketik alamat, kirim email dari aplikasi, lihat hasilnya.
- Pengalaman pakai setara atau lebih nyaman daripada Yopmail, tanpa langkah daftar.
- Inbox bisa dikunci dengan kata sandi dan disimpan lebih lama dari 8 hari.
- Kode terbuka dan mudah dipasang orang lain di server mereka sendiri.

### Bukan tujuan (untuk versi pertama)
- **Mengirim atau membalas email** (menyusul, lihat bagian 14).
- API publik dan fitur tes otomatis/CI (menyusul).
- Menjadi layanan email pribadi jangka panjang atau pengganti Gmail.
- Aplikasi mobile native.

### Anti-tujuan (hal yang membuat proyek ini gagal walau secara teknis berjalan)
- Dipakai sebagai alat spam atau penipuan, atau domainnya masuk daftar hitam.
- Email pengguna yang terkunci bisa dibaca orang lain.
- Terlalu merepotkan sehingga developer kembali ke Yopmail (misalnya memaksa login).
- Email berbahaya (script, pelacak) mengeksekusi sesuatu di browser pengguna.

## 4. Pengguna dan kasus penggunaan

**Pengguna utama:** developer dan penguji (QA) yang menguji aplikasi, termasuk mencoba aplikasi milik orang lain.

| # | Kasus penggunaan |
|---|---|
| U1 | Developer mendaftar di aplikasi dengan `budi-test@domain` lalu membuka inbox untuk melihat email verifikasi. |
| U2 | Developer mencari kode OTP 6 digit atau link verifikasi di email dan menyalinnya dengan satu klik. |
| U3 | Developer ingin inbox ujinya tidak bisa dibaca orang lain, sehingga mengunci inbox dengan kata sandi. |
| U4 | Developer membuka kembali inbox yang sama beberapa hari kemudian dan email lama masih ada. |
| U5 | Developer memeriksa HTML, versi teks, dan header (SPF/DKIM) sebuah email untuk men-debug template. |
| U6 | Developer memeriksa lampiran yang dikirim aplikasinya. |

## 5. Pembeda dari Yopmail dan layanan sejenis

Tidak ada fitur versi pertama yang benar-benar baru. Pembedanya adalah **kombinasi**, sebagai alternatif open source yang bisa dikunci:

| | Yopmail dan sejenisnya | Proyek ini |
|---|---|---|
| Kata sandi inbox | Umumnya tidak ada | Ada, opsional |
| Masa simpan | Singkat (Yopmail 8 hari) | 30 hari secara default, dapat diatur [Usulan] |
| Kode sumber | Tertutup | Open source |
| Self-host | Tidak bisa | Bisa |
| Deteksi OTP/tautan | Tidak ada | Ada |
| Balasan (tahap berikutnya) | Tidak ada | Direncanakan |

**Catatan:** ini penilaian awal dari pengetahuan umum, belum dibandingkan satu per satu dengan layanan lain. Pembanding yang sudah ada: Mailpit/MailHog (lokal), Mailinator, mail.tm, Guerrilla Mail, MailSlurp. Tim sebaiknya mencoba beberapa sebelum finalisasi.

## 6. Ruang lingkup versi pertama (MVP)

Prioritas: **P0** wajib, **P1** sebaiknya ada, **P2** boleh menyusul.

### Fitur

| ID | Prioritas | Kebutuhan |
|---|---|---|
| FR-01 | P0 | Server SMTP menerima email untuk `apa-saja@domain` tanpa pendaftaran. |
| FR-02 | P0 | Web inbox: pengguna mengetik nama alamat lalu melihat daftar email. |
| FR-03 | P0 | Tampilan dua panel (daftar di kiri, isi di kanan) mengikuti prototipe di `docs/design/`. |
| FR-04 | P0 | Email baru muncul otomatis tanpa memuat ulang halaman (realtime). |
| FR-05 | P0 | Isi email ditampilkan dalam tab **HTML**, **Teks**, dan **Header**. HTML dirender di iframe ber-sandbox. |
| FR-06 | P0 | Kunci inbox dengan kata sandi. Inbox terkunci tidak bisa dibuka tanpa sandi. |
| FR-07 | P0 | Retensi: email dihapus otomatis setelah masa simpan (default 30 hari, dapat diatur pengelola). |
| FR-08 | P0 | Pembatasan penyalahgunaan: batas jumlah dan ukuran email per inbox, dan batas permintaan per IP. |
| FR-09 | P1 | Deteksi otomatis kode OTP 6 digit dan link verifikasi, dengan tombol Salin. |
| FR-10 | P1 | Tampilkan dan unduh lampiran. |
| FR-11 | P1 | Cari email (pengirim, subjek, isi), favorit, tandai belum dibaca, hapus dan pulihkan. |
| FR-12 | P1 | Mode gelap dan tata letak satu kolom di layar sempit. |
| FR-13 | P1 | Pemasangan dengan Docker (satu perintah) di VPS kecil. |
| FR-14 | P2 | Daftar "inbox saya" untuk berpindah cepat antar-inbox yang pernah dibuka (disimpan di browser). |

### Tombol yang tampil tetapi nonaktif
**Tulis**, **Balas**, **Terkirim**, dan **Draf** tampil dengan label "segera hadir" [Keputusan dari pemilik; pilihan awal untuk MVP].

## 7. Perilaku inti yang harus jelas

**Membuka inbox.** Mengetik nama alamat membuat inbox secara otomatis saat email pertama masuk atau saat dibuka. Inbox yang belum dikunci terbuka untuk siapa pun yang tahu namanya (sama seperti Yopmail). Ini harus dijelaskan di UI.

**Mengunci inbox.** Pemilik pertama yang menetapkan kata sandi menjadi pemilik kunci. Inbox terkunci hanya bisa dibuka dengan sandi tersebut. Kata sandi disimpan sebagai hash (argon2id atau bcrypt) [Usulan]. Tidak ada fitur lupa sandi di MVP, dan hal ini harus dinyatakan jelas di UI. Inbox terkunci yang tidak aktif tetap mengikuti aturan retensi.

> **[Terbuka]** Risiko: orang bisa mengunci nama populer seperti `test`. Perlu aturan (misalnya nama dicadangkan, kunci kedaluwarsa jika tidak aktif, atau batas jumlah kunci per IP). Lihat bagian 15.

**Retensi.** Penghapusan berdasarkan umur email (30 hari [Usulan]). Pengelola dapat mengubah angka ini lewat konfigurasi.

**Email berbahaya.** Isi HTML dirender hanya di iframe `sandbox` tanpa script. Gambar dan sumber daya jarak jauh diblokir secara default untuk mencegah pelacakan (tombol "muat gambar" opsional).

## 8. Persyaratan non-fungsional

| Area | Persyaratan |
|---|---|
| Kinerja | Email yang diterima muncul di inbox yang sedang dibuka dalam waktu kurang dari 2 detik [Usulan]. |
| Ukuran | Batas ukuran email dan lampiran per pesan (awal: 10 MB) dan jumlah email per inbox (awal: 200) [Usulan]. |
| Keamanan | Hash sandi, iframe sandbox, pembatasan laju, header keamanan (CSP), tanpa menyimpan data pribadi yang tidak perlu. |
| Privasi | Tidak ada pelacak pihak ketiga. Log tidak menyimpan isi email. |
| Ketersediaan | Satu server cukup di tahap awal. Tidak ada janji uptime tinggi. |
| Portabilitas | Satu binary backend dan satu image Docker. |
| Aksesibilitas | Dapat dioperasikan dengan keyboard, kontras memadai, mendukung mode gelap. |
| Bahasa | UI berbahasa Indonesia terlebih dahulu. Struktur teks disiapkan agar mudah diterjemahkan (i18n) [Usulan]. |

## 9. Arsitektur dan teknologi

| Komponen | Pilihan | Status |
|---|---|---|
| Backend | **Go** (server SMTP + API + realtime dalam satu proses) | [Keputusan]: pemilik menyebut Go atau Rust, dan Go dipilih sebagai rekomendasi |
| Frontend | **React** | [Keputusan]; dikerjakan bersama teman pemilik |
| Library SMTP/MIME | `go-smtp`, `go-message` (atau setara) | [Usulan] |
| Database | **SQLite** untuk awal. Pindah ke PostgreSQL bila perlu | [Usulan] |
| Realtime | WebSocket atau SSE | [Usulan] |
| Penyimpanan lampiran | Disk lokal pada MVP | [Usulan] |
| Pemasangan | Docker / docker compose | [Usulan] |

```
Pengirim email ──SMTP:25──▶ [ Server SMTP (Go) ] ──▶ [ Database + file lampiran ]
                                      │                          │
Browser (React) ◀──HTTP + WebSocket──▶ [ API (Go) ] ◀───────────┘
```

**Kontrak yang harus dijaga sejak awal:** model data menyimpan email sebagai bagian dari **percakapan (thread)** dan menyediakan tempat untuk pesan berarah "keluar", sehingga fitur balasan nanti tidak perlu membongkar skema.

### Garis besar API (arah, bukan kontrak final)
- `GET /api/inboxes/{name}/messages`: daftar email.
- `GET /api/inboxes/{name}/messages/{id}`: isi email (HTML, teks, header, lampiran).
- `DELETE /api/inboxes/{name}/messages/{id}`.
- `POST /api/inboxes/{name}/lock` dan `POST /api/inboxes/{name}/unlock`.
- `GET /api/inboxes/{name}/events`: aliran email baru (realtime).

### Model data kasar
`Inbox(name, lock_hash?, created_at)` → `Message(id, inbox, thread_id, direction[in|out], from, to, subject, text, html, headers, received_at, read, favorite, deleted)` → `Attachment(id, message, filename, size, content_type, path)`.

## 10. Infrastruktur dan DNS

- **Domain** milik pemilik, dengan record **MX** mengarah ke server.
- **VPS:** belum dibeli. Untuk *menerima* email, port 25 masuk (*inbound*) cukup dan hampir semua VPS mengizinkan. Memilih provider berdasarkan port 25 **keluar** baru penting saat fitur kirim dibangun (banyak provider memblokirnya). Referensi riset ada di bagian 16.
- Tahap kirim email (setelah MVP) disarankan lewat **relay** (misalnya Amazon SES di port 587) agar reputasi IP dan rDNS lebih mudah dikelola. Perlu juga SPF, DKIM, dan DMARC.
- Sertifikat TLS (misalnya Let's Encrypt) untuk web dan, bila dipakai, STARTTLS SMTP.

## 11. Penyalahgunaan dan risiko

| Risiko | Mitigasi awal |
|---|---|
| Domain dipakai untuk menerima email spam/penipuan | Batas laju, batas ukuran, retensi, dan fitur melapor. Perlu kebijakan penggunaan. |
| Pengiriman email dua arah dipakai sebagai spam (tahap berikutnya) | Kirim hanya lewat relay, batas ketat per inbox/IP, dan tidak boleh kirim ke daftar besar. |
| Pencurian inbox terbuka (nama mudah ditebak) | Disebutkan jelas di UI bahwa inbox terbuka itu publik. Gunakan kunci sandi. |
| Perebutan nama inbox (squatting) oleh pengunci nama populer | **[Terbuka]**, lihat bagian 15. |
| Serangan lewat isi HTML (XSS, pelacak) | Iframe sandbox, blokir sumber jarak jauh, CSP. |
| Penyimpanan penuh | Batas per inbox, penghapusan otomatis, pemantauan disk. |
| Isu hukum/privasi (isi email pihak ketiga, penghapusan atas permintaan) | **[Terbuka]**: perlu kebijakan privasi dan kontak penghapusan. |

## 12. Desain UI

Acuan visual ada di folder `docs/design/`:
- `reference-ui.jpg`: gambar referensi dari pemilik proyek (klien email modern dengan panel daftar dan panel isi).
- `prototype-inbox.html`: **prototipe sementara yang bisa dibuka langsung di browser** dan sudah disetujui pemilik sebagai arah tampilan. Datanya contoh, dan prototipe ini **bukan kode produksi**. Tim frontend boleh mengambil tata letak dan perilakunya sebagai acuan, tetapi sebaiknya membangun ulang dengan komponen React.

Perilaku yang sudah diperagakan prototipe: buka inbox dengan nama bebas, kunci sandi, daftar dan detail dua panel, tab HTML/Teks/Header, deteksi OTP dan tautan dengan tombol salin, simulasi email masuk, favorit, hapus dan pulihkan, mode gelap, dan tata letak satu kolom di layar sempit.

## 13. Kriteria keberhasilan MVP

- Email yang dikirim dari aplikasi nyata (misalnya form registrasi) ke `nama@domain` muncul di web dalam beberapa detik.
- Verifikasi OTP/link bisa dilakukan tanpa menyalin manual dari isi email panjang.
- Inbox yang dikunci tidak bisa dibuka tanpa sandi (diuji, bukan diasumsikan).
- Email lama tetap ada sampai batas retensi lalu terhapus otomatis.
- Satu orang lain berhasil memasang layanan ini di server sendiri hanya dengan mengikuti README.
- Tidak ada eksekusi script dari isi email pada pengujian keamanan dasar.

## 14. Peta jalan

1. **MVP** (dokumen ini): menerima email, web inbox, kunci sandi, retensi, pembatasan.
2. **Setelah MVP:** balasan dan pengiriman lewat relay, lalu Compose/Terkirim/Draf aktif.
3. **Berikutnya:** API untuk tes otomatis (endpoint "tunggu email", webhook, kunci API), enkripsi tambahan pada penyimpanan.

## 15. Pertanyaan terbuka

Belum diputuskan, jawab sebelum atau selama perencanaan:

1. **Nama proyek dan domain** layanan.
2. **Perebutan nama inbox:** aturan untuk mencegah nama populer dikunci orang (nama dicadangkan, batas per IP, kunci kedaluwarsa).
3. **Lupa kata sandi:** tidak ada pemulihan di MVP. Apakah ini diterima?
4. **Lisensi open source** (MIT, Apache-2.0, AGPL, dll.). AGPL melindungi dari penyedia layanan tertutup, MIT lebih bebas.
5. **Kebijakan privasi dan penghapusan atas permintaan** untuk layanan publik.
6. **Jumlah nyata batas** (ukuran email, jumlah per inbox, laju per IP) setelah ada data pemakaian.
7. **Pilihan VPS** (belum dibeli) dan anggaran bulanan.

## 16. Referensi riset

Pencarian awal (Oktober 2026) soal kebijakan port 25 outbound provider VPS. Hasilnya sumber sekunder dan klaim vendor, jadi **verifikasi sendiri** sebelum membeli (uji `nc -vz gmail-smtp-in.l.google.com 25` dari instance baru).

- Terbuka tanpa permintaan (menurut sumber): OVHcloud, Kamatera, Contabo.
- Diblokir default, dapat dibuka lewat permintaan: Hetzner (setelah tagihan pertama dibayar).
- Diblokir: Vultr, UpCloud, Azure.
- Sumber: octeth.com/vps-for-email-marketing, webshanks.com/vps-with-open-port-25, vpsforlife.com/which-vps-providers-allow-port-25.php, findstack.com/resources/port-25-blocked-vps.

---

## 17. Panduan untuk kontributor dan pengembangan dengan bantuan AI

Tim berencana memakai AI sebagai alat bantu pengembangan [Keputusan]. Aturan berikut [Usulan] menjaga kualitas dan keamanan:

**Prinsip**
- AI adalah alat bantu. **Manusia bertanggung jawab** atas setiap perubahan yang digabung. Tidak ada kode hasil AI yang digabung tanpa ditinjau oleh seorang kontributor.
- Jangan menempelkan rahasia (kata sandi, kunci API, isi `.env`, data email nyata) ke alat AI.
- Perubahan yang dibantu AI harus lolos tes yang sama dengan kode lain. Tulis tes untuk perilaku penting, terutama kunci sandi, retensi, pembatasan, dan sanitasi HTML.

**Alur kerja**
1. Satu isu atau tugas kecil per cabang dan per PR.
2. Jelaskan konteks kepada AI: rujuk bagian PRD yang relevan (misalnya FR-06) agar hasilnya sesuai kebutuhan.
3. Jalankan tes dan linter secara lokal sebelum membuka PR.
4. Di PR, tuliskan apa yang diubah dan mengapa, lalu cantumkan jika AI dipakai secara signifikan.
5. Tinjauan kode oleh minimal satu orang lain.

**Hal yang wajib diperiksa manusia pada kode hasil AI**
- Penanganan kata sandi dan sesi (tidak ada kebocoran, perbandingan yang aman).
- Sanitasi dan pengisolasian isi email (iframe sandbox, tanpa eval).
- Batas input (ukuran, jumlah, laju) dan penanganan lampiran.
- Dependensi baru: periksa kebutuhan, lisensi, dan reputasi sebelum menambah.
- Klaim yang tidak bisa diverifikasi (fungsi atau API yang tidak ada).

**Struktur proyek** (usulan, bisa disesuaikan saat perencanaan): `backend/` (Go), `frontend/` (React), `docs/`, `deploy/` (Docker). Berkas `AGENTS.md` / `CLAUDE.md` di akar repositori dapat berisi konvensi kode dan perintah tes supaya alat AI bekerja konsisten.
