# Pathfinding Playground

### Visualizer for pathfinding algorithms
Visualizer for pathfinding algorithms powered by WASM. You can use ANY language (supported by WIT) to generate API bindings, run your pathfinding algorithm and see how it works 

![image](https://imgur.com/F862GbP.png)

# Technologies

- [Rust](https://rust-lang.org/)
- [Bevy 0.17.3](https://bevy.org/)
- [Wasmtime (WebAssembly)](https://wasmtime.dev/)
- [Wasm Component Model (WIT)](https://component-model.bytecodealliance.org/)

# How to use

1. Clone the repository
2. Use [wit-bindgen](https://github.com/bytecodealliance/wit-bindgen) to generate bindings for your language
3. Compile into .wasm with WIT support
4. Run this project with `cargo run`
5. Select algorithm

# Controls
### Mouse:
* Left click - Place walls
* Right click - Remove walls
* Middle click - Move camera
* Scroll wheel - Zoom
* Left click drag on goals - move goals

### Goals:
* Fox - start position
* Flag - end position

# TODO

0. Decouple visual representation from actual data (for map)
1. Hot-reloading
2. Pre-compiled binaries
3. More examples
4. Detect if path is actually valid
5. See TimelineActions in realtime
6. Confetti!
7. load_map in wit
8. Animate fox
9. Coins as more goals
