use std::mem::MaybeUninit;
use std::time::Instant;

// State holding all of the data for FleX.
use crate::{
	config,
	helper::*,
	types::{Particle, Quat, Vector3, Vector4},
};
use nvflex_sys::*;

mod geometry;
use geometry::GeometryState;

#[derive(Debug)]
pub struct FlexState<'a> {
	/* Shared */
	initialized: bool,
	instant: Instant,
	lib: *mut NvFlexLibrary,

	/* Particles */
	nactive: i32,
	pub particles: Vec<Particle<'a>>,

	/// Note this will most likely be null.
	desc: *mut NvFlexInitDesc,
	/// Note this will also most likely be null.
	solver_desc: MaybeUninit<NvFlexSolverDesc>,

	pub solver: *mut NvFlexSolver,
	pub particle_buffer: *mut NvFlexBuffer,
	pub velocity_buffer: *mut NvFlexBuffer,
	pub phase_buffer: *mut NvFlexBuffer,
	pub active_buffer: *mut NvFlexBuffer,

	/* Geometry */
	pub geometry: GeometryState,
}

impl<'a> Default for FlexState<'a> {
	fn default() -> Self {
		Self {
			/* Shared */
			initialized: false,
			instant: Instant::now(),
			lib: std::ptr::null_mut(),

			/* Particles */
			nactive: config::MAX_PARTICLES,
			particles: vec![],

			desc: std::ptr::null_mut(),
			solver_desc: MaybeUninit::uninit(),

			solver: std::ptr::null_mut(),

			particle_buffer: std::ptr::null_mut(),
			velocity_buffer: std::ptr::null_mut(),
			phase_buffer: std::ptr::null_mut(),
			active_buffer: std::ptr::null_mut(),

			/* Geometry */
			geometry: GeometryState::default(),
		}
	}
}

#[derive(Debug, thiserror::Error)]
pub enum InitError {
	#[error("Failed to create Flex Library")]
	NvFlexInit,
}

#[derive(Debug, thiserror::Error)]
pub enum ShutdownError {}

impl<'a> FlexState<'a> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn init(&mut self) -> Result<&mut Self, InitError> {
		unsafe {
			use std::mem::size_of;

			let flex = NvFlexInit(NV_FLEX_VERSION as i32, None, std::ptr::null_mut());

			if flex.is_null() {
				return Err(InitError::NvFlexInit);
			}

			NvFlexSetSolverDescDefaults(self.solver_desc.as_mut_ptr());

			self.solver = NvFlexCreateSolver(flex, self.solver_desc.as_ptr());

			// TODO: Move this into another part that flushes and allocates like how GeometryState does right now
			// ParticleState?

			self.particle_buffer = NvFlexAllocBuffer(
				flex,
				self.nactive,
				size_of::<Vector4>() as i32,
				eNvFlexBufferHost,
			);
			self.velocity_buffer = NvFlexAllocBuffer(
				flex,
				self.nactive,
				size_of::<Vector3>() as i32,
				eNvFlexBufferHost,
			);
			self.phase_buffer = NvFlexAllocBuffer(
				flex,
				self.nactive,
				size_of::<i32>() as i32,
				eNvFlexBufferHost,
			);
			self.active_buffer = NvFlexAllocBuffer(
				flex,
				self.nactive,
				size_of::<i32>() as i32,
				eNvFlexBufferHost,
			);

			self.geometry = GeometryState::default();
			self.geometry.alloc(flex);

			let particles = NvFlexMap(self.particle_buffer, eNvFlexMapWait) as *mut Vector4;
			let velocities = NvFlexMap(self.velocity_buffer, eNvFlexMapWait) as *mut Vector3;
			let phases = NvFlexMap(self.phase_buffer, eNvFlexMapWait) as *mut i32;
			let active = NvFlexMap(self.active_buffer, eNvFlexMapWait) as *mut i32;

			let baux = NvFlexCollisionGeometry {
				box_: NvFlexBoxGeometry {
					halfExtents: [50000.0, 50000.0, 5.0],
				},
			};

			let flag = NvFlexMakeShapeFlags(eNvFlexShapeBox, false);
			self.geometry.add_shape(
				baux,
				Vector4(0.0, 0.0, 0.0, 0.0),
				Quat(0.0, 0.0, 0.0, 0.0),
				flag,
			);

			self.geometry.add_shape(
				baux,
				Vector4(0.0, 0.0, 0.0, 0.0),
				Quat(0.0, 1.0, 0.0, 0.0),
				flag,
			);

			self.geometry.add_shape(
				baux,
				Vector4(0.0, 0.0, 0.0, 0.0),
				Quat(1.0, 0.0, 0.0, 0.0),
				flag,
			);

			let fluid = NvFlexMakePhase(0, eNvFlexPhaseSelfCollide | eNvFlexPhaseFluid);

			for i in 0..self.nactive {
				let ind = i as isize;
				particles
					.offset(ind)
					.write(Vector4(50.0 * i as f32, 0.0, 5000.0, 2.0));

				velocities.offset(ind).write(Vector3(0.0, 0.0, -5.0));
				phases.offset(ind).write(fluid);
				active.offset(ind).write(i);
			}

			// Unmap to transfer data to GPU
			self.unmap();

			// Transfer data
			NvFlexSetParams(self.solver, &config::PARAMS);
			NvFlexSetParticles(self.solver, self.particle_buffer, std::ptr::null_mut());
			NvFlexSetVelocities(self.solver, self.velocity_buffer, std::ptr::null_mut());
			NvFlexSetPhases(self.solver, self.phase_buffer, std::ptr::null_mut());
			NvFlexSetActive(self.solver, self.active_buffer, std::ptr::null_mut());
			NvFlexSetActiveCount(self.solver, self.nactive);

			// This will call SetShapes
			self.geometry.flush(self.solver);

			self.lib = flex;
		}

