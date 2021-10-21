[[block]]
struct Uniforms {
    vp_mx: mat4x4<f32>;
};

struct Instance {
    mx: mat4x4<f32>;
};

[[block]]
struct Instances {
    instances : [[stride(64)]] array<Instance>;
};

struct DrawIndexedIndirect {
    vertex_count: u32;   // The number of vertices to draw.
    instance_count: u32; // The number of instances to draw.
    base_index: u32;     // The base index within the index buffer.
    vertex_offset: i32; // The value added to the vertex index before indexing into the vertex buffer.
    base_instance: u32; // The instance ID of the first instance to draw.
};

[[block]]
struct Indirect {
    indirect : [[stride(20)]] array<DrawIndexedIndirect>;
};

[[group(0), binding(0)]] var<uniform> uniforms : Uniforms;
[[group(0), binding(1)]] var<storage, read_write> instances : Instances;
[[group(0), binding(2)]] var<storage, write> indirect : Indirect;

[[stage(compute), workgroup_size(64)]]
fn main([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
    let total = arrayLength(&instances.instances);
    let index = global_invocation_id.x;
    if (index >= total) {
        return;
    }

    // Fetch instance
    let instance = instances.instances[index];

    // Project instance transform to NDC
    let mx = uniforms.vp_mx * instance.mx;

    // Extract position
    let pos = mx[3].xyz / mx[3].w;

    // Frustum cull
    if (abs(pos.x) > 1.0 || abs (pos.y) > 1.0) {
        instances.instances[index].mx = mat4x4<f32>(
            vec4<f32>(1.0, 0.0, 0.0, 0.0),
            vec4<f32>(0.0, 1.0, 0.0, 0.0),
            vec4<f32>(0.0, 0.0, 1.0, 0.0),
            vec4<f32>(0.0, 0.0, 0.0, 1.0),
        );
    }
}
