# Repo Skeleton + POST /internal/inbound Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Lay out the repo skeleton (`backend/`, `frontend/`, `deploy/`) and implement an internal Go HTTP endpoint, `POST /internal/inbound`, that persists an inbound email into SQLite, with tests covering the happy path and the failure modes a reasonable operator would expect it to handle.

**Architecture:** A single Go module under `backend/` with a domain-oriented package layout: `internal/store` owns the SQLite schema and persistence (inbox auto-provisioning, message insert), `internal/inbound` owns the HTTP handler that validates and decodes a JSON payload and calls into `store`. `cmd/server` wires them into a runnable binary. `frontend/` and `deploy/` get placeholder READMEs only — no code yet, per PRD roadmap (this plan covers roadmap steps 1–2, not the SMTP receiver, the web UI, or Docker packaging).

**Tech Stack:** Go (stdlib `net/http`, `database/sql`), `modernc.org/sqlite` (pure-Go SQLite driver — no CGO, keeps the "single binary" portability requirement from PRD §8/§9), Go's built-in `testing` + `net/http/httptest`.

**Spec:** `/Users/apple/Desktop/Quickbox/PRD.md` (sections 6 FR-01/FR-08, 7, 8, 9, 17) and `/Users/apple/Desktop/Quickbox/CLAUDE.md`. Executors should read both before starting.

## Global Constraints

- Backend language is Go; database is SQLite for this stage (PRD §9).
- The `Message` model must carry a `direction` column (`in`/`out`) and a thread reference from day one, even if threading logic isn't built yet (PRD §9 contract; CLAUDE.md "jangan hilangkan kolom ini meski belum dipakai").
- Default per-message size limit is 10 MB (PRD §8, NFR "Ukuran") and must be enforced at the ingestion endpoint, not left to the caller.
- `/internal/inbound` is an internal ingestion path, not part of the public `/api/...` surface listed in PRD §9 — it must not be mounted behind any future public router/CORS config.
- Inboxes are created automatically on first message, not pre-registered (PRD §7).
- Every behavior that touches persistence must have a test (CLAUDE.md "Tes wajib"); `gofmt`/`go vet` must be clean before each commit (CLAUDE.md "Konvensi kode").
- No attachment handling in this plan (out of scope — PRD FR-10 is P1, separate work).

## Review Focus

- Oversized request body (>10 MB default, PRD §8) must be rejected with an error response, not accepted in full or allowed to exhaust memory. → Task 3, `TestHandler_RejectsOversizedBody`.
- Missing or empty `inbox` field must be rejected (400), not silently create an inbox with an empty name. → Task 3, `TestHandler_RejectsMissingInbox`.
- Malformed JSON body must return 400, not panic or 500. → Task 3, `TestHandler_RejectsMalformedJSON`.
- Non-POST methods on `/internal/inbound` must return 405, not process the body. → Task 3, `TestHandler_RejectsNonPOST`.
- The first inbound message for a brand-new inbox name must auto-create that inbox row (PRD §7), and a second message to the same name must reuse it rather than erroring or creating a duplicate. → Task 2, `TestSaveInboundMessage_ReusesExistingInbox`.

---

### Task 1: Repo skeleton (`backend/`, `frontend/`, `deploy/`)

**Files:**
- Create: `backend/README.md`
- Create: `frontend/README.md`
- Create: `deploy/README.md`

**Interfaces:**
- Produces: the three top-level directories that Task 2 (`backend/...`) builds inside of. No code interfaces.

- [ ] **Step 1: Create `backend/README.md`**

One short paragraph: this directory holds the Go backend (SMTP receiver, HTTP API, realtime, SQLite persistence) — one process/binary, per PRD §9. Note that at this stage only the `/internal/inbound` ingestion endpoint exists; the SMTP server and public `/api/...` surface are future work.

- [ ] **Step 2: Create `frontend/README.md`**

One short paragraph: this directory will hold the React web inbox (PRD §9, §12). Empty placeholder for now — no code in this plan.

- [ ] **Step 3: Create `deploy/README.md`**

