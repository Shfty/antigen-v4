struct Frustum {
    left: Plane;
    right: Plane;
    top: Plane;
    bottom: Plane;
    near: Plane;
    far: Plane;
};

fn frustum_normalize(f: Frustum) -> Frustum {
    return Frustum(
        plane_normalize(f.left),
        plane_normalize(f.right),
        plane_normalize(f.top),
        plane_normalize(f.bottom),
        plane_normalize(f.near),
        plane_normalize(f.far),
    );
}

fn frustum_from_projection_matrix(p: mat4x4<f32>) -> Frustum {
    return Frustum(
        Plane(
            p[3][0] + p[0][0],
            p[3][1] + p[0][1],
            p[3][2] + p[0][2],
            p[3][3] + p[0][3],
        ),
        Plane(
            p[3][0] - p[0][0],
            p[3][1] - p[0][1],
            p[3][2] - p[0][2],
            p[3][3] - p[0][3],
        ),
        Plane(
            p[3][0] - p[1][0],
            p[3][1] - p[1][1],
            p[3][2] - p[1][2],
            p[3][3] - p[1][3],
        ),
        Plane(
            p[3][0] + p[1][0],
            p[3][1] + p[1][1],
            p[3][2] + p[1][2],
            p[3][3] + p[1][3],
        ),
        Plane(
            p[3][0] + p[2][0],
            p[3][1] + p[2][1],
            p[3][2] + p[2][2],
            p[3][3] + p[2][3],
        ),
        Plane(
            p[3][0] - p[2][0],
            p[3][1] - p[2][1],
            p[3][2] - p[2][2],
            p[3][3] - p[2][3],
        ),
    );
}

fn frustum_outside(f: Frustum, p: vec3<f32>, r: f32) -> bool {
    return (
        plane_dist(f.left, p) + r < 0.0 ||
        plane_dist(f.right, p) + r < 0.0 ||
        plane_dist(f.top, p) + r < 0.0 ||
        plane_dist(f.bottom, p) + r < 0.0 ||
        plane_dist(f.near, p) + r < 0.0 ||
        plane_dist(f.far, p) + r < 0.0
    );
}

