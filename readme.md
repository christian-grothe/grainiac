# Grainiac
Grainiac is a granular sampler that has a terminal user interface. The plugin project does not work at the moment because I am focussing on the standalone. For the standalone version I created a custom midi controller. 
At some point I am planning on releasing this as a DIY package where the sampler can run on a raspberry pi running the terminal user interface on a small lcd screen.
Currently it just works on Linux as a Jack Client.

[![Demo](https://img.youtube.com/vi/uADcnCzQX3A/0.jpg)](https://www.youtube.com/watch?v=uADcnCzQX3A)

# Config
You need to have a config.json file under _~/.config/grainiac/_ with the following schema:

```json
{
  "presets": [
    {
      "gain": [0.37301588, 0.73015875, 0.16666667, 0.0],
      "loop_start": [0.07936508, 0.24603175, 0.67460316, 0.071428575],
      "loop_length": [0.08730159, 0.08730159, 0.20634921, 0.3809524],
      "density": [15.873016, 28.968254, 31.349205, 0.5],
      "grain_length": [0.2777778, 0.11111111, 0.11904762, 0.48412699],
      "play_speed": [0.11111111, 0.33333334, 0.11111111, 1.0],
      "spray": [0.071428575, 0.071428575, 0.1, 0.1],
      "pan": [-0.079365075, 0.03174603, 0.0, 0.0],
      "spread": [0.6984127, 1.0, 1.0, 1.0],
      "attack": [5.0396824, 0.515873, 0.43650794, 0.25],
      "release": [5.0396824, 5.0396824, 4.7619047, 0.25],
      "pitch": [0, -12, 0, 0],
      "play_dir": [0, 0, 0, 0],
      "grain_dir": [0, 0, 1, 0],
      "name": "preset_1",
      "char": "1"
    },
  ],
  "mapping": {
    "loop_start": 1,
    "loop_length": 2,
    "density": 3,
    "grain_length": 4,
    "play_speed": 5,
    "spray": 6,
    "pan": 9,
    "spread": 10,
    "attack": 8,
    "release": 7,
    "pitch": 11,
    "gain": 12,
    "record": 13,
    "hold": 14,
    "play_dir": 15,
    "grain_dir": 16
  }
}
```

The arrays under presets represent the tracks from A to D.
The name field is currently not used.
the char field is the character that you press to load the preset.

The mapping section is to map midi cc to certain parameters. Currently every track of the sampler is mapped to a unique midi channel. So with the JSON above loop_start is mapped to midi cc 1 and channel 1 is changing track A, channel 2 track B and so on.

## BOM
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
