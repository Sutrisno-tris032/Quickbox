# Quick-Box Full-Stack Monolith Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build Quick-Box, an open-source, developer-focused temporary email service featuring instant public inboxes, optional Argon2id password locking, 30-day retention with auto-release anti-squatting, regex OTP extraction, and sandboxed HTML rendering in a single-process Rust monolith with a React frontend.

**Architecture:** Monolithic async application in Rust executing Tokio tasks for an inbound SMTP receiver (RFC 5321), an Axum REST/SSE HTTP API, and a background SQLite retention cleanup worker. React 19 frontend (Vite + TypeScript + Tailwind CSS) built with a modern 3-pane client layout communicating over REST and lightweight SSE event pings.

**Tech Stack:** Rust (Tokio, Axum, sqlx, mail-parser, argon2), SQLite (WAL mode), Bun, React 19, TypeScript, Tailwind CSS, Lucide icons.

**Spec:** `docs/superpowers/specs/2026-11-04-quick-box-design.md` and `PRD.md`

## Global Constraints

- Single-domain scope configured via `DOMAIN` env var (default: `quickbox.dev`); foreign domains rejected with SMTP `550 5.1.1 Mailbox Unrecognized`.
- Password hashing using Argon2id with standard OWASP parameters.
- Inactive locked inboxes automatically release and purge after 30 days without human web activity (`last_user_activity_at < now() - 30 days`).
- Public emails purged upon initial inbox lock to prevent verification code hijacking.
- HTML email rendering strictly sandboxed in `<iframe sandbox="allow-popups">` without `allow-scripts` or `allow-same-origin`.
- Remote tracking images blocked by default with user-controlled toggle.
- Monolithic process target memory footprint < 50 MB RAM on idle/moderate load.

## Review Focus

1. Inactivity auto-release does not trigger if a human user views or unlocks the inbox within 30 days.
2. Realtime SSE (`/api/inboxes/{name}/events`) emits only lightweight notification pings (`{"event": "new_email", "id": "..."}`) and never leaks email bodies or OTPs over unauthenticated connections.
3. Malicious HTML payloads with `<script>`, `onload`, or `javascript:` URLs cannot execute inside the viewer iframe.
4. Extremely large messages (>10MB) or emails with malformed MIME boundaries fail gracefully with SMTP 552 / parse fallback without crashing the Tokio runtime.
5. Locking an inbox with existing public emails purges old messages so previous secrets cannot be hijacked.

---

### Task 1: Environment Toolchain & Rust Workspace Scaffolding

**Files:**
- Create: `Cargo.toml`
- Create: `crates/quickbox-core/Cargo.toml`
- Create: `crates/quickbox-core/src/lib.rs`
- Create: `crates/quickbox-smtp/Cargo.toml`
- Create: `crates/quickbox-smtp/src/lib.rs`
- Create: `crates/quickbox-api/Cargo.toml`
- Create: `crates/quickbox-api/src/lib.rs`
- Create: `crates/quickbox-app/Cargo.toml`
- Create: `crates/quickbox-app/src/main.rs`
- Test: `crates/quickbox-app/tests/workspace_smoke.rs`

**Interfaces:**
- Consumes: Nothing
- Produces: Cargo virtual workspace compiling `quickbox-core`, `quickbox-smtp`, `quickbox-api`, and `quickbox-app`.

- [ ] **Step 1: Verify / Install Rust toolchain on host**
  If `cargo` is not in PATH, install Rust using `winget install --id Rustlang.Rustup -e --silent` or rustup-init, and ensure `cargo` is accessible.
- [ ] **Step 2: Write failing test in `crates/quickbox-app/tests/workspace_smoke.rs`**
  ```rust
  #[test]
  fn test_workspace_smoke() {
      assert_eq!(quickbox_core::version(), "0.1.0");
  }
  ```
- [ ] **Step 3: Run test to verify it fails**
  Run: `cargo test --test workspace_smoke`
  Expected: FAIL with missing module or function `version`.
- [ ] **Step 4: Implement workspace files & `version()` in `crates/quickbox-core/src/lib.rs`**
  Setup `Cargo.toml` workspace members and return `"0.1.0"`.
- [ ] **Step 5: Run test to verify it passes**
  Run: `cargo test --test workspace_smoke`
  Expected: PASS.
- [ ] **Step 6: Commit**
  ```bash
  git add Cargo.toml crates/
  git commit -m "chore: setup Rust cargo multi-crate workspace"
  ```

---

