use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::TcpStream;

async fn read_response(reader: &mut BufReader<OwnedReadHalf>) -> String {
    let mut full = String::new();
    let mut line = String::new();
    loop {
        line.clear();
        reader.read_line(&mut line).await.unwrap();
        full.push_str(&line);
        if line.len() >= 4 && &line[3..4] == " " {
            break;
        }
    }
    full
}

#[tokio::test]
async fn test_smtp_rfc5321_handshake() {
    let server = quickbox_smtp::SmtpServer::bind("127.0.0.1:0", "quickbox.dev", 10_000_000)
        .await
        .unwrap();
    let addr = server.local_addr().unwrap();
    tokio::spawn(server.serve());

    let stream = TcpStream::connect(addr).await.unwrap();
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    // 1. Read 220 Greeting Banner
    let resp = read_response(&mut reader).await;
    assert!(resp.starts_with("220"), "Expected 220 greeting, got: {}", resp);

    // 2. Send EHLO
    writer.write_all(b"EHLO client.test\r\n").await.unwrap();
    let resp = read_response(&mut reader).await;
    assert!(resp.starts_with("250"), "Expected 250 for EHLO, got: {}", resp);

    // 3. Send MAIL FROM
    writer.write_all(b"MAIL FROM:<sender@example.com>\r\n").await.unwrap();
    let resp = read_response(&mut reader).await;
    assert!(resp.starts_with("250"), "Expected 250 for MAIL FROM, got: {}", resp);

    // 4. Send RCPT TO for valid domain -> 250 OK
    writer.write_all(b"RCPT TO:<budi-test@quickbox.dev>\r\n").await.unwrap();
    let resp = read_response(&mut reader).await;
    assert!(resp.starts_with("250"), "Expected 250 for valid domain recipient, got: {}", resp);

    // 5. Send RCPT TO for foreign domain -> 550 Rejected
    writer.write_all(b"RCPT TO:<someone@gmail.com>\r\n").await.unwrap();
    let resp = read_response(&mut reader).await;
    assert!(resp.starts_with("550"), "Expected 550 for foreign domain recipient, got: {}", resp);

    // 6. QUIT
    writer.write_all(b"QUIT\r\n").await.unwrap();
    let resp = read_response(&mut reader).await;
    assert!(resp.starts_with("221"), "Expected 221 on QUIT, got: {}", resp);
}
