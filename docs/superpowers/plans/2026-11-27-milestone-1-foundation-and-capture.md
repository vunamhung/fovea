# Fovea Milestone 1 — Foundation, Core Engine & Native macOS Capture Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver a compiling, tested, runnable pure-Rust macOS GPUI Studio application (`vunamhung/fovea`) with a complete `.fovea` core domain/command/physics engine and native macOS screen/audio/telemetry capture.

**Architecture:** 6-crate Cargo workspace (`fovea-core`, `fovea-capture`, `fovea-render`, `fovea-ai`, `fovea-mcp`, `fovea-app`) targeting macOS 14+ first behind OS-agnostic backend traits. GPUI runs via `gpui-kit 0.11` (`runtime_shaders`) so no full Xcode installation is required. Milestones 2 (Metal compositor, interactive stage, full timeline, VideoToolbox MP4 + GIF) and 3 (AI Chat, live `fovea mcp`, Vbee TTS, on-demand Whisper CoreML) follow in dedicated plans.

**Tech Stack:** Rust 1.100+, `gpui-kit 0.11`, `screencapturekit 12` (`macos_14_4`), `objc2-* 0.3.2`, `serde`/`serde_json`, `thiserror`, `uuid`, `clap`.

**Spec:** `docs/superpowers/specs/2026-11-27-fovea-design.md`

## Global Constraints

- Pure Rust; zero Electron/WebView and zero FFmpeg subprocesses in capture/export paths.
- macOS 14.6.1+ (`arm64`) with Command Line Tools only (`gpui-kit` / `runtime_shaders`, no `xcrun metal`).
- Every source file ≤ 250 lines, DRY, SOLID, English-only UI strings.
- Single-take multi-clip timeline (`ProjectDoc` v1), atomic Undo/Redo transactions, MIT license.

## Review Focus

1. Truncated/malformed lines in `telemetry.jsonl` or unknown `schema_version` in `project.json` → recover valid lines / return typed error without panicking (Task 2).
2. Zero-length, negative, or out-of-bounds timeline split/trim/freeze edits and empty AI transactions → cleanly rejected or no-op with lossless Undo/Redo (Task 3).
3. Double `stop()` or stopping capture before the first frame arrives → writer finalizes safely without deadlock or corrupt 0-byte `.mov` (Task 4).
4. Missing macOS TCC permissions (Screen Recording, Accessibility, Input Monitoring, Mic) → structured `PermissionStatus` returned to UI with System Settings deep-link (Task 4).
5. macOS Secure Input active (password fields) during Keystroke HUD capture → keystrokes strictly dropped and never persisted to `telemetry.jsonl` (Task 5).

---

### Task 1: Cargo Workspace & 6-Crate Skeleton

**Files:**
- Create: `Cargo.toml`, `crates/fovea-{core,capture,render,ai,mcp,app}/Cargo.toml`, `src/lib.rs` (5 lib crates), `crates/fovea-app/src/main.rs`

**Interfaces:**
- Produces: Compiling workspace with shared workspace deps (`gpui-kit 0.11`, `serde`, `serde_json`, `thiserror`, `uuid`, `tracing`, `screencapturekit 12`, `objc2-* 0.3.2`).

- [ ] **Step 1:** Create root `Cargo.toml` with resolver 2, edition 2024, `[profile.release]` (`lto = "thin"`, `strip = "symbols"`), and 6 member crates.
- [ ] **Step 2:** Create minimal `Cargo.toml` + `src/lib.rs` / `src/main.rs` for all 6 crates (pinning `gpui-kit = { version = "0.11.0", default-features = false, features = ["component", "assets"] }`).
- [ ] **Step 3:** Run `cargo check --workspace` — Expected: exit 0 with no `xcrun metal` error.
- [ ] **Step 4:** Commit `chore: scaffold 6-crate Fovea Rust workspace`.

### Task 2: `fovea-core` — `.fovea` Project Schema, Bundle & Telemetry I/O

**Files:**
- Create: `crates/fovea-core/src/{schema,telemetry,bundle,error,lib}.rs`
- Test: `crates/fovea-core/tests/bundle_io.rs`