### Task 2: Core Domain Models & SQLite Migration Engine (`quickbox-core`)

**Files:**
- Create: `crates/quickbox-core/migrations/0001_initial_schema.sql`
- Create: `crates/quickbox-core/src/models.rs`
- Create: `crates/quickbox-core/src/db.rs`
- Create: `crates/quickbox-core/src/auth.rs`
- Modify: `crates/quickbox-core/src/lib.rs`
- Test: `crates/quickbox-core/tests/db_test.rs`

**Interfaces:**
- Consumes: `sqlx`, `argon2`, `chrono`, `uuid`, `serde`
- Produces: `init_db(url: &str) -> Result<SqlitePool>`, `models::{Inbox, Message, Attachment}`, `hash_password(pwd: &str) -> Result<String>`, `verify_password(hash: &str, pwd: &str) -> Result<bool>`

- [ ] **Step 1: Write the failing test in `crates/quickbox-core/tests/db_test.rs`**
  ```rust
  use quickbox_core::{auth, db, models::*};
  #[tokio::test]
  async fn test_db_inbox_and_password_lifecycle() {
      let pool = db::init_db("sqlite::memory:").await.unwrap();
      let hashed = auth::hash_password("secret123").unwrap();
      assert!(auth::verify_password(&hashed, "secret123").unwrap());
      assert!(!auth::verify_password(&hashed, "wrong").unwrap());
  }
  ```
- [ ] **Step 2: Run test to verify it fails**
  Run: `cargo test -p quickbox-core --test db_test`
  Expected: FAIL with undefined `db` or `auth`.
- [ ] **Step 3: Implement SQLite migration, models, and Argon2id in `quickbox-core`**
  - Implement `migrations/0001_initial_schema.sql` (creating `inboxes`, `messages`, `attachments` tables).
  - Implement `init_db` setting WAL mode and running `sqlx::migrate!()`.
  - Implement `hash_password` and `verify_password` using `argon2`.
- [ ] **Step 4: Run test to verify it passes**
  Run: `cargo test -p quickbox-core --test db_test`
  Expected: PASS.
- [ ] **Step 5: Commit**
  ```bash
  git add crates/quickbox-core
  git commit -m "feat(core): implement sqlite schema migrations, models, and argon2 auth"
  ```

---

### Task 3: Inbound SMTP Receiver Protocol & State Machine (`quickbox-smtp`)

**Files:**
- Create: `crates/quickbox-smtp/src/connection.rs`
- Create: `crates/quickbox-smtp/src/server.rs`
- Modify: `crates/quickbox-smtp/src/lib.rs`
- Test: `crates/quickbox-smtp/tests/smtp_handshake_test.rs`

**Interfaces:**
- Consumes: `tokio::net::TcpListener`, `quickbox-core`
- Produces: `SmtpServer::bind(addr: &str, domain: &str, max_bytes: usize) -> Result<SmtpServer>`

- [ ] **Step 1: Write the failing test in `crates/quickbox-smtp/tests/smtp_handshake_test.rs`**
  ```rust
  #[tokio::test]
  async fn test_smtp_rfc5321_handshake() {
      let server = quickbox_smtp::SmtpServer::bind("127.0.0.1:0", "quickbox.dev", 10_000_000).await.unwrap();
      let port = server.local_addr().unwrap().port();
      tokio::spawn(server.serve());

      let mut stream = tokio::net::TcpStream::connect(("127.0.0.1", port)).await.unwrap();
      // Read 220 banner, send EHLO, MAIL FROM, RCPT TO, DATA, QUIT.
      // Assert 250 for matching domain, assert 550 for foreign domain @gmail.com.
  }
  ```
- [ ] **Step 2: Run test to verify it fails**
  Run: `cargo test -p quickbox-smtp --test smtp_handshake_test`
  Expected: FAIL with `SmtpServer` not defined.
- [ ] **Step 3: Implement SMTP server connection handler in `crates/quickbox-smtp`**
  - Implement state machine: `GREETING` -> `EHLO` -> `MAIL_FROM` -> `RCPT_TO` -> `DATA` -> `QUIT`.
  - Reject recipients where domain != configured domain with `550 5.1.1 Mailbox Unrecognized`.
  - Handle dot-stuffing termination (`\r\n.\r\n`) and enforce max size limit (552).
- [ ] **Step 4: Run test to verify it passes**
  Run: `cargo test -p quickbox-smtp --test smtp_handshake_test`
  Expected: PASS.