One short paragraph: this directory will hold Docker/docker-compose files for installation (PRD §9, FR-13). Empty placeholder for now — no code in this plan.

- [ ] **Step 4: Verify**

Run: `find backend frontend deploy -maxdepth 1 -type f`
Expected: lists exactly `backend/README.md`, `frontend/README.md`, `deploy/README.md`.

- [ ] **Step 5: Commit**

```bash
git add backend/README.md frontend/README.md deploy/README.md
git commit -m "chore: add repo skeleton (backend, frontend, deploy)"
```

---

### Task 2: SQLite store — schema + inbound message persistence

**Files:**
- Create: `backend/go.mod`
- Create: `backend/internal/store/sqlite.go`
- Create: `backend/internal/store/sqlite_test.go`

**Interfaces:**
- Consumes: nothing from earlier tasks (first code task).
- Produces (for Task 3 to consume):
  - `package store` (import path `backend/internal/store`)
  - `type InboundMessage struct { Inbox, From, To, Subject, Text, HTML, Headers string; ReceivedAt time.Time }`
  - `type Store struct { ... }` (opaque; fields not exported)
  - `func Open(path string) (*Store, error)`
  - `func (s *Store) Migrate(ctx context.Context) error`
  - `func (s *Store) Close() error`
  - `func (s *Store) SaveInboundMessage(ctx context.Context, msg InboundMessage) (messageID int64, err error)` — auto-creates the inbox row if `msg.Inbox` doesn't exist yet, reuses it otherwise; inserts the message with `direction='in'`; if `msg.ReceivedAt.IsZero()`, stores `time.Now().UTC()` instead.

- [ ] **Step 1: Initialize the Go module**

```bash
cd backend && go mod init backend && go get modernc.org/sqlite
```

- [ ] **Step 2: Write the failing tests in `backend/internal/store/sqlite_test.go`**

