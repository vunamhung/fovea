# Fovea — Product & Architecture Design

## 1. Intent & Why

**Fovea** (`github.com/vunamhung/fovea`, MIT, local `/Users/hungvn6.sc/github/fovea`) is a pure-Rust, GPU-native screen recorder and demo video editor built with **GPUI** (`gpui.rs`).

It eliminates every structural weakness of Electron recorders like Recordly — bloated bundles, high idle RAM/CPU, WebGL-to-Canvas2D CPU readbacks + IPC rawvideo pipes, dual-engine Preview/Export mismatches, and 5-language native helper sprawl — while adding **AI-native editing**: an in-app conversational Chat UI, live **Claude / Antigravity (AGY) MCP control**, **Vbee neural voiceovers**, and offline **Vietnamese + English auto-captions**.

## 2. Goals, Non-Goals & Anti-Goals

### Goals
- **Ultra-lightweight footprint:** ~15–25 MB release binary (no bundled FFmpeg, no bundled AI models), minimal idle CPU/RAM, instant startup.
- **Single GPU Compositor (Preview == Export):** One Metal shader pipeline renders both the live GPUI viewport (`CVPixelBuffer` → `gpui::surface` zero-copy) and VideoToolbox/`AVAssetWriter` MP4 output — pixel-identical preview and export with faster-than-realtime hardware encoding.
- **Complete Recordly parity + pro upgrades:**
  - **Capture:** Full display, single window, custom region; system audio, mic, webcam; global hotkeys; OS cursor hiding; cursor/click telemetry + lightweight active app/AXUIElement click context; **Keystroke HUD** events (filtering macOS Secure Input password fields).
  - **Styling & Motion:** Solid/gradient/image/video backgrounds, wallpaper blur, padding, squircle corners, soft shadow, aspect presets, crop, spring auto-zoom & manual zoom regions, cursor smoothing/click bounce/motion blur/sway/idle hide, zoom-reactive webcam bubble, **Privacy Blur** redaction regions.
  - **Non-Destructive Timeline (Single-Take Multi-Clip):** Split, trim, delete gaps, reorder segments, speed regions, **freeze-frame/retiming segments** (for voice sync), zoom track, text/shape/image/privacy-blur annotations, Keystroke HUD track, word-level animated captions, extra audio/voiceover tracks with **auto-ducking**, `.fovea` project bundle, MP4 (H.264/HEVC) & pure-Rust GIF export.
  - **Direct Stage Manipulation:** Coordinate-synced interactive GPUI gizmo overlay on top of the Live GPU Stage for clicking zoom focal points and dragging/resizing webcam bubble, crop box, privacy blur, and annotations.
- **AI-Native Editing & Voice:**
  - **In-App AI Chat Panel:** Streaming markdown UI powered by local **`claude` / `agy` CLIs** (inheriting user subscriptions) or direct HTTP API keys (Anthropic/OpenAI/Gemini).
  - **Live MCP Server (`fovea mcp`):** External Claude Code / Claude Desktop / AGY control open projects live (including on-demand visual frame snapshots for silent demos).
  - **Vbee TTS (`https://vbee.vn/`)** primary voiceover engine (live voice catalog + async synthesis) with zero-MB macOS native voice offline fallback, auto-ducking, and optional video freeze-frame/retime sync.
  - **On-Demand Local Whisper (CoreML):** Downloaded only on first use; 100% offline Vietnamese + English word-level timestamps.

### Non-Goals (v1 / macOS-first)
- Shipping Windows/Linux capture & hardware encode backends in Milestones 1–3 (traits stay OS-agnostic so Windows WGC/MediaFoundation & Linux PipeWire/VA-API plug in cleanly in Phase 4).
- Multi-source NLE mixing arbitrary external video files with disparate resolutions/FPS into one timeline (v1 is single-take segmentation: split/trim/reorder segments of one recording).
- Cloud video hosting, accounts, billing, or tethered mobile capture.