		self.initialized = true;

		Ok(self)
	}

	pub fn unmap(&self) {
		unsafe {
			NvFlexUnmap(self.velocity_buffer);
			NvFlexUnmap(self.particle_buffer);
			NvFlexUnmap(self.phase_buffer);
			NvFlexUnmap(self.active_buffer);
		}
	}

	pub fn tick(&mut self) {
		unsafe {
			let dt = self.instant.elapsed();
			self.instant = Instant::now();

			NvFlexUpdateSolver(self.solver, dt.as_secs_f32(), 1, false);
		}
	}

	/// Gets a pointer to the particle buffer
	/// # Safety
	/// This function must be followed by a proper release, through self.unmap(), as this calls NvFlexMap
	pub unsafe fn get_particles(&self) -> *mut Vector4 {
		NvFlexGetParticles(self.solver, self.particle_buffer, std::ptr::null());

		NvFlexMap(self.particle_buffer, eNvFlexMapWait) as *mut Vector4
	}

	/// # Safety
	/// This function must be followed by a proper release, through self.unmap(), as this calls NvFlexMap
	pub unsafe fn get_velocities(&self) -> *mut Vector3 {
		NvFlexGetVelocities(self.solver, self.velocity_buffer, std::ptr::null());

		NvFlexMap(self.velocity_buffer, eNvFlexMapWait) as *mut Vector3
	}

	/// # Safety
	/// This function must be followed by a proper release, through self.unmap(), as this calls NvFlexMap
	pub unsafe fn get_phases(&self) -> *mut i32 {
		NvFlexGetPhases(self.solver, self.phase_buffer, std::ptr::null());

		NvFlexMap(self.phase_buffer, eNvFlexMapWait) as *mut i32
	}

	/// # Safety
	/// This function must be followed by a proper release, through self.unmap(), as this calls NvFlexMap
	pub unsafe fn get(&self) -> Option<Vec<Particle>> {
		let particles = self.get_particles();
		let velocities = self.get_velocities();
		let phases = self.get_phases();

		let mut pvec = vec![];
		for i in 0..self.nactive as isize {
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
}

impl<'a> Drop for FlexState<'a> {
	/// Consumes the FlexState, properly releasing allocated resources.
	fn drop(&mut self) {
		unsafe {
			NvFlexFreeBuffer(self.particle_buffer);
			NvFlexFreeBuffer(self.phase_buffer);
			NvFlexFreeBuffer(self.velocity_buffer);
			NvFlexFreeBuffer(self.active_buffer);
			NvFlexDestroySolver(self.solver);
			NvFlexShutdown(self.lib);
		}
	}
}
