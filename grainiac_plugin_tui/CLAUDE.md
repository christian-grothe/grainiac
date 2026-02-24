# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is **Grainiac**, a granular sampler instrument built with NIH-plug, using Ratatui for its plugin editor UI. It exports as CLAP and VST3 formats and also runs standalone. The TUI is rendered into a pixel buffer via `soft_ratatui`'s `SoftBackend` + `softbuffer`.

## Build Commands

```bash
# Build (debug)
cargo build

# Build and bundle as CLAP/VST3 plugins (release)
cargo xtask bundle nih_ratatui --release

# Run standalone version
cargo run

# Check without compiling
cargo check

# Run tests
cargo test

# Lint
cargo clippy

# Format
cargo fmt
```

## Architecture

### Source Files
- `src/lib.rs` — `Grainiac` plugin struct, `GrainiacParams`, `InstanceParams`, `FileMessage` enum, and NIH-plug trait impls (`Plugin`, `ClapPlugin`, `Vst3Plugin`)
- `src/ratavstui.rs` — Editor implementation: `RatatuiEditor` (NIH-plug `Editor` trait), `RatatuiWindowHandler` (baseview `WindowHandler`), `RatatuiState` (persisted window size/open status)
- `src/track_widget.rs` — Ratatui `Widget` impl for `Track`: renders waveform, active grains, play heads, loop markers, and parameter readouts
- `src/track_widget/brailles.rs` — Braille character lookup tables (11 amplitude states × 5 rows per column) used to draw the waveform
- `src/main.rs` — Standalone entry point via `nih_export_standalone!`

### Plugin Structure
- `Grainiac` holds `Arc<GrainiacParams>`, a `Sampler` (from `grainiac_core`), a softbuffer `Output` for draw data, and a `crossbeam` channel for `FileMessage` (audio loading)
- `GrainiacParams` contains `editor_state: Arc<RatatuiState>` (persisted) and two `InstanceParams` instances (`[InstanceParams; 2]`) using `#[nested(array)]`
- `InstanceParams` exposes all per-instance parameters: loop_start/length, play_speed, density, spray, grain_length, attack, release, pitch, gain, pan, spread, g_dir, p_dir, hold

### Ratatui / Rendering Pipeline
1. `RatatuiWindowHandler::new()` creates a `SoftBackend<EmbeddedGraphics>` sized in character cells using **8×13 px** per cell (mono_8x13 font atlas)
2. `softbuffer::Surface` is sized to `cols*8 × rows*13` pixels
3. Each frame in `on_frame()`: draw via `terminal.draw(|f| ...)`, extract RGB pixels from `backend.get_pixmap_data()`, pack as `0xFF_RR_GG_BB` u32 values, copy into softbuffer
4. On resize: recalculate cols/rows from pixel size, call `backend_mut().resize()` then `terminal.resize()`
5. `DrawData` (from `grainiac_core`) is passed from the audio thread via `Arc<Mutex<Output<Vec<DrawData>>>>` and read in `on_frame()`

### Key Dependencies
- **nih_plug** (git) — plugin framework with `assert_process_allocs` + `standalone` features
- **baseview** (git, pinned rev) — cross-platform windowing
- **softbuffer 0.3.4** — CPU pixel buffer; must stay at 0.3.x to match baseview's `raw-window-handle = "0.5"`
- **soft_ratatui 0.1.3** — provides `SoftBackend` and embedded-graphics font atlases
- **ratatui 0.30** — TUI widgets/layout
- **grainiac_core** (path: `../grainiac/grainiac_core/`) — `Sampler`, `DrawData`, `Output`, `PlayDirection`, `Mode`
- **crossbeam** — `AtomicCell` for thread-safe state, bounded channel for `FileMessage`

## Important Notes

- Rust edition 2024; crate-type is `["cdylib", "lib"]`
- Git dependencies require internet on first build
- `FileMessage::LoadAudio` sends pre-loaded `Vec<f32>` samples to the audio thread; `FileMessage::OpenFileDialog` is defined but not yet handled
- The `Track` widget uses braille characters for a 5-row-tall waveform display; loop start marker is red, loop end is green, play head and active grains are yellow
- `RatatuiEditorHandle` closes the window on drop; `unsafe impl Send` is required because `WindowHandle` is not `Send`
