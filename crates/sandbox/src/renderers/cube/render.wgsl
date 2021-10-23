[[block]]
struct Locals {
    position: vec4<f32>;
    orientation: Quaternion;
    projection: mat4x4<f32>;
};

struct VertexInput {
    [[location(0)]] position: vec4<f32>;
    [[location(1)]] tex_coord: vec2<f32>;
    [[location(2)]] texture: i32;
};

struct InstanceInput {
    [[location(3)]] position: vec4<f32>;
    [[location(4)]] orientation: vec4<f32>;
};

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] tex_coord: vec2<f32>;
    [[location(1)]] texture: i32;
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

    let model_tx = model.position;
    let model_tx = quat_mul(instance_orientation, model_tx.xyz);
    let model_tx = model_tx.xyz + instance.position.xyz;

    let model_tx = model_tx.xyz - r_locals.position.xyz;
    let model_tx = quat_mul(r_locals.orientation, model_tx.xyz);

    var out: VertexOutput;
    out.position = r_locals.projection * vec4<f32>(model_tx, model.position.w);
    out.tex_coord = model.tex_coord;
    out.texture = model.texture;
    return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let tex = textureLoad(r_color, vec2<i32>(in.tex_coord * 256.0), 0, in.texture);
    let v = f32(tex.x) / 255.0;
    return vec4<f32>(1.0 - (v * 5.0), 1.0 - (v * 15.0), 1.0 - (v * 50.0), 1.0);
}

[[stage(fragment)]]
fn fs_wire() -> [[location(0)]] vec4<f32> {
    return vec4<f32>(0.0, 0.5, 0.0, 0.5);
}