```go
package store

import (
	"context"
	"path/filepath"
	"testing"
	"time"
)

func openTestStore(t *testing.T) *Store {
	t.Helper()
	path := filepath.Join(t.TempDir(), "test.db")
	s, err := Open(path)
	if err != nil {
		t.Fatalf("Open: %v", err)
	}
	t.Cleanup(func() { s.Close() })
	if err := s.Migrate(context.Background()); err != nil {
		t.Fatalf("Migrate: %v", err)
	}
	return s
}

func TestMigrate_IsIdempotent(t *testing.T) {
	s := openTestStore(t)
	if err := s.Migrate(context.Background()); err != nil {
		t.Fatalf("second Migrate: %v", err)
	}
}

func TestSaveInboundMessage_CreatesInboxAndMessage(t *testing.T) {
	s := openTestStore(t)
	id, err := s.SaveInboundMessage(context.Background(), InboundMessage{
		Inbox:   "budi-test",
		From:    "noreply@app.com",
		To:      "budi-test@domain.com",
		Subject: "Verify your account",
		Text:    "Your code is 123456",
		HTML:    "<p>Your code is 123456</p>",
		Headers: "From: noreply@app.com\r\n",
	})
	if err != nil {
		t.Fatalf("SaveInboundMessage: %v", err)
	}
	if id <= 0 {
		t.Fatalf("expected positive message id, got %d", id)
	}

	var inboxCount int
	if err := s.db.QueryRow(`SELECT COUNT(*) FROM inboxes WHERE name = ?`, "budi-test").Scan(&inboxCount); err != nil {
		t.Fatalf("query inboxes: %v", err)
	}
	if inboxCount != 1 {
		t.Fatalf("expected 1 inbox row, got %d", inboxCount)
	}

	var direction, subject string
	if err := s.db.QueryRow(`SELECT direction, subject FROM messages WHERE id = ?`, id).Scan(&direction, &subject); err != nil {
		t.Fatalf("query messages: %v", err)
	}
	if direction != "in" {
		t.Fatalf("expected direction 'in', got %q", direction)
	}
	if subject != "Verify your account" {
		t.Fatalf("expected subject to be persisted, got %q", subject)
	}
}

func TestSaveInboundMessage_ReusesExistingInbox(t *testing.T) {
	s := openTestStore(t)
	ctx := context.Background()
	if _, err := s.SaveInboundMessage(ctx, InboundMessage{Inbox: "repeat", From: "a@a.com", To: "repeat@domain.com"}); err != nil {
		t.Fatalf("first SaveInboundMessage: %v", err)
	}
	if _, err := s.SaveInboundMessage(ctx, InboundMessage{Inbox: "repeat", From: "b@b.com", To: "repeat@domain.com"}); err != nil {
		t.Fatalf("second SaveInboundMessage: %v", err)
	}

	var inboxCount, messageCount int
	if err := s.db.QueryRow(`SELECT COUNT(*) FROM inboxes WHERE name = ?`, "repeat").Scan(&inboxCount); err != nil {
		t.Fatalf("query inboxes: %v", err)
	}
	if inboxCount != 1 {
		t.Fatalf("expected inbox to be reused (1 row), got %d", inboxCount)
	}
	if err := s.db.QueryRow(`SELECT COUNT(*) FROM messages WHERE from_addr IN ('a@a.com','b@b.com')`).Scan(&messageCount); err != nil {
		t.Fatalf("query messages: %v", err)
	}
	if messageCount != 2 {
		t.Fatalf("expected 2 message rows, got %d", messageCount)
	}
}

func TestSaveInboundMessage_DefaultsReceivedAtWhenZero(t *testing.T) {
	s := openTestStore(t)
	before := time.Now().UTC().Add(-time.Second)
	id, err := s.SaveInboundMessage(context.Background(), InboundMessage{Inbox: "ts-test", From: "a@a.com", To: "ts-test@domain.com"})
	if err != nil {
		t.Fatalf("SaveInboundMessage: %v", err)
	}
	var receivedAt time.Time
	if err := s.db.QueryRow(`SELECT received_at FROM messages WHERE id = ?`, id).Scan(&receivedAt); err != nil {
		t.Fatalf("query received_at: %v", err)
	}
	if receivedAt.Before(before) {
		t.Fatalf("expected received_at to default to ~now, got %v (before %v)", receivedAt, before)
	}
}
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `cd backend && go test ./internal/store/... -v`
Expected: FAIL — `Open`, `Store`, `InboundMessage`, etc. are undefined.

- [ ] **Step 4: Implement `backend/internal/store/sqlite.go`**

Implement the exact signatures listed in **Interfaces** above. Notes that fix the one choice the signatures leave open:
- Schema (apply via `Migrate` using `CREATE TABLE IF NOT EXISTS`):
  ```sql
  CREATE TABLE IF NOT EXISTS inboxes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    lock_hash TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
  );
  CREATE TABLE IF NOT EXISTS messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    inbox_id INTEGER NOT NULL REFERENCES inboxes(id),
    thread_id INTEGER,
    direction TEXT NOT NULL CHECK (direction IN ('in','out')),
    from_addr TEXT NOT NULL,
    to_addr TEXT NOT NULL,
    subject TEXT NOT NULL DEFAULT '',
    text_body TEXT NOT NULL DEFAULT '',
    html_body TEXT NOT NULL DEFAULT '',
    headers TEXT NOT NULL DEFAULT '',
    received_at DATETIME NOT NULL,
    read INTEGER NOT NULL DEFAULT 0,
    favorite INTEGER NOT NULL DEFAULT 0,
    deleted INTEGER NOT NULL DEFAULT 0
  );
  ```
  (`thread_id` stays nullable — threading logic is future work; the column exists per the PRD contract.)
- Inbox auto-provisioning: `INSERT INTO inboxes(name) VALUES(?) ON CONFLICT(name) DO NOTHING`, then `SELECT id FROM inboxes WHERE name = ?`. This is race-safe under SQLite's upsert without needing an explicit transaction.
- Open with driver `modernc.org/sqlite` (registers as `"sqlite"`), DSN is just the file path.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cd backend && go test ./internal/store/... -v`
Expected: PASS (all 4 tests).

- [ ] **Step 6: `gofmt` and `go vet`**