### Anti-Goals
- Any Electron/WebView runtime or external FFmpeg subprocess in the core capture/export path.
- Any GPU→CPU Canvas readback or IPC raw-frame pipe on macOS preview or MP4 export.
- Separate preview vs. export rendering engines that can visually diverge.
- Non-English UI text (all UI labels/chrome strictly English).
- Monolithic files (every source file ≤ 250 lines, modular, DRY, SOLID).

## 3. Constraints & Verified Host Environment

- **Host:** macOS 14.6.1 (Sonoma), Apple Silicon (`arm64`), `rustc 1.100.0`, `cargo 1.100.0`, `gh` logged in as `vunamhung`.
- **Zero-Xcode GPUI Setup:** Host has Command Line Tools only (no `xcrun metal`). Resolved by pinning `gpui = { package = "gpui-pre", version = "=0.3.14" }`, `gpui_platform = { package = "gpui-pre-platform", version = "=0.3.14", features = ["font-kit", "runtime_shaders"] }`, and `gpui-component = "0.11.0"`. Both GPUI and `fovea-render` compile Metal shaders at runtime via `MTLDevice::newLibraryWithSource`.
- **Verified Local AI CLIs:** Both `/Users/hungvn6.sc/.local/bin/claude` and `/Users/hungvn6.sc/.local/bin/agy` exist and support non-interactive streaming (`-p --input-format stream-json --output-format stream-json --resume <id>`) plus MCP wiring (`claude --mcp-config` + `--allowedTools`; `agy --settings` + `--sandbox` or `agy acp`).
- **macOS Permissions & Self-Signing:** Screen Recording, Accessibility (cursor/AXUIElement context), Input Monitoring (Keystroke HUD, respecting `IsSecureEventInputEnabled()`), Microphone, Camera. Ship a local self-signing script (proven in `vunamhung/rec`) so TCC permissions survive rebuilds.

## 4. Approaches Considered

| Approach | Summary | Tradeoff | Decision |
| --- | --- | --- | --- |
| **1. Apple Native Zero-Copy + Unified Metal Compositor + `gpui-component`** | `gpui-component` UI; ScreenCaptureKit + AVFoundation + CGEventTap; single Metal compositor outputting `CVPixelBuffer` directly to `gpui::surface` & VideoToolbox/`AVAssetWriter`, pure-Rust GIF encoder. | Requires careful `objc2-*` bindings, but delivers true zero-copy, ~15–25 MB binary, no FFmpeg, exact Preview==Export parity. | **Selected** |
| **2. Unified `wgpu` Compositor + FFmpeg Encode/Decode** | Same UI/AI/capture, delegating MP4/GIF decode/encode to `ffmpeg`. | Faster multi-OS codec parity, but adds GPU→CPU readback and ~60–80 MB FFmpeg bundle bloat. | Rejected |
| **3. Raw `gpui` (no `gpui-component`) + Apple Native** | Build all 60+ desktop UI primitives from raw `gpui::div()`. | Minimal dependencies, but burns weeks rebuilding inputs/docks/sliders/markdown chat. | Rejected |

## 5. System Architecture

### Cargo Workspace Layout
```text
fovea/
├── crates/
│   ├── fovea-core/     # .fovea bundle schema, single-take piecewise timeline + freeze-frame
│   │                   # math, shared StageTransform, spring zoom/cursor physics,
│   │                   # Unified Command Engine (Undo/Redo + AI batch transactions)
│   ├── fovea-capture/  # CaptureBackend trait + macOS ScreenCaptureKit (`screencapturekit`
│   │                   # 12 / `objc2-*`), AVFoundation (mic/webcam), CGEventTap (cursor,
│   │                   # clicks, AXUIElement click label, Secure-Input-safe keystrokes)
│   ├── fovea-render/   # Unified Metal Compositor (runtime shaders, multi-pass privacy blur,
│   │                   # word-level caption/annotation rasterizer), IOSurface CVPixelBuffer
│   │                   # output, VideoToolbox AVAssetWriter MP4, pure-Rust GIF, auto-duck mixer
│   ├── fovea-ai/       # Unified Chat engine (claude/agy CLI bridge + Anthropic/OpenAI/Gemini
│   │                   # APIs), Vbee TTS client + voice catalog + macOS fallback,
│   │                   # on-demand Whisper CoreML STT downloader & transcriber
│   ├── fovea-mcp/      # Live MCP server (`rmcp`) + Unix socket IPC (`live.sock`)
│   └── fovea-app/      # GPUI Studio UI (`gpui-component`), Stage + interactive gizmo layer,
│                       # Multi-track Timeline, Inspector, AI Chat panel, CLI (`fovea`, `fovea mcp`)
└── docs/superpowers/specs/
```

