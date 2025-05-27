# About

A simple graphics demo where you can fly around, spawn boxes and move them.

I made this project to learn Rust and try it in game development. There is no attempt to create an "engine", everything
is pretty low level and abstractions are built along the way when needed.

![Demo](/screenshot.png?raw=true)

## Building and running

```
cargo run
```

Tested and _should_ work on macOS, Windows and Linux 🙈

## Features

- ECS: [hecs](https://github.com/Ralith/hecs).
- Rendering: [wgpu](https://github.com/gfx-rs/wgpu).
- UI: [imgui](https://github.com/Yatekii/imgui-wgpu-rs). 
- Math: [nalgebra](https://github.com/dimforge/nalgebra).
- Physics: [Rapier](https://rapier.rs)
    - Rigid bodies with colliders.
    - Camera with character controller, preventing it from passing through objects.
    - Ray casting.
    - Drag-n-drop.
- First person flying camera ("spectator") with protection from overturning.
- Skybox rendering on a full-screen quad.
- Vignette post-processing.
