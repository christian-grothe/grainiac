# Grainiac

Grainiac is a granular sampler. It currently runs as a vst3 plugin, standlone with a tui and a vst3 plugin with a tui style interface which is currently experimental and under development.

[![Demo#1](https://img.youtube.com/vi/uADcnCzQX3A/0.jpg)](https://www.youtube.com/watch?v=XzJnMVo1ZkM)
[![Demo#2](https://img.youtube.com/vi/uADcnCzQX3A/0.jpg)](https://www.youtube.com/watch?v=uADcnCzQX3A)

## Project Structure

| Crate                 | Description                                               |
| --------------------- | --------------------------------------------------------- |
| `grainiac_core`       | Core audio DSP engine (no UI dependencies)                |
| `grainiac_tui`        | Standalone JACK client with Ratatui TUI                   |
| `grainiac_plugin_tui` | VST3/CLAP plugin with Ratatui TUI editor _(experimental)_ |
| `grainiac_plugin_gui` | Vizia-based GUI plugin                                    |
| `midi_ctrl`           | PlatformIO firmware for Teensy 4.0 MIDI controller        |

## Config

You need a `config.json` file under `~/.config/grainiac/` with the following schema:

```json
{
  "presets": [
    {
      "gain": [0.5, 0.5, 0.5, 0.5],
      "loop_start": [0.25, 0.25, 0.25, 0.25],
      "loop_length": [0.5, 0.5, 0.5, 0.5],
      "density": [0.5, 0.5, 0.5, 0.5],
      "grain_length": [0.5, 0.5, 0.5, 0.5],
      "play_speed": [1.0, 1.0, 1.0, 1.0],
      "spray": [0.1, 0.1, 0.1, 0.1],
      "pan": [0.0, 0.0, 0.0, 0.0],
      "spread": [1.0, 1.0, 1.0, 1.0],
      "attack": [0.25, 0.25, 0.25, 0.25],
      "release": [0.25, 0.25, 0.25, 0.25],
      "pitch": [1, 1, 1, 1],
      "play_dir": [0, 0, 0, 0],
      "grain_dir": [0, 0, 0, 0],
      "mode": [0, 0, 0, 0],
      "name": "preset_1",
      "char": "1"
    }
  ],
  "mapping": {
    "loop_start": 40,
    "loop_length": 41,
    "density": 42,
    "grain_length": 43,
    "play_speed": 44,
    "spray": 45,
    "pan": 46,
    "spread": 47,
    "attack": 48,
    "release": 49,
    "pitch": 50,
    "gain": 51,
    "record": 52,
    "hold": 53,
    "play_dir": 54,
    "grain_dir": 55,
    "mode": 56
  }
}
```

The arrays under presets represent tracks A to D. The `char` field is the key to press to load the preset. The `name` field is currently unused.

The `mapping` section maps MIDI CC numbers to parameters. Each track corresponds to a MIDI channel (channel 1 = track A, channel 2 = track B, etc.).

To save and load audio files, manually create the folder `~/.local/share/grainiac/`.

## Key Mappings

| Key    | Function    |
| ------ | ----------- |
| `m`    | change mode |
| `0..9` | load / save |
| `n`    | switch view |
| `esc`  | close       |