### Key Architectural Contracts
1. **Non-Destructive `.fovea` Bundle (`project.fovea/`):** Holds `project.json` (versioned single-take clip segments, speed/freeze-frame ranges, zoom regions, cursor/keystroke HUD settings, background/frame style, webcam config, privacy blur & text/shape/image annotations, word-level captions, voiceover tracks + auto-ducking config), `telemetry.jsonl` (cursor positions, clicks, active app/AXUIElement label, filtered keystrokes), and media (`screen.mov`, `webcam.mov`, `system.caf`, `mic.caf`, `voices/*.wav|mp3`).
2. **Unified Command Engine & Atomic AI Undo (`fovea-core`):** Every UI drag, AI Chat turn, and MCP call emits typed `EditCommand`s wrapped in transactions (`BeginTransaction`/`CommitTransaction`), giving one-click Undo/Redo across multi-step AI edits and instant live GPUI preview refresh.
3. **Single Metal Compositor + Shared `StageTransform` (`fovea-core` + `fovea-render`):**
   - `fovea-core` computes pure, deterministic `FrameState` and `StageTransform` at timestamp `t` (clip time mapping including speed/freeze-frames, spring camera zoom, smoothed cursor + bounce + velocity vector, webcam rect, active privacy blurs, keystroke pills, annotations, word-level highlighted captions).
   - `fovea-render` executes one runtime-compiled Metal pipeline into an `IOSurface`-backed `CVPixelBuffer` → zero-copy to `gpui::surface()` (preview) and `AVAssetWriter`/VideoToolbox (MP4 export), or quantized frames to pure-Rust GIF.
   - `fovea-app` uses the exact same `StageTransform` to render interactive GPUI gizmos (drag/resize crop, webcam, blur boxes, annotations, click-to-zoom) directly over `gpui::surface()`.
4. **AI Chat, Visual Context & `fovea mcp` (`fovea-ai` + `fovea-mcp`):**
   - Running GUI listens on `~/Library/Application Support/Fovea/live.sock`; `fovea mcp` proxies MCP tools live (or opens a `.fovea` bundle headlessly): `get_project_state`, `get_telemetry_summary`, `capture_frame_snapshot` (on-demand resized JPEG/PNG from `fovea-render` so AI sees silent demos), `apply_edit_commands`, `generate_vbee_voice`, `transcribe_captions`, `export_video`.
   - **Our call on CLI execution:** In-app Chat spawns `claude`/`agy` in headless `stream-json` mode connected exclusively to `fovea mcp` (disabling host shell/file tools via `--allowedTools` / `--sandbox`) so both CLI and HTTP API modes manipulate the project strictly through `EditCommand` transactions.