**Interfaces:**
- Produces: `ProjectDoc` (`SCHEMA_VERSION = 1`), `ClipSegment`, `SpeedSegment`, `FreezeSegment`, `ZoomRegion`, `AnnotationItem` (Text, Shape, Image, PrivacyBlur), `CaptionWord`, `VoiceClip` (`auto_duck: bool`, `video_sync: VoiceSyncMode`), `FrameStyle`, `CursorStyle`, `WebcamConfig`, `TelemetryEvent`, `AxClickContext`, `FoveaBundle::{create, open, save_project, append_telemetry, read_telemetry_lenient}`.

- [ ] **Step 1:** Write failing tests in `crates/fovea-core/tests/bundle_io.rs` covering `FoveaBundle::create`/`open` round-trip, unsupported `schema_version: 99` returning `CoreError::UnsupportedSchemaVersion(99)`, and `read_telemetry_lenient` skipping a corrupted trailing JSONL line while preserving prior events.
- [ ] **Step 2:** Run `cargo test -p fovea-core --test bundle_io` — Expected: FAIL (types undefined).
- [ ] **Step 3:** Implement `error.rs`, `schema.rs`, `telemetry.rs`, `bundle.rs`, and exports in `lib.rs` (each file ≤ 250 lines).
- [ ] **Step 4:** Run `cargo test -p fovea-core --test bundle_io` — Expected: PASS.
- [ ] **Step 5:** Commit `feat(core): add .fovea project schema, bundle I/O, and telemetry log`.

### Task 3: `fovea-core` — Timeline Math, Spring Physics, `StageTransform` & Command Engine

**Files:**
- Create: `crates/fovea-core/src/{timeline,physics,stage,command,engine}.rs`
- Modify: `crates/fovea-core/src/lib.rs`
- Test: `crates/fovea-core/tests/engine_and_math.rs`

**Interfaces:**
- Consumes: `ProjectDoc`, `ClipSegment`, `ZoomRegion`, `TelemetryEvent` from Task 2.
- Produces: `TimelineMap::{output_duration_ms, map_output_to_source}`, `SpringState::step`, `evaluate_cursor_pose`, `StageTransform::compute` + `stage_to_source_point`, `EditCommand` enum, `CommandEngine::{new, project, apply, begin_transaction, commit_transaction, undo, redo}`.

- [ ] **Step 1:** Write failing tests in `crates/fovea-core/tests/engine_and_math.rs`: split/trim/reorder + freeze-frame `map_output_to_source`, invalid zero-length trim rejected (`CoreError::InvalidRange`), spring camera converges smoothly to target zoom, `StageTransform` round-trips stage↔source normalized points, atomic multi-command AI transaction undoes/redoes in 1 step, empty transaction leaves undo stack unchanged.
- [ ] **Step 2:** Run `cargo test -p fovea-core --test engine_and_math` — Expected: FAIL.
- [ ] **Step 3:** Implement `timeline.rs`, `physics.rs`, `stage.rs`, `command.rs`, `engine.rs` (each ≤ 250 lines).
- [ ] **Step 4:** Run `cargo test -p fovea-core` — Expected: PASS.
- [ ] **Step 5:** Commit `feat(core): add timeline mapper, spring physics, stage math, and command engine`.

### Task 4: `fovea-capture` — `CaptureBackend`, macOS Permissions, ScreenCaptureKit & `AVAssetWriter`

**Files:**
- Create: `crates/fovea-capture/src/{backend,permissions,error,lib}.rs`, `crates/fovea-capture/src/macos/{mod,sources,writer,recorder}.rs`
- Test: `crates/fovea-capture/tests/capture_lifecycle.rs`

**Interfaces:**
- Consumes: `FoveaBundle` from Task 2.
- Produces: `CaptureSource` (`Display`, `Window`, `Region`), `CaptureConfig`, `PermissionKind` (`ScreenRecording`, `Accessibility`, `InputMonitoring`, `Microphone`, `Camera`), `PermissionStatus`, `CaptureBackend` trait, `MacOsCaptureBackend`.

