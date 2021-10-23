struct Plane {
    x: f32;
    y: f32;
    z: f32;
    d: f32;
};

fn plane_normalize(p: Plane) -> Plane {
    let n = normalize(vec3<f32>(p.x, p.y, p.z));
    return Plane(n.x, n.y, n.z, p.d);
}

fn plane_dist(p: Plane, v: vec3<f32>) -> f32 {
    return dot(v, vec3<f32>(p.x, p.y, p.z)) + p.d;
}

