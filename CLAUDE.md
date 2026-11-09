# CLAUDE.md

Panduan ini berlaku untuk seluruh repo. Rujukan lengkap kebutuhan produk ada di `PRD.md` — baca bagian yang relevan (misalnya `FR-06`) sebelum mengerjakan sebuah isu.

## Struktur repo

```
backend/   # Go: server SMTP + API + realtime dalam satu proses
frontend/  # React: web inbox
deploy/    # Docker / docker compose untuk pemasangan
docs/      # desain, riset, keputusan tambahan
```

Satu komponen, satu tempat. Jangan campur kode Go dan React di luar folder masing-masing.

## Stack dan keputusan teknis terkunci (lihat PRD bagian 9)

- **Backend:** Go. SMTP + HTTP API + realtime (WebSocket/SSE) berjalan dalam satu proses/binary.
- **Frontend:** React.
- **Database:** SQLite untuk tahap awal (migrasi ke PostgreSQL hanya jika benar-benar perlu — jangan desain ulang skema demi ini sebelum diperlukan).
- **Lampiran:** disimpan di disk lokal pada MVP.
- **Pemasangan:** satu binary backend + satu image Docker (`deploy/`).
- **Model data:** email disimpan sebagai bagian dari *thread*, dengan kolom `direction` (`in`/`out`) sejak awal — supaya fitur balasan nanti tidak perlu mengubah skema. Jangan hilangkan kolom ini meski belum dipakai.

## Prinsip kerja dengan AI

- AI adalah alat bantu. **Manusia bertanggung jawab** atas setiap perubahan yang digabung — tidak ada kode hasil AI yang digabung tanpa ditinjau kontributor manusia.
- Jangan menempelkan rahasia (kata sandi, API key, isi `.env`, data email nyata/pengguna) ke alat AI.
- Perubahan yang dibantu AI harus lolos tes yang sama dengan kode lain. **Wajib ada tes** untuk perilaku penting: kunci sandi (lock/unlock inbox), retensi, pembatasan laju/ukuran, dan sanitasi HTML.

## Alur kerja kontribusi

1. Satu isu/tugas kecil per cabang, per PR.
2. Saat meminta bantuan AI, rujuk bagian PRD yang relevan agar hasilnya sesuai kebutuhan (contoh: "implementasikan FR-06").
3. Jalankan tes dan linter secara lokal sebelum membuka PR.
4. Di deskripsi PR: jelaskan apa yang diubah dan mengapa; cantumkan jika AI dipakai secara signifikan.
5. Minimal satu tinjauan kode oleh kontributor lain sebelum digabung.

## Hal yang wajib diperiksa manusia pada kode hasil AI

- Penanganan kata sandi dan sesi: tidak ada kebocoran hash/plaintext, perbandingan sandi memakai fungsi yang aman terhadap timing attack.
- Sanitasi dan pengisolasian isi email: HTML dirender di iframe `sandbox` tanpa script, sumber daya jarak jauh (gambar, dll.) diblokir default.
- Batas input: ukuran email/lampiran, jumlah email per inbox, laju permintaan per IP.
- Dependensi baru: periksa kebutuhan, lisensi, dan reputasi sebelum ditambahkan ke `go.mod`/`package.json`.
- Klaim yang tidak bisa diverifikasi: jangan percaya referensi ke fungsi atau API yang tidak benar-benar ada — cek langsung di kode/dokumentasi resmi.

## Konvensi kode

- **Go:** `gofmt`/`go vet` bersih sebelum commit. Error ditangani secara eksplisit, tidak di-swallow. Package per domain (misalnya `internal/inbox`, `internal/mail`), bukan per jenis file.
- **Tes:** tes menyertai perubahan pada perilaku di atas (kunci sandi, retensi, pembatasan, sanitasi HTML) — bukan opsional.
- **Tidak menyimpan data pribadi yang tidak perlu** di log; log tidak boleh menyimpan isi email (lihat PRD bagian 8, baris Privasi).

## Cakupan MVP (jangan over-engineer di luar ini)

Bukan tujuan versi pertama: mengirim/membalas email, API publik untuk CI, aplikasi mobile native. Lihat PRD bagian 3 dan 14 sebelum menambah fitur di luar lingkup MVP.
