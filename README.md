# Purdy

Purdy is an experimental PDF renderer built on top of WebGPU.

Today Purdy is too nascent to be used in any real application. Eventually the goal is to have a PDF renderer that compiles to WebAssembly and can be used in a variety of applications, browser and otherwise.

## Current Status

The current focus is to enable PDF graphics commands (drawing lines, shapes, etc.).

For this we rely on [lyon](https://github.com/nical/lyon) to tesselate paths which are then fed to `wgpu` for rendering.

## How to run Purdy

Most of the work on purdy is being done through `examples/playground/src/main.rs`. As concepts are solidified there, they are then moved over to the core library in `crates`.

`examples/playground` attempts to render `pdfs/sample-no-xref-entries/sample-no-xref-entries.pdf`, which has some text on pages one and two, and has some shapes on page 2. Right now, only the shapes are rendered.

To run Purdy, clone this repo and run `make eg1` from the repo root. Once the program has finished, you can open `examples/playground/img/page-2.png` to see the output.

## Contributing

PDF rendering is a massive undertaking, so contributors are welcome! There are no contributing guidelines so your best bet is to open an issue if you're trying to figure out where to start.
