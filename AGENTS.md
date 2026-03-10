# Image Kit Architecture & Guidelines

## Tech Stack

Frontend: React, TypeScript, Vite, Tailwind CSS (for styling).

WASM Core: Rust, wasm-bindgen, image crate.

Package Manager: npm.

## Architecture

The application is a browser-based image utility.

The React frontend handles the UI (canvas settings, text settings, export settings).

The UI passes configuration parameters (text string, font size, hex colors, canvas width/height, rotation angle, spacing) to the WebAssembly module.

The Rust/WASM module generates the image buffer with the repeating text pattern and returns it to the frontend as a byte array.

The frontend displays the generated image in an <img> tag for preview, and handles the download logic.

## Rules for Jules

Write modular, strongly typed code.

Ensure all WASM functions exposed to JavaScript have clear documentation.

Run npm run build and wasm-pack build to verify the build before submitting a PR.
