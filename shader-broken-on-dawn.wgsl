@group(0) @binding(0) var<uniform> flag: u32;
@group(0) @binding(1) var<storage, read_write> output: atomic<u32>;

var<workgroup> shared_flag: mat2x2<f32>;

@compute @workgroup_size(64)
fn main(@builtin(local_invocation_id) local_id: vec3<u32>) {
    shared_flag[0].x = 1.0;
    workgroupBarrier();

    if local_id.x == 0u {
        shared_flag[0].x = f32(flag);
    }

    let abort_mat = workgroupUniformLoad(&shared_flag);
    let abort = abort_mat[0].x;
    if abort != 0.0 {
        return;
    }

    atomicAdd(&output, 1u);
}
