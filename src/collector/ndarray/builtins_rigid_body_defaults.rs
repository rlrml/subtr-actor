use super::*;

pub(crate) fn default_rb_state<F: TryFrom<f32>>() -> RigidBodyArrayResult<F>
where
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    convert_all!(
        convert_float_conversion_error,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
    )
}

pub(crate) fn default_rb_state_quaternion<F: TryFrom<f32>>() -> RigidBodyQuaternionArrayResult<F>
where
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    convert_all_floats!(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,)
}

pub(crate) fn default_rb_state_basis<F: TryFrom<f32>>() -> RigidBodyBasisArrayResult<F>
where
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    convert_all_floats!(0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,)
}

pub(crate) fn default_rb_state_no_velocities<F: TryFrom<f32>>() -> SubtrActorResult<[F; 7]>
where
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    convert_all_floats!(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,)
}
