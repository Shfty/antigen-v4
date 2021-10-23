[[block]]
struct Uniforms {
    position: vec4<f32>;
    orientation: Quaternion;
    projection: mat4x4<f32>;
};

struct Instance {
    position: vec4<f32>;
    orientation: Quaternion;
    visible: u32;
    radius: f32;
};

[[block]]
struct Instances {
    instances : [[stride(48)]] array<Instance>;
};

struct Indirect {
    vertex_count: u32;
    instance_count: u32;
    base_index: u32;
    vertex_offset: i32;
    base_instance: u32;
};

[[block]]
struct Indirect {
    indirect : [[stride(20)]] array<Indirect>;
};

[[group(0), binding(0)]] var<uniform> uniforms : Uniforms;
[[group(0), binding(1)]] var<storage, read> instances : Instances;
[[group(0), binding(2)]] var<storage, read_write> indirect : Indirect;

[[stage(compute), workgroup_size(64)]]
fn main([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
    let total = arrayLength(&instances.instances);
    let index = global_invocation_id.x;
    if (index >= total) {
        return;
    }

    // Extract clipping planes
    let frustum = frustum_from_projection_matrix(uniforms.projection);
    let frustum = frustum_normalize(frustum);

    // Fetch instance data
    let instance = instances.instances[index];

    // Transform position
    let model_tx = instance.position.xyz;
    let model_tx = model_tx - uniforms.position.xyz;
    let model_tx = quat_mul(uniforms.orientation, model_tx.xyz);

    // Cull
    if(instance.visible == 0u || frustum_outside(frustum, model_tx, instance.radius)) {
        indirect.indirect[index].instance_count = 0u;
    }
    else {
        indirect.indirect[index].instance_count = 1u;
    }
}
