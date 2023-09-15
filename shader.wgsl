@group(0) @binding(0) var<uniform> flag: u32;
@group(0) @binding(1) var<storage, read_write> output: atomic<u32>;

var<workgroup> shared_flag: u32;

@compute @workgroup_size(64)
fn main(@builtin(local_invocation_id) local_id: vec3<u32>) {
    shared_flag = 0xffffffffu;
    workgroupBarrier();

    if local_id.x == 0u {
        shared_flag = flag;
    }

    let abort = workgroupUniformLoad(&shared_flag);
    if abort != 0u {
        return;
    }

    atomicAdd(&output, 1u);
}