Run: `cd backend && gofmt -l . && go vet ./...`
Expected: no output from `gofmt -l`, no errors from `go vet`.

- [ ] **Step 7: Commit**

```bash
git add backend/go.mod backend/go.sum backend/internal/store
git commit -m "feat: add SQLite store with inbox auto-provisioning"
```

---

### Task 3: `POST /internal/inbound` HTTP handler + server entrypoint

**Files:**
- Create: `backend/internal/inbound/handler.go`
- Create: `backend/internal/inbound/handler_test.go`
- Create: `backend/cmd/server/main.go`
- Modify: `backend/internal/store/sqlite.go` (add a one-line `DB()` accessor, Step 3 below)

**Interfaces:**
- Consumes (from Task 2): `store.Open`, `(*store.Store).Migrate`, `(*store.Store).SaveInboundMessage`, `store.InboundMessage`.
- Produces:
  - `package inbound` (import path `backend/internal/inbound`)
  - `const DefaultMaxBodyBytes int64 = 10 << 20` (10 MB, PRD §8)
  - `type Handler struct { Store *store.Store; MaxBodyBytes int64 }` implementing `http.Handler` via `ServeHTTP(w http.ResponseWriter, r *http.Request)`
  - JSON request shape: `{"inbox","from","to","subject","text","html","headers"}` (all strings)
  - JSON success response (201): `{"id": <int64>, "inbox": "<string>"}`

- [ ] **Step 1: Write the failing tests in `backend/internal/inbound/handler_test.go`**

```go
package inbound

import (
	"bytes"
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"path/filepath"
	"strings"
	"testing"

	"backend/internal/store"
)

func newTestHandler(t *testing.T) (*Handler, *store.Store) {
	t.Helper()
	s, err := store.Open(filepath.Join(t.TempDir(), "test.db"))
	if err != nil {
		t.Fatalf("store.Open: %v", err)
	}
	t.Cleanup(func() { s.Close() })
	if err := s.Migrate(context.Background()); err != nil {
		t.Fatalf("Migrate: %v", err)
	}
	return &Handler{Store: s, MaxBodyBytes: DefaultMaxBodyBytes}, s
}

func TestHandler_ValidInbound_Returns201AndPersists(t *testing.T) {
	h, _ := newTestHandler(t)
	body := `{"inbox":"budi-test","from":"noreply@app.com","to":"budi-test@domain.com","subject":"Hi","text":"body"}`
	req := httptest.NewRequest(http.MethodPost, "/internal/inbound", strings.NewReader(body))
	rec := httptest.NewRecorder()

	h.ServeHTTP(rec, req)

	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}
	var resp struct {
		ID    int64  `json:"id"`
		Inbox string `json:"inbox"`
	}
	if err := json.Unmarshal(rec.Body.Bytes(), &resp); err != nil {
		t.Fatalf("decode response: %v", err)
	}
	if resp.ID <= 0 || resp.Inbox != "budi-test" {
		t.Fatalf("unexpected response: %+v", resp)
	}
}

func TestHandler_RejectsNonPOST(t *testing.T) {
	h, _ := newTestHandler(t)
	req := httptest.NewRequest(http.MethodGet, "/internal/inbound", nil)
	rec := httptest.NewRecorder()

	h.ServeHTTP(rec, req)

	if rec.Code != http.StatusMethodNotAllowed {
		t.Fatalf("expected 405, got %d", rec.Code)
	}
}

func TestHandler_RejectsMalformedJSON(t *testing.T) {
	h, _ := newTestHandler(t)
	req := httptest.NewRequest(http.MethodPost, "/internal/inbound", strings.NewReader("{not json"))
	rec := httptest.NewRecorder()

	h.ServeHTTP(rec, req)

	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400, got %d", rec.Code)
	}
}

func TestHandler_RejectsMissingInbox(t *testing.T) {
	h, s := newTestHandler(t)
	body := `{"inbox":"","from":"a@a.com","to":"x@domain.com"}`
	req := httptest.NewRequest(http.MethodPost, "/internal/inbound", strings.NewReader(body))
	rec := httptest.NewRecorder()

	h.ServeHTTP(rec, req)

	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400, got %d", rec.Code)
	}
	var count int
	if err := s.DB().QueryRow(`SELECT COUNT(*) FROM messages`).Scan(&count); err != nil {
		t.Fatalf("query messages: %v", err)
	}
	if count != 0 {
		t.Fatalf("expected no message persisted, got %d", count)
	}
}

func TestHandler_RejectsOversizedBody(t *testing.T) {
	h, s := newTestHandler(t)
	h.MaxBodyBytes = 10 // force a tiny limit for this test
	big := bytes.Repeat([]byte("a"), 1024)
	req := httptest.NewRequest(http.MethodPost, "/internal/inbound", bytes.NewReader(big))
	rec := httptest.NewRecorder()

	h.ServeHTTP(rec, req)

	if rec.Code != http.StatusRequestEntityTooLarge {
		t.Fatalf("expected 413, got %d: %s", rec.Code, rec.Body.String())
	}
	var count int
	if err := s.DB().QueryRow(`SELECT COUNT(*) FROM messages`).Scan(&count); err != nil {
		t.Fatalf("query messages: %v", err)
	}
	if count != 0 {
		t.Fatalf("expected no message persisted, got %d", count)
	}
}
```

