

use nvflex_sys::*;

use crate::{config, types::*};
use std::mem::size_of;

mod factory;

#[derive(derivative::Derivative)]
#[derivative(Debug)]
pub struct ParticleState {
	has_changes: bool,

	count: i32,
	active: Vec<i32>,

	pub buffer: *mut NvFlexBuffer,
	pub velocities: *mut NvFlexBuffer,
	pub phases: *mut NvFlexBuffer,
	pub active_indices: *mut NvFlexBuffer,
}

impl Default for ParticleState {
	fn default() -> Self {
		Self {
			has_changes: false,

			count: 0,
			active: vec![],

			buffer: std::ptr::null_mut(),

			velocities: std::ptr::null_mut(),
			phases: std::ptr::null_mut(),
			active_indices: std::ptr::null_mut(),
		}
	}
}

impl ParticleState {
	/// # Safety
	/// Do not call this function more than once
	pub unsafe fn alloc(&mut self, flex: *mut NvFlexLibrary) {
		self.buffer = NvFlexAllocBuffer(
			flex,
			config::MAX_PARTICLES,
			size_of::<Vector4>() as i32,
			eNvFlexBufferHost,
		);

		self.velocities = NvFlexAllocBuffer(
			flex,
			config::MAX_PARTICLES,
			size_of::<Vector3>() as i32,
			eNvFlexBufferHost,
		);

		self.phases = NvFlexAllocBuffer(
			flex,
			config::MAX_PARTICLES,
			size_of::<i32>() as i32,
			eNvFlexBufferHost,
		);

		self.active_indices = NvFlexAllocBuffer(
			flex,
			config::MAX_PARTICLES,
			size_of::<i32>() as i32,
			eNvFlexBufferHost,
		);
	}

	/// Adds a particle to FleX
	/// Note the changes won't be applied to flex immediately, you need to call [self.flush]
	/// Also this is very inefficient since it maps and unmaps every call..
	pub fn add_particle(
		&mut self,
		pos: Vector4,
		vel: Vector3,
		phase: i32,
		active: bool
	) {
		let i = self.count;
		let ind = i as isize;
		unsafe {
			let particles = NvFlexMap(self.buffer, eNvFlexMapWait) as *mut Vector4;
			let velocities = NvFlexMap(self.velocities, eNvFlexMapWait) as *mut Vector3;
			let phases = NvFlexMap(self.phases, eNvFlexMapWait) as *mut i32;
			let active_indices = NvFlexMap(self.active_indices, eNvFlexMapWait) as *mut i32;

			particles
				.offset(ind)
				.write(Vector4(50.0 * i as f32, 0.0, 5000.0, 2.0));

			velocities.offset(ind).write(Vector3(0.0, 0.0, -5.0));
			phases.offset(ind).write(phase);

			// Assume active for now.
			active_indices.offset(ind).write(i);

			self.unmap();
		}

		self.count += 1;
		self.has_changes = true;
	}

	pub fn unmap(&self) {
		unsafe {
			NvFlexUnmap(self.buffer);
			NvFlexUnmap(self.velocities);
			NvFlexUnmap(self.phases);
			NvFlexUnmap(self.active_indices);
		}
	}

	/// Gets a pointer to the particle buffer
	/// # Safety
	/// This function must be followed by a proper release, through self.unmap(), as this calls NvFlexMap
	pub unsafe fn get_particles(&self, solver: *mut NvFlexSolver) -> *mut Vector4 {
		NvFlexGetParticles(solver, self.buffer, std::ptr::null());
		NvFlexMap(self.buffer, eNvFlexMapWait) as *mut Vector4
	}

	/// Gets a pointer to the particle buffer
	/// # Safety
	/// This function must be followed by a proper release, through self.unmap(), as this calls NvFlexMap
	pub unsafe fn get_velocities(&self, solver: *mut NvFlexSolver) -> *mut Vector3 {
		NvFlexGetVelocities(solver, self.velocities, std::ptr::null());
		NvFlexMap(self.velocities, eNvFlexMapWait) as *mut Vector3
	}

	/// Gets a pointer to the particle buffer
	/// # Safety
	/// This function must be followed by a proper release, through self.unmap(), as this calls NvFlexMap
	pub unsafe fn get_phases(&self, solver: *mut NvFlexSolver) -> *mut i32 {
		NvFlexGetPhases(solver, self.velocities, std::ptr::null());
		NvFlexMap(self.velocities, eNvFlexMapWait) as *mut i32
	}

	/// # Safety
	/// This function must be followed by a proper release, through self.unmap(), as this calls NvFlexMap
	pub unsafe fn get(&self, solver: *mut NvFlexSolver) -> Option<Vec<Particle>> {
		let particles = self.get_particles(solver);
		let velocities = self.get_velocities(solver);
		let phases = self.get_phases(solver);

		let mut pvec = vec![];
		for i in 0 .. self.count as isize {
			let particle = particles.offset(i);
			if particle.is_null() {
				break;
			}

			let velocity = velocities.offset(i);
			let phase = phases.offset(i);

			pvec.push(Particle {
				pdata: particle.as_ref()?,
				velocity: velocity.as_ref()?,
				phase: phase.as_ref()?,
			});
		}

		Some(pvec)
	}

	pub fn flush(&mut self, solver: *mut NvFlexSolver) -> bool {
		if !self.has_changes {
			return false;
		}

		unsafe {
			NvFlexSetParticles(solver, self.buffer, std::ptr::null_mut());
			NvFlexSetVelocities(solver, self.velocities, std::ptr::null_mut());
			NvFlexSetPhases(solver, self.phases, std::ptr::null_mut());
			NvFlexSetActive(solver, self.active_indices, std::ptr::null_mut());
			NvFlexSetActiveCount(solver, self.count as i32); // All are active for now.
		}

		self.has_changes = false;

		true
	}

	/// Creates an environment to safely and efficiently create new particles.
	/// They will be properly mapped and unmapped, however, you still need to [flush] these changes.
	pub unsafe fn factory<F: Fn(&mut factory::ParticleFactory)>(&mut self, generator: F) {
		let particles = NvFlexMap(self.buffer, eNvFlexMapWait) as *mut Vector4;
		let velocities = NvFlexMap(self.velocities, eNvFlexMapWait) as *mut Vector3;
		let phases = NvFlexMap(self.phases, eNvFlexMapWait) as *mut i32;
		let active_indices = NvFlexMap(self.active_indices, eNvFlexMapWait) as *mut i32;

		let mut factory = factory::ParticleFactory::new(
			None,
			particles,
			velocities,
			phases,
			active_indices
		);

		generator(&mut factory);

		if factory.nparticles > 0 {
			self.has_changes = true;
			self.count += factory.nparticles as i32;
		}

		NvFlexUnmap(self.buffer);
		NvFlexUnmap(self.velocities);
		NvFlexUnmap(self.phases);
		NvFlexUnmap(self.active_indices);
	}
}

impl Drop for ParticleState {
	fn drop(&mut self) {
		unsafe {
			NvFlexFreeBuffer(self.buffer);
			NvFlexFreeBuffer(self.velocities);
			NvFlexFreeBuffer(self.phases);
			NvFlexFreeBuffer(self.active_indices);
		}
	}
}