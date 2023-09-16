This is a wgpu reproduction of the Metal bug described in
https://github.com/armansito/metal-workgroup-memory-bug.

## Results on native
On macOS using Metal on Apple M1 GPUs, this program results in 10 while the expected result is 640.

On Linux using Vulkan on a NVIDIA RTX 4090, this program results in 640, which is correct.

## Result in Chrome (WASM) on macOS

On WebGPU with Chrome (using Dawn's implementation) this results in 640, which is correct.

However, a modified version of the program using a matrix type in workgroup shared memory
results in 10 (see `shader-broken-on-dawn.wgsl`).

## Running the test
To run the test natively:
```
$ cargo run
```

For WASM, run the following commands to build and host a page that you can run in a browser that
supports WebGPU). Navigate your browser to http://localhost:8000 to view the page.
```
$ RUSTFLAGS=--cfg=web_sys_unstable_apis wasm-pack build --target web --release
$ python -m http.server
```