- [ ] **Step 1:** Write failing tests in `crates/fovea-capture/tests/capture_lifecycle.rs` testing `CaptureConfig::validate` (rejecting zero/odd dimensions), idempotent double-`stop()`, early-stop with zero frames producing `CaptureStats { video_frames: 0 }` without panic/0-byte corruption, and `PermissionReport::all_required_granted()`.
- [ ] **Step 2:** Run `cargo test -p fovea-capture --test capture_lifecycle` — Expected: FAIL.
- [ ] **Step 3:** Implement `error.rs`, `permissions.rs` (macOS CGPreflightScreenCaptureAccess / AXIsProcessTrusted / System Settings URLs), `backend.rs`, and `macos/{sources,writer,recorder}.rs` using `screencapturekit 12` (`macos_14_4`) + `AVAssetWriter` via `objc2-av-foundation`.
- [ ] **Step 4:** Run `cargo test -p fovea-capture` — Expected: PASS.
- [ ] **Step 5:** Commit `feat(capture): add macOS ScreenCaptureKit recorder, writer, and permissions`.

### Task 5: `fovea-capture` — `CGEventTap` Telemetry, `AXUIElement` Context & Secure-Input Keystrokes

**Files:**
- Create: `crates/fovea-capture/src/{keystroke_filter,telemetry_tap}.rs`, `crates/fovea-capture/src/macos/{event_tap,ax_context}.rs`
- Modify: `crates/fovea-capture/src/{lib,macos/mod}.rs`
- Test: `crates/fovea-capture/tests/keystroke_and_telemetry.rs`

**Interfaces:**
- Consumes: `TelemetryEvent`, `AxClickContext`, `FoveaBundle::append_telemetry` from Task 2.
- Produces: `RawKeyEvent`, `filter_keystroke(event: &RawKeyEvent, secure_input_active: bool) -> Option<String>`, `TelemetryTapHandle` (`start`, `stop`).

- [ ] **Step 1:** Write failing tests in `crates/fovea-capture/tests/keystroke_and_telemetry.rs`: `filter_keystroke` returns `None` for ALL keys when `secure_input_active == true`, formats `Cmd+Shift+4` → `"⌘⇧4"` and `Ctrl+Opt+Z` → `"⌃⌥Z"`, drops plain typing when `shortcuts_only == true`, and flushes buffered telemetry on stop.
- [ ] **Step 2:** Run `cargo test -p fovea-capture --test keystroke_and_telemetry` — Expected: FAIL.
- [ ] **Step 3:** Implement `keystroke_filter.rs`, `telemetry_tap.rs`, and macOS `CGEventTap` + `IsSecureEventInputEnabled()` + `AXUIElementCopyElementAtPosition` in `macos/{event_tap,ax_context}.rs`.
- [ ] **Step 4:** Run `cargo test -p fovea-capture` — Expected: PASS.
- [ ] **Step 5:** Commit `feat(capture): add CGEventTap telemetry, AX click context, and safe keystrokes`.

### Task 6: `fovea-app` — GPUI Studio Shell (`gpui-kit`), Recording HUD, CLI & Self-Signing Script

**Files:**
- Create: `crates/fovea-app/src/{main,cli,state}.rs`, `crates/fovea-app/src/ui/{mod,studio,topbar,stage,inspector,timeline,record_modal,permissions}.rs`, `scripts/sign-macos-app.sh`
- Test: `crates/fovea-app/tests/cli_and_state.rs`

**Interfaces:**
- Consumes: `CommandEngine`, `FoveaBundle` (Tasks 2–3); `MacOsCaptureBackend`, `PermissionReport`, `TelemetryTapHandle` (Tasks 4–5).
- Produces: Runnable `fovea` desktop Studio binary (`fovea` GUI + `fovea mcp` stub), macOS self-signing helper script `scripts/sign-macos-app.sh`.

- [ ] **Step 1:** Write failing tests in `crates/fovea-app/tests/cli_and_state.rs`: CLI parses `fovea`, `fovea open <path>`, and `fovea mcp`; `StudioState` transitions `Idle -> Recording -> Editing` and dispatches Undo/Redo + right-panel Inspector/AI-Chat tab switching.
- [ ] **Step 2:** Run `cargo test -p fovea-app --test cli_and_state` — Expected: FAIL.
- [ ] **Step 3:** Implement `cli.rs`, `state.rs`, modular English-only GPUI views under `src/ui/` (each ≤ 250 lines) using `gpui-kit` dark studio layout matching the spec, `main.rs`, and `scripts/sign-macos-app.sh`.
- [ ] **Step 4:** Run `cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings && cargo build --release -p fovea-app` and check release binary size — Expected: all tests PASS, zero warnings, binary built cleanly without Xcode.
- [ ] **Step 5:** Commit `feat(app): build GPUI Studio shell, recording controls, and macOS signing script` and push to `origin main`.
