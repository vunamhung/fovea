# Fovea

Pure-Rust, GPU-native screen recorder and AI-powered demo video studio built with [GPUI](https://gpui.rs).

## Why Fovea

- **Ultra-lightweight native binary:** Zero Electron, zero bundled FFmpeg, zero bundled AI models.
- **Single GPU Compositor:** One Metal shader pipeline drives both live zero-copy `CVPixelBuffer` preview and VideoToolbox hardware MP4 export.
- **AI-Native Editing & Voice:** Built-in AI Chat (`claude` / `agy` CLI + API keys), live MCP server (`fovea mcp`), Vbee neural voiceovers, and on-demand offline Whisper CoreML captions.

## Workspace Crates

- `crates/fovea-core`: `.fovea` project schema, single-take timeline math, spring zoom/cursor physics, shared `StageTransform`, and atomic Undo/Redo Command Engine.
- `crates/fovea-capture`: Cross-platform `CaptureBackend` trait + macOS ScreenCaptureKit, AVFoundation, CGEventTap telemetry, AXUIElement click context, and Secure-Input-safe keystroke capture.
- `crates/fovea-render`: Unified Metal GPU compositor, VideoToolbox `AVAssetWriter` MP4 exporter, pure-Rust GIF exporter, and auto-ducking audio mixer.
- `crates/fovea-ai`: Sandboxed `claude`/`agy` CLI & HTTP chat providers, Vbee TTS client, and on-demand local Whisper CoreML transcriber.
- `crates/fovea-mcp`: Live MCP server (`rmcp`) bridged via local Unix socket to the active Fovea Studio window.
- `crates/fovea-app`: GPUI (`gpui-kit`) desktop Studio UI and CLI entrypoint (`fovea`, `fovea mcp`).

## Development (macOS 14+)

```sh
cargo test --workspace
cargo run -p fovea-app
```

## License

MIT