Note: this test calls `s.DB()` — add a small accessor to Task 2's `Store` if it isn't already exported. Since Task 2 kept `db` unexported, add `func (s *Store) DB() *sql.DB { return s.db }` as part of this task's Step 4 (implementation), in `backend/internal/store/sqlite.go` — it's a one-line addition needed for test assertions, not a scope change to Task 2's behavior.

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd backend && go test ./internal/inbound/... -v`
Expected: FAIL — `Handler`, `DefaultMaxBodyBytes`, `(*store.Store).DB` undefined.

- [ ] **Step 3: Add `func (s *Store) DB() *sql.DB` to `backend/internal/store/sqlite.go`**

One line: `return s.db`.

- [ ] **Step 4: Implement `backend/internal/inbound/handler.go`**

Implement `ServeHTTP` per the **Interfaces** block and the JSON shapes above:
1. If `r.Method != http.MethodPost`: `http.Error(w, "method not allowed", http.StatusMethodNotAllowed)`, return.
2. Wrap body: `r.Body = http.MaxBytesReader(w, r.Body, h.MaxBodyBytes)`.
3. Decode JSON into the request struct; on error, use `errors.As` to check for `*http.MaxBytesError` → respond 413; any other decode error → respond 400.
4. `strings.TrimSpace` the `inbox` field; if empty, respond 400.
5. Call `h.Store.SaveInboundMessage(r.Context(), store.InboundMessage{...})` mapping all fields 1:1 (leave `ReceivedAt` zero so the store defaults it); on error, respond 500.
6. On success, `w.WriteHeader(http.StatusCreated)` and encode `{"id": id, "inbox": inbox}` as JSON.

- [ ] **Step 5: Implement `backend/cmd/server/main.go`**

Wire a runnable binary: read DB path from env `QUICKBOX_DB_PATH` (default `"./data/quickbox.db"`) and listen address from env `PORT` (default `":8080"`); `store.Open` + `Migrate`; register `&inbound.Handler{Store: s, MaxBodyBytes: inbound.DefaultMaxBodyBytes}` at path `/internal/inbound` on a `http.ServeMux`; `http.ListenAndServe`. Log fatal on setup errors.

- [ ] **Step 6: Run tests to verify they pass**

Run: `cd backend && go test ./... -v`
Expected: PASS (all tests in `internal/store` and `internal/inbound`).

- [ ] **Step 7: Build the binary**

Run: `cd backend && go build ./...`
Expected: builds with no errors.

- [ ] **Step 8: `gofmt` and `go vet`**

Run: `cd backend && gofmt -l . && go vet ./...`
Expected: no output from `gofmt -l`, no errors from `go vet`.

- [ ] **Step 9: Commit**

```bash
git add backend/internal/inbound backend/cmd/server backend/internal/store/sqlite.go
git commit -m "feat: add POST /internal/inbound handler and server entrypoint"
```
