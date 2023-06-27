# map-engine-server

[![Gitlab pipeline status](https://gitlab.com/spadarian/map_engine/badges/master/pipeline.svg)](https://gitlab.com/spadarian/map-engine)
[![Gitlab code coverage](https://gitlab.com/spadarian/map_engine/badges/master/coverage.svg)](https://gitlab.com/spadarian/map-engine)
[![Crates.io](https://img.shields.io/crates/v/map-engine.svg)](https://crates.io/crates/map-engine-server)
[![Documentation](https://docs.rs/map-engine/badge.svg)](https://docs.rs/map-engine)

An HTTP map tile server.

It serves tiles following the XYZ scheme (e.g. http://{host}/{map_name}/{z}/{x}/{y}.{ext}).

The sources should be defined in a json config file like this:

```json
[
    {
        "name": "chile_optimised",
        "path": "/path/to/file.tif",
        "style": {
            "name": "viridis"
            "vmin": 0.0,
            "vmax": 27412.0
            "bands": [1],
        },
    }
]
```

that should be passed as a parameter when running the server:

```bash
map-engine-server --config config_file.json
```

### Supported styles

You can define different styles depending on what you want to visualise and how much customisation you need.

The RGBA colours can be specified in multiple ways. Here are some examples for black:

* As components in the range 0.0..=1.0: `[0.0, 0.0, 0.0, 1.0]`
* As components in the range 0..=255: `[0, 0, 0, 255]`
* As [hex triplets](https://en.wikipedia.org/wiki/Web_colors#Hex_triplet) + opacity in multiple formats:
    - `"#000000ff"`. The `#` is optional.
    - `"000000"`. Since opacity is missing, we assume `ff`.
    - `"000000FF"`. Here we set the opacity explicitly.
    - `"000000ff"`. Lowercase characters are also valid.

#### Using a named palette

Available options are the names of functions provided in the [`map_engine::cmap`](https://docs.rs/map-engine/latest/map-engine/cmap/index.html#functions) module

At the moment, this option is only available to visualise a single, continuous band. Eventually we will add more palettes, both continuous and discrete.

```json
"style": {
    "name": "viridis"
    "vmin": 0.0,
    "vmax": 100.0
    "bands": [1]
}
```
#### Using custom equally-spaced colours for a single, continuous band

Pixels between 0.0 and 100.0 mapped to a red-to-blue gradient:

```json
"style": {
    "colours": [
        "FF0000",
        [0, 0, 255, 255]
    ],
    "vmin": 0.0,
    "vmax": 100.0
    "bands": [1]
}
```

#### Using custom breaks for a single, continuous band

Pixels between 0.0 and 100.0 mapped to a red-to-blue gradient but purple shifted towards red:

```json
"style": {
    "colours": [
        [0.0, "FF0000"],
        [25.0, [127, 0, 127, 255]],
        [100.0, "0000FF"]
    ],
    "bands": [1]
}
```

#### Colours for a single, discrete band

Pixels between 1 and 5 mapped to different colours. Pixel values that are not defined will be fully transparent:

```json
"style": {
    "colours": [
        [1, "#AA0000"],
        [2, "0000ff"],
        [3, [0.0, 0.5, 0.0, 1.0]],
        [4, [255, 255, 0, 255]],
        [5, "0000ff66"]
    ],
    "bands": [1]
}
```

#### Colour composites for multiple continuous bands

[False colour](https://en.wikipedia.org/wiki/False_color) Landsat-8 composite:

```json
"style": {
      "colours": [
          [0.0261112, 0.035925, 0.035925],
          [0.312009, 0.125154, 0.121313]
      ],
      "bands": [5, 4, 3]
}
```

### Previewing tiles

When you launch the tile server you should be able to request individual tiles using an URL like this: http://localhost:8080/{map_name}/{z}/{x}/{y}.png. We also serve a webmap where you can preview your tiles. You can access the previewer at http://localhost:8080/{map_name}.

![A screenshot of the map previewer generated with MapEngine](https://gitlab.com/spadarian/map_engine/uploads/01b0e623d290774d218460483d7620aa/image.png)