5. **Vbee TTS & Voice-Video Sync (`fovea-ai` + `fovea-core` + `fovea-render`):**
   - Fetches voice catalog via `GET https://vbee.vn/api/public/v1/voices` (`app-id` header) with bundled fallback list; synthesizes via `POST https://vbee.vn/api/v1/tts` (`Authorization: Bearer <token>`, `app_id`, `input_text`, `voice_code`, `speed_rate`, `audio_type`) + polling `GET /api/v1/tts/{request_id}` (configurable `callback_url` in Settings if required by the user's Vbee tier).
   - Inserts voice clip onto the Voiceover track with **auto-ducking** enabled by default (smoothly lowering mic/system audio underneath), plus per-clip **Sync Video** mode (auto freeze-frame or slow-down when narration outlasts the visual beat).
6. **On-Demand Local Whisper CoreML (`fovea-ai`):** First time user clicks *Auto-Captions*, Fovea downloads the chosen Whisper GGML/CoreML model (`base`/`small`/`large-v3-turbo`) into `~/Library/Application Support/Fovea/models/` with progress UI and runs local offline transcription producing word-level timestamps for animated captions.

### Studio UI Layout (GPUI)
```text
+---------------------------------------------------------------------------------+
| [● Record] [Demo.fovea]   [16:9 ▾] [Zoom: Auto]           [Undo/Redo] [Export]  |
+------------------------------------------------+--------------------------------+
|                                                | [Inspector] | [AI Chat]        |
|          LIVE GPU STAGE + GIZMOS               |--------------------------------|
|        (CVPixelBuffer zero-copy)               | Frame / Blur / Auto-Zoom       |
|   Styled Frame + Screen + Cursor + KeyHUD      | Cursor / Webcam / Vbee Voice   |
|       + Webcam Bubble + Privacy Blur           |   — or —                       |
|                                                | Chat stream [claude/agy/API]   |
+------------------------------------------------+--------------------------------+
| ▶ 00:12.4 / 01:05.0  [✂ Split][+Zoom][+Blur][+Text][+Voice][+CC]   [- Zoom +]   |
| Video      [====== Clip 1 ======][❄ Freeze][===== Clip 2 =====]                 |
| Zoom           [  2.0x  ]                      [  1.8x  ]                       |
| Overlays   [KeyHUD] [🔒 Blur] [Intro Title] [Word Captions...]                   |
| Audio/Vbee [~~~ Mic/Sys (ducked) ~~~]  [~~~ Vbee Voiceover ~~~]                 |
+---------------------------------------------------------------------------------+
```

## 6. Phased Delivery Milestones

- **Milestone 1 — Repo, Studio Shell & Native macOS Capture:** Create public `vunamhung/fovea`, full Cargo workspace, GPUI + `gpui-component` Studio UI (dark theme, launcher/HUD, source picker, TCC permission guide, self-signing script), native ScreenCaptureKit + AVFoundation + CGEventTap recording (video, mic, system audio, webcam, cursor/clicks/AXUIElement context, Secure-Input-filtered keystrokes) → saved `.fovea` bundle.
- **Milestone 2 — Unified Metal Compositor, Interactive Stage, Timeline & Hardware Export:** Single Metal compositor (backgrounds, squircle corners, shadows, multi-pass privacy blur, spring auto-zoom, cursor physics/motion blur, Keystroke HUD, webcam bubble, annotations, word-level captions), interactive GPUI gizmo overlay, full single-take multi-clip timeline (split/trim/reorder/speed/freeze-frame/auto-duck), zero-copy `gpui::surface` preview, VideoToolbox MP4 + pure-Rust GIF export.
- **Milestone 3 — AI Chat UI, Live `fovea mcp`, Vbee TTS & On-Demand Whisper CoreML:** In-app AI Chat (`claude`/`agy` CLI sandboxed to `fovea mcp` + direct API keys in Keychain), `fovea mcp` with visual frame snapshots, full Vbee TTS voice picker/generator + auto-duck & freeze-frame sync + macOS offline voice fallback, and on-demand local Whisper CoreML auto-captions.

## 7. Left to the Builder

- Exact Metal shader math for multi-pass Kawase/Gaussian blur (backgrounds & privacy redaction), squircle SDF masks, and directional zoom/cursor motion blur.
- Exact `gpui-component` theme palette, icons, keyboard shortcut map, and default panel proportions.
- Choice of pure-Rust GIF quantizer (`color_quant` vs `quantette`), text/SVG overlay rasterizer (`cosmic-text`/`tiny-skia` or CoreText→Metal texture), and audio resampler (`rubato` vs CoreAudio).
- Exact Unix socket JSON-RPC wire framing between `fovea-app` and `fovea-mcp`, and Vbee polling backoff / disk cache keys.
