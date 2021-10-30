use cgmath::InnerSpace;
use cgmath::SquareMatrix;

use crate::components::*;
use crate::OPENGL_TO_WGPU_MATRIX;

#[legion::system(par_for_each)]
pub fn look_at(
    EyePosition(eye_position): &EyePosition,
    LookAt(look_at): &LookAt,
    UpVector(up_vector): &UpVector,
    view_matrix: &mut ViewMatrix,
) {
    **view_matrix = cgmath::Matrix4::look_at_rh(*eye_position, *look_at, *up_vector);
}

#[legion::system(par_for_each)]
pub fn look_at_quat(
    EyePosition(eye_position): &EyePosition,
    LookAt(look_at): &LookAt,
    UpVector(up_vector): &UpVector,
    orientation: &mut Orientation,
) {
    let look_to = (look_at - eye_position).normalize();
    let mat = cgmath::Matrix3::look_to_rh(look_to, *up_vector);
    orientation.set_checked(mat.into());
}

#[legion::system(par_for_each)]
pub fn look_to(
    EyePosition(eye_position): &EyePosition,
    LookTo(look_to): &LookTo,
    UpVector(up_vector): &UpVector,
    view_matrix: &mut ViewMatrix,
) {
    **view_matrix = cgmath::Matrix4::look_to_rh(*eye_position, *look_to, *up_vector);
}

#[legion::system(par_for_each)]
pub fn look_to_quat(
    EyePosition(_): &EyePosition,
    LookTo(look_to): &LookTo,
    UpVector(up_vector): &UpVector,
    orientation: &mut Orientation,
) {
    let mat = cgmath::Matrix3::look_to_rh(*look_to, *up_vector);
    orientation.set_checked(mat.into());
}

#[legion::system(par_for_each)]
pub fn perspective_projection(
    _: &PerspectiveProjection,
    field_of_view: &FieldOfView,
    NearPlane(near_plane): &NearPlane,
    FarPlane(far_plane): &FarPlane,
    AspectRatio(aspect_ratio): &AspectRatio,
    projection_matrix: &mut ProjectionMatrix,
) {
    **projection_matrix = OPENGL_TO_WGPU_MATRIX * field_of_view.to_matrix(*aspect_ratio, *near_plane, *far_plane);
}

#[legion::system(par_for_each)]
pub fn orthographic_projection(
    orthographic_projection: &OrthographicProjection,
    NearPlane(near_plane): &NearPlane,
    FarPlane(far_plane): &FarPlane,
    projection_matrix: &mut ProjectionMatrix,
) {
    **projection_matrix = orthographic_projection.to_matrix(*near_plane, *far_plane);
}

#[legion::system(for_each)]
pub fn view_projection_matrix(
    projection_matrix: Option<&ProjectionMatrix>,
    view_matrix: Option<&ViewMatrix>,
    ViewProjectionMatrix(matrix): &mut ViewProjectionMatrix,
) {
    let mx_total = cgmath::Matrix4::<f32>::identity();

    let mx_total = if let Some(view_matrix) = view_matrix {
        (**view_matrix) * mx_total
    } else {
        mx_total
    };

    let mx_total = if let Some(projection_matrix) = projection_matrix {
        (**projection_matrix) * mx_total
    } else {
        mx_total
    };

    let mx_total = OPENGL_TO_WGPU_MATRIX * mx_total;
    matrix.set_checked(mx_total);
}
