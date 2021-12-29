use crate::types::*;

#[derive(Debug)]
pub struct ParticleFactory {
	pub nparticles: isize,

	/// Return values from NvFlexMap(...)
	buffer: *mut Vector4,
	velocities: *mut Vector3,
	phases: *mut i32,
	active_indices: *mut i32
}

impl ParticleFactory {
	pub fn new(offset: Option<isize>, buffer: *mut Vector4, velocities: *mut Vector3, phases: *mut i32, indices: *mut i32) -> Self {
		Self {
			nparticles: offset.unwrap_or(0_isize),

			buffer,
			velocities,
			phases,
			active_indices: indices
		}
	}

	pub fn create(&mut self, pos: Vector4, velocity: Vector3, phase: i32, _active: bool){
		let index = self.nparticles;

		unsafe {
			self.buffer
				.offset(index)
				.write(pos);

			self.velocities
				.offset(index)
				.write(velocity);

			self.phases
				.offset(index)
				.write(phase);

			// Assumes particle is active for now.
			self.active_indices
				.offset(index)
				.write(index as i32);
		}
		self.nparticles += 1;
	}
}