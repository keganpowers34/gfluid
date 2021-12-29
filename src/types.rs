#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Vector3(pub f32, pub f32, pub f32);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
/// x, y, z, imass
pub struct Vector4(pub f32, pub f32, pub f32, pub f32);

#[derive(Debug)]
pub struct Particle<'a> {
	pub pdata: &'a Vector4,
	pub velocity: &'a Vector3,
	pub phase: &'a i32,
}

// xyzw
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Quat(pub f32, pub f32, pub f32, pub f32);
