This is a wgpu reproduction of the Metal bug described in
https://github.com/armansito/metal-workgroup-memory-bug.

On macOS using Metal on Apple M1 GPUs, this program results in 10 while the expected result is 640.