- [ ] **Step 5: Commit**
  ```bash
  git add crates/quickbox-smtp
  git commit -m "feat(smtp): implement RFC 5321 TCP listener and domain filtering"
  ```

---

### Task 4: MIME Parsing, OTP/Link Extraction & Storage Ingestion (`quickbox-smtp` & `quickbox-core`)

**Files:**
- Create: `crates/quickbox-smtp/src/parser.rs`
- Create: `crates/quickbox-smtp/src/extractor.rs`
- Create: `crates/quickbox-smtp/src/ingest.rs`
- Modify: `crates/quickbox-smtp/src/lib.rs`
- Test: `crates/quickbox-smtp/tests/parser_and_ingest_test.rs`

**Interfaces:**
- Consumes: `mail-parser`, `regex`, `quickbox-core::db`, `quickbox-core::models`
- Produces: `parse_and_ingest(raw_bytes: &[u8], pool: &SqlitePool, attachments_dir: &Path, tx: &broadcast::Sender<NewMailEvent>) -> Result<Message>`

- [ ] **Step 1: Write failing test in `crates/quickbox-smtp/tests/parser_and_ingest_test.rs`**
  ```rust
  #[test]
  fn test_otp_and_link_extraction() {
      let text = "Your verification code is 482910. Or click https://example.com/verify?token=abc";
      let otp = quickbox_smtp::extractor::extract_otp(text);
      let links = quickbox_smtp::extractor::extract_links(text);
      assert_eq!(otp, Some("482910".to_string()));
      assert_eq!(links, vec!["https://example.com/verify?token=abc"]);
  }
  ```
- [ ] **Step 2: Run test to verify it fails**
  Run: `cargo test -p quickbox-smtp --test parser_and_ingest_test`
  Expected: FAIL with `extractor` not found.
- [ ] **Step 3: Implement MIME parser, regex extractor, and ingest pipeline**
  - Use `mail-parser` for multipart HTML/text/attachment extraction.
  - Regex for 4-8 digit codes in proximity of keywords ("code", "verification", "OTP", "PIN").
  - Save attachments to `attachments_dir`, persist message to SQLite, notify `broadcast::Sender`.
- [ ] **Step 4: Run test to verify it passes**
  Run: `cargo test -p quickbox-smtp --test parser_and_ingest_test`
  Expected: PASS.
- [ ] **Step 5: Commit**
  ```bash
  git add crates/quickbox-smtp
  git commit -m "feat(smtp): implement MIME parsing, OTP regex extractor, and DB ingestion"
  ```

---

### Task 5: Axum REST API & Sockets/SSE Realtime Stream (`quickbox-api`)

**Files:**
- Create: `crates/quickbox-api/src/routes/inboxes.rs`
- Create: `crates/quickbox-api/src/routes/messages.rs`
- Create: `crates/quickbox-api/src/routes/events.rs`
- Create: `crates/quickbox-api/src/middleware/auth.rs`
- Create: `crates/quickbox-api/src/app.rs`
- Modify: `crates/quickbox-api/src/lib.rs`
- Test: `crates/quickbox-api/tests/api_routes_test.rs`

**Interfaces:**
- Consumes: `axum`, `tower-http`, `quickbox-core`, `tokio::sync::broadcast`
- Produces: `create_router(state: AppState) -> axum::Router`

- [ ] **Step 1: Write failing test in `crates/quickbox-api/tests/api_routes_test.rs`**
  ```rust
  #[tokio::test]
  async fn test_inbox_lock_unlock_flow() {
      // 1. Lock inbox with password "pass123"
      // 2. Fetch messages without token -> expect 401 Unauthorized
      // 3. Unlock with "pass123" -> expect 200 with session token
      // 4. Fetch messages with session token -> expect 200 OK
      // 5. Delete lock -> expect 200 OK
  }
  ```
- [ ] **Step 2: Run test to verify it fails**
  Run: `cargo test -p quickbox-api --test api_routes_test`
  Expected: FAIL with router or endpoints not found.
- [ ] **Step 3: Implement Axum endpoints and SSE stream**
  - Implement `/api/inboxes/{name}/status`, `/lock`, `/unlock`, `/messages`, `/attachments/{id}`, and `/events`.
  - In `/lock`, purge existing public messages before setting `is_locked = 1`.
  - In `/events`, emit lightweight JSON events (`{"event": "new_email", "id": "..."}`).
- [ ] **Step 4: Run test to verify it passes**
  Run: `cargo test -p quickbox-api --test api_routes_test`
  Expected: PASS.
