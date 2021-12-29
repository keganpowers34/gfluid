use nvflex_sys::*;

pub const MAX_PARTICLES: i32 = 10;
pub const MAX_SHAPES: i32 = 50;

pub const PARAMS: NvFlexParams = NvFlexParams {
	numIterations: 3,
	gravity: [0.0, 0.0, -9.8],
	radius: 0.15,
	solidRestDistance: 0.0,
	fluidRestDistance: 0.0,
	dynamicFriction: 0.0,
	staticFriction: 0.0,
	particleFriction: 0.0,
	restitution: 0.0,
	adhesion: 0.0,
	sleepThreshold: 0.0,
	maxSpeed: f32::MAX,
	maxAcceleration: 100.0, // 10x gravity
	shockPropagation: 0.0,
	dissipation: 0.0,
	damping: 0.0,
	wind: [0.0, 0.0, 0.0],
	drag: 0.0,
	lift: 0.0,
	cohesion: 0.025,
	surfaceTension: 0.0,
	viscosity: 0.0,
	vorticityConfinement: 40.0,
	anisotropyScale: 20.0,
	anisotropyMin: 0.1,
	anisotropyMax: 2.0,
	smoothing: 1.0,
	solidPressure: 1.0,
	freeSurfaceDrag: 0.0,
	buoyancy: 1.0,
	diffuseThreshold: f32::MAX,
	diffuseBuoyancy: 1.0,
	diffuseDrag: 0.8,
	diffuseBallistic: 16,
	diffuseLifetime: 2.0,
	collisionDistance: 0.025,
	particleCollisionMargin: 0.01,
	shapeCollisionMargin: 0.01,
	planes: [
		[0.0, 1.0, 0.0, 0.0], // Floor, also maybe 0, 1, 0, 2
		[0.0, 0.0, 0.0, 0.0],
		[0.0, 0.0, 0.0, 0.0],
		[0.0, 0.0, 0.0, 0.0],
		[0.0, 0.0, 0.0, 0.0],
		[0.0, 0.0, 0.0, 0.0],
		[0.0, 0.0, 0.0, 0.0],
		[0.0, 0.0, 0.0, 0.0],
	],
	numPlanes: 1,
	relaxationMode: nvflex_sys::eNvFlexRelaxationLocal,
	relaxationFactor: 1.0,
};

const MAX_PLANES: i32 = 12;
