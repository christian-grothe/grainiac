# Grainiac

Grainiac is a granular sampler that has a terminal user interface. The plugin project does not work at the moment because I am focussing on the standalone. For the standalone version I created a custom midi controller.
At some point I am planning on releasing this as a DIY package where the sampler can run on a raspberry pi running the terminal user interface on a small lcd screen.
Currently it just works on Linux as a Jack Client.

[![Demo#1](https://img.youtube.com/vi/uADcnCzQX3A/0.jpg)](https://www.youtube.com/watch?v=XzJnMVo1ZkM)
[![Demo#2](https://img.youtube.com/vi/uADcnCzQX3A/0.jpg)](https://www.youtube.com/watch?v=uADcnCzQX3A)

# Config

You need to have a config.json file under ~/.config/grainiac/ with the following schema:

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

The arrays under presets represent the tracks from A to D.
The name field is currently not used.
the char field is the character that you press to load the preset.

The mapping section is to map midi cc to certain parameters. Currently every track of the sampler is mapped to a unique midi channel. So with the JSON above loop_start is mapped to midi cc 1 and channel 1 is changing track A, channel 2 track B and so on.

In order to save and load audios you need to manually create the folder ~/.local/share/grainiac/

# Mappings
|   Key     |   Function    |
| --------- | ------------- |
|   m       |   change mode |
|   0..9    |   load / save |
|   n       |   switch view | 
|   esc     |   close       |


# BOM

This is the bill of material that I use for the midi controller and the raspberry pi.

| Part           | Price | Quantity | Total |
| -------------- | ----- | -------- | ----- |
| RaspberryPi 5  | 67,40 | 1        | 67.40 |
| LCD Screen     | 70    | 1        | 70    |
| Teensy 4.0     | 33    | 1        | 33    |
| 4051 MUX       | 0.60  | 8        | 4.8   |
| Potis          | 0.99  | 4 x 12   | 47.52 |
| Tactile Switch |       | 4 x 4    |       |
| USB Audio      | 11    | 1        | 11    |
| PSU            | 12.65 | 1        | 12.65 |
| PCB            |       |          |       |

**_TOTAL: 246.37_**