- [ ] **Step 5: Commit**
  ```bash
  git add crates/quickbox-api
  git commit -m "feat(api): implement Axum REST endpoints, auth middleware, and SSE stream"
  ```

---

### Task 6: Retention & Abuse Mitigation Worker (`quickbox-core` & `quickbox-app`)

**Files:**
- Create: `crates/quickbox-core/src/retention.rs`
- Modify: `crates/quickbox-core/src/lib.rs`
- Modify: `crates/quickbox-app/src/main.rs`
- Test: `crates/quickbox-core/tests/retention_test.rs`

**Interfaces:**
- Consumes: `SqlitePool`, `std::path::Path`
- Produces: `run_cleanup_cycle(pool: &SqlitePool, attachments_dir: &Path, retention_days: i64) -> Result<CleanupSummary>`

- [ ] **Step 1: Write failing test in `crates/quickbox-core/tests/retention_test.rs`**
  ```rust
  #[tokio::test]
  async fn test_retention_and_auto_release_inactive_lock() {
      let pool = quickbox_core::db::init_db("sqlite::memory:").await.unwrap();
      // Insert locked inbox with last_user_activity_at = 31 days ago
      // Insert message with received_at = 31 days ago
      // Run cleanup cycle
      // Assert message is deleted and inbox is_locked is reset to 0
  }
  ```
- [ ] **Step 2: Run test to verify it fails**
  Run: `cargo test -p quickbox-core --test retention_test`
  Expected: FAIL with `retention` module not found.
- [ ] **Step 3: Implement cleanup worker query & file deletion**
  - Delete messages older than `retention_days` and unlink attachment files from disk.
  - Reset `is_locked = 0`, `password_hash = NULL` for inboxes where `last_user_activity_at` exceeds retention threshold.
- [ ] **Step 4: Run test to verify it passes**
  Run: `cargo test -p quickbox-core --test retention_test`
  Expected: PASS.
- [ ] **Step 5: Commit**
  ```bash
  git add crates/quickbox-core
  git commit -m "feat(core): implement 30-day message retention and anti-squatting lock auto-release"
  ```

---

### Task 7: React Frontend Setup & Design System (`frontend/`)

**Files:**
- Create: `frontend/package.json`
- Create: `frontend/vite.config.ts`
- Create: `frontend/src/index.css`
- Create: `frontend/src/components/ui/Button.tsx`
- Create: `frontend/src/components/ui/Badge.tsx`
- Create: `frontend/src/components/ui/Modal.tsx`
- Create: `frontend/src/components/layout/AppLayout.tsx`
- Test: `frontend/src/components/ui/__tests__/Button.test.tsx`

**Interfaces:**
- Consumes: Bun, React 19, Tailwind CSS v4, Lucide icons
- Produces: Reusable UI design system and 3-pane client shell.

- [ ] **Step 1: Initialize Vite React-TS project with Bun**
  Run: `bun create vite frontend --template react-ts`
  Install dependencies: `cd frontend && bun add tailwindcss @tailwindcss/vite lucide-react clsx tailwind-merge`
- [ ] **Step 2: Write failing component test**
  Add Vitest to frontend and write test asserting `Button` renders with variant styles.
- [ ] **Step 3: Run test to verify it fails**
  Run: `cd frontend && bun test`
  Expected: FAIL with missing Button component.
- [ ] **Step 4: Implement Tailwind design system & 3-pane layout shell**
  Implement `Button`, `Badge`, `Modal`, and responsive 3-pane `AppLayout`.
- [ ] **Step 5: Run test to verify it passes**
  Run: `cd frontend && bun test`
  Expected: PASS.
- [ ] **Step 6: Commit**
  ```bash
  git add frontend/
  git commit -m "feat(frontend): scaffold React 19 + Tailwind 3-pane client layout"
  ```

---

### Task 8: Frontend State Management, Realtime SSE & Inbox Switching

**Files:**
- Create: `frontend/src/hooks/useInbox.ts`
- Create: `frontend/src/hooks/useRealtimeEvents.ts`
- Create: `frontend/src/components/inbox/Sidebar.tsx`
- Create: `frontend/src/components/inbox/MessageList.tsx`
- Create: `frontend/src/components/inbox/LockModal.tsx`
- Modify: `frontend/src/App.tsx`
- Test: `frontend/src/hooks/__tests__/useInbox.test.ts`

**Interfaces:**
- Consumes: `/api/inboxes/{name}/*` REST & SSE endpoints
- Produces: Dynamic inbox switcher, lock/unlock dialog, message list with instant filtering, and unread badges.

