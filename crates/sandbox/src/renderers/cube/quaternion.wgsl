struct Quaternion {
    x: f32;
    y: f32;
    z: f32;
    w: f32;
};

fn quat_from_vec4(v: vec4<f32>) -> Quaternion {
    return Quaternion(
        v.x,
        v.y,
        v.z,
        v.w,
    );
}

fn quat_inv(q: Quaternion) -> Quaternion {
    return Quaternion(-q.x, -q.y, -q.z, q.w);
}

fn quat_dot(q1: Quaternion, q2: Quaternion) -> Quaternion {
    let q1_xyz = vec3<f32>(q1.x, q1.y, q1.z);
    let q2_xyz = vec3<f32>(q2.x, q2.y, q2.z);

    let scalar = q1.w * q2.w - dot(q1_xyz, q2_xyz);
    let v = cross(q1_xyz, q2_xyz) + q1.w * q2_xyz + q2.w * q1_xyz;

    return Quaternion(v.x, v.y, v.z, scalar);
}

fn quat_mul(q: Quaternion, v: vec3<f32>) -> vec3<f32> {
    let r = quat_dot(q, quat_dot(Quaternion(v.x, v.y, v.z, 0.0), quat_inv(q)));
    return vec3<f32>(r.x, r.y, r.z);
}

