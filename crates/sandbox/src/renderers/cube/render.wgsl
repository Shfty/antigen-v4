[[block]]
struct Locals {
    position: vec4<f32>;
    orientation: Quaternion;
    projection: mat4x4<f32>;
};

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] normal: vec3<f32>;
    [[location(2)]] tex_coord: vec2<f32>;
    [[location(3)]] texture: i32;
};

struct InstanceInput {
    [[location(4)]] position: vec4<f32>;
    [[location(5)]] orientation: vec4<f32>;
};

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] normal: vec3<f32>;
    [[location(1)]] tex_coord: vec2<f32>;
    [[location(2)]] texture: i32;
};

[[group(0), binding(0)]]
var<uniform> r_locals: Locals;

[[group(0), binding(1)]]
var r_color: texture_2d_array<u32>;

[[stage(vertex)]]
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let instance_orientation = quat_from_vec4(instance.orientation);

    let model_pos = model.position;
    let model_pos = quat_mul(instance_orientation, model_pos);
    let model_pos = model_pos + instance.position.xyz;

    let model_pos = model_pos - r_locals.position.xyz;
    let model_pos = quat_mul(r_locals.orientation, model_pos);

    let model_normal = model.normal;
    let model_normal = quat_mul(instance_orientation, model_normal);

    return VertexOutput (
        r_locals.projection * vec4<f32>(model_pos, 1.0),
        model_normal,
        model.tex_coord,
        model.texture,
    );
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let uv = vec2<i32>((abs(in.tex_coord) * 256.0) % 256.0);
    let tex = textureLoad(r_color, uv, in.texture, 0);
    let v = f32(tex.x) / 255.0;
    let color = vec4<f32>(1.0 - (v * 5.0), 1.0 - (v * 15.0), 1.0 - (v * 50.0), 1.0);
    let light = (dot(normalize(vec3<f32>(1.0)), in.normal) + 1.0) * 0.5;
    return color * light;
}

[[stage(fragment)]]
fn fs_wire() -> [[location(0)]] vec4<f32> {
    return vec4<f32>(0.0, 0.5, 0.0, 0.5);
}
