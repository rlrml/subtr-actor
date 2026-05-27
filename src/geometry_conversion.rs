pub fn vec_to_glam(v: &boxcars::Vector3f) -> glam::f32::Vec3 {
    glam::f32::Vec3::new(v.x, v.y, v.z)
}

pub fn glam_to_vec(v: &glam::f32::Vec3) -> boxcars::Vector3f {
    boxcars::Vector3f {
        x: v.x,
        y: v.y,
        z: v.z,
    }
}

pub fn quat_to_glam(q: &boxcars::Quaternion) -> glam::Quat {
    glam::Quat::from_xyzw(q.x, q.y, q.z, q.w)
}

pub fn glam_to_quat(rotation: &glam::Quat) -> boxcars::Quaternion {
    boxcars::Quaternion {
        x: rotation.x,
        y: rotation.y,
        z: rotation.z,
        w: rotation.w,
    }
}
