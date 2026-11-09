/**
 * Quick-Box — Cloudflare Email Worker
 *
 * Fungsi:
 *   Menangkap email masuk via Cloudflare Email Routing (port 25 Cloudflare)
 *   dan mem-forward-nya ke home server Quick-Box melalui Cloudflare Tunnel
 *   menggunakan HTTP POST ke endpoint /api/inbound/email.
 *
 * Deployment:
 *   1. Simpan script ini di Workers & Pages > Create Application > Workers
 *   2. Tambahkan Secret: WEBHOOK_SECRET (sama dengan env var di server)
 *   3. Tambahkan var: QUICKBOX_URL (contoh: https://quickbox.domainanda.com)
 *   4. Hubungkan ke Email Routing: Konfigurasi catch-all -> "Send to a Worker"
 *
 * Docs: https://developers.cloudflare.com/email-routing/email-workers/
 */

export default {
  /**
   * @param {EmailMessage} message - Objek email masuk dari Cloudflare
   * @param {Env} env - Binding environment (secrets & vars)
   * @param {ExecutionContext} ctx
   */
  async email(message, env, ctx) {
    const QUICKBOX_URL = env.QUICKBOX_URL;
    const WEBHOOK_SECRET = env.WEBHOOK_SECRET;

    // Validasi konfigurasi
    if (!QUICKBOX_URL || !WEBHOOK_SECRET) {
      console.error("Quick-Box Worker: QUICKBOX_URL atau WEBHOOK_SECRET belum dikonfigurasi");
      message.setReject("Konfigurasi server tidak lengkap");
      return;
    }

    // Baca raw email bytes (format RFC 822 / MIME)
    const rawEmail = await new Response(message.raw).arrayBuffer();

    const targetUrl = `${QUICKBOX_URL}/api/inbound/email`;

    console.log(
      `Quick-Box Worker: Meneruskan email dari <${message.from}> ke <${message.to}> → ${targetUrl}`
    );

    let response;
    try {
      response = await fetch(targetUrl, {
        method: "POST",
        headers: {
          "Content-Type": "message/rfc822",
          "X-Webhook-Secret": WEBHOOK_SECRET,
          "X-Recipient": message.to,
          "X-Sender": message.from,
        },
        body: rawEmail,
      });
    } catch (err) {
      // Jaringan gagal (tunnel down, timeout, dll.)
      console.error("Quick-Box Worker: Gagal menghubungi server:", err);
      // Jangan reject — biarkan Cloudflare retry beberapa saat kemudian
      return;
    }

    if (!response.ok) {
      const errText = await response.text();
      console.error(
        `Quick-Box Worker: Server merespons ${response.status}: ${errText}`
      );
      // Reject hanya jika 4xx (client error) supaya tidak loop retry
      if (response.status >= 400 && response.status < 500) {
        message.setReject(`Server menolak email: ${response.status} ${errText}`);
      }
      return;
    }

    const result = await response.json();
    console.log(
      `Quick-Box Worker: Email berhasil disimpan. Pesan tersimpan: ${result.messages_stored}`
    );
  },
};