- [ ] **Step 1: Write failing hook test**
  Test `useInbox` fetching message lists and handling locked inbox states (prompting for password).
- [ ] **Step 2: Run test to verify it fails**
  Run: `cd frontend && bun test`
  Expected: FAIL.
- [ ] **Step 3: Implement hooks, Sidebar, MessageList, and LockModal**
  - Implement SSE connection triggering background refresh on `new_email`.
  - Store recent inboxes in `localStorage` for quick 1-click jump in sidebar.
  - Implement lock/unlock modal dialog with session token management.
- [ ] **Step 4: Run test to verify it passes**
  Run: `cd frontend && bun test`
  Expected: PASS.
- [ ] **Step 5: Commit**
  ```bash
  git add frontend/
  git commit -m "feat(frontend): implement realtime SSE sync, address switcher, and lock dialog"
  ```

---

### Task 9: Message Viewer, Sandboxed Iframe, OTP Extraction & Attachments

**Files:**
- Create: `frontend/src/components/viewer/MessageDetail.tsx`
- Create: `frontend/src/components/viewer/SandboxedHtmlFrame.tsx`
- Create: `frontend/src/components/viewer/OtpBanner.tsx`
- Create: `frontend/src/components/viewer/AttachmentList.tsx`
- Modify: `frontend/src/App.tsx`
- Test: `frontend/src/components/viewer/__tests__/SandboxedHtmlFrame.test.tsx`

**Interfaces:**
- Consumes: Message entity from `quickbox-core`
- Produces: Secure email detail view with 1-click OTP copy, sandboxed iframe, tab switcher (HTML/Text/Headers), and attachment downloads.

- [ ] **Step 1: Write failing test in `SandboxedHtmlFrame.test.tsx`**
  Verify iframe has strict `sandbox="allow-popups"` attribute and blocks scripts.
- [ ] **Step 2: Run test to verify it fails**
  Run: `cd frontend && bun test`
  Expected: FAIL.
- [ ] **Step 3: Implement MessageDetail, SandboxedHtmlFrame, OtpBanner, and AttachmentList**
  - Highlight detected OTP code with prominent copy button and toast feedback.
  - Toggle for loading remote external images (blocked by default to prevent tracking).
  - Tabs: HTML Preview, Plain Text, and Raw Headers.
- [ ] **Step 4: Run test to verify it passes**
  Run: `cd frontend && bun test`
  Expected: PASS.
- [ ] **Step 5: Commit**
  ```bash
  git add frontend/
  git commit -m "feat(frontend): implement secure sandboxed email reader, OTP banner, and headers tab"
  ```

---

### Task 10: Monolith Integration, Production Build & Docker Packaging

**Files:**
- Modify: `crates/quickbox-app/src/main.rs`
- Create: `Dockerfile`
- Create: `docker-compose.yml`
- Modify: `README.md`
- Test: `crates/quickbox-app/tests/e2e_integration_test.rs`

**Interfaces:**
- Consumes: `quickbox-core`, `quickbox-smtp`, `quickbox-api`, frontend compiled static assets (`dist/`)
- Produces: Single executable binary and single container deployment.

- [ ] **Step 1: Write failing end-to-end integration test**
  ```rust
  #[tokio::test]
  async fn test_full_e2e_monolith_flow() {
      // 1. Start monolith (SMTP on random port, Axum on random port)
      // 2. Send email via SMTP TCP client
      // 3. Query HTTP API /api/inboxes/{name}/messages
      // 4. Verify message received with extracted OTP
  }
  ```
- [ ] **Step 2: Run test to verify it fails**
  Run: `cargo test -p quickbox-app --test e2e_integration_test`
  Expected: FAIL.
- [ ] **Step 3: Implement main orchestrator, static asset serving, and Dockerfile**
  - In `crates/quickbox-app/src/main.rs`, spawn SMTP server, background retention worker, and Axum HTTP server with static file fallback.
  - Create multi-stage `Dockerfile` (Stage 1: build frontend with Bun, Stage 2: build Rust binary with Cargo, Stage 3: Debian-slim runtime).
- [ ] **Step 4: Run test to verify it passes**
  Run: `cargo test -p quickbox-app --test e2e_integration_test`
  Expected: PASS.
- [ ] **Step 5: Commit**
  ```bash
  git add crates/quickbox-app/ Dockerfile docker-compose.yml README.md
  git commit -m "feat(app): finalize monolith integration, static asset hosting, and Dockerfile"
  ```

