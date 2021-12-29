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

mod particle;
use particle::ParticleState;

#[derive(Debug)]
pub struct FlexState {
	/* Shared */
	initialized: bool,
	instant: Instant,
	lib: *mut NvFlexLibrary,

	/// Note this will most likely be null.
	desc: *mut NvFlexInitDesc,

	solver_desc: MaybeUninit<NvFlexSolverDesc>,
	pub solver: *mut NvFlexSolver,

	pub particles: ParticleState,
	pub geometry: GeometryState,
}

impl Default for FlexState {
	fn default() -> Self {
		Self {
			initialized: false,
			instant: Instant::now(),
			lib: std::ptr::null_mut(),

			desc: std::ptr::null_mut(),

			solver_desc: MaybeUninit::uninit(),
			solver: std::ptr::null_mut(),

			/* Separate States */

			particles: ParticleState::default(),
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

impl FlexState {
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

			self.particles = ParticleState::default();
			self.particles.alloc(flex);

			self.geometry = GeometryState::default();
			self.geometry.alloc(flex);

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

			self.particles.factory(|mut x| {
				for i in 0 .. config::MAX_PARTICLES {
					x.create(
						Vector4(50.0 * i as f32, 0.0, 5000.0, 2.0),
						Vector3(0.0, 0.0, -5.0),
						fluid,
						true
					);
				}
			});

			// Unmap here

			// Transfer data
			NvFlexSetParams(self.solver, &config::PARAMS);

			// This will call all of the NvFlexSet* functions
			self.particles.flush(self.solver);
			self.geometry.flush(self.solver);

			self.lib = flex;
		}

		self.initialized = true;

		Ok(self)
	}

	pub fn tick(&mut self) {
		unsafe {
			let dt = self.instant.elapsed();
			self.instant = Instant::now();

			NvFlexUpdateSolver(self.solver, dt.as_secs_f32(), 1, false);
		}
	}

	pub unsafe fn get(&self) -> Option<Vec<Particle>> {
		self.particles.get(self.solver)
	}

	pub fn unmap(&self) {
		unsafe {
			self.particles.unmap();
			self.geometry.unmap();
		}
	}
}

impl Drop for FlexState {
	/// Consumes the FlexState, properly releasing allocated resources.
	fn drop(&mut self) {
		unsafe {
			NvFlexDestroySolver(self.solver);
			NvFlexShutdown(self.lib);
		}
	}
}
