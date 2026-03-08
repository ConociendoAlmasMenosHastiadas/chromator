# Chromator

A simple HSV color picker built with Rust and [egui](https://github.com/emilk/egui).

## Features

- **SV Plane** — smooth 2D saturation-value gradient rendered as a GPU texture
- **Hue Bar** — full-spectrum hue selector
- **Color output** — displays the picked color as:
  - RGB tuple
  - CMYK tuple
  - HSV tuple
  - HSL tuple
  - Hex string (uppercase)

## Building

```
cargo build --release
```

## Running

```
cargo run --release
```

## License

MIT
