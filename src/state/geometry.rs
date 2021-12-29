use nvflex_sys::*;
use std::mem::size_of;

use crate::{
	config,
	types::{Quat, Vector3, Vector4},
};

use super::FlexState;

#[derive(derivative::Derivative)]
#[derivative(Debug)]
pub struct GeometryState {
	#[derivative(Debug = "ignore")]
	shapes: Vec<NvFlexCollisionGeometry>,
	count: i32,
	has_changes: bool,

	pub buffer: *mut NvFlexBuffer,

	pub positions: *mut NvFlexBuffer, // Vec<Vec4>
	pub rotations: *mut NvFlexBuffer, // Vec<Quat>

	pub previous_positions: *mut NvFlexBuffer, // Vec<Vec4>
	pub previous_rotations: *mut NvFlexBuffer, // Vec<Quat>

	pub flags: *mut NvFlexBuffer, // Vec<i32>
}

impl Default for GeometryState {
	fn default() -> Self {
		Self {
			shapes: vec![],
			count: 0,
			has_changes: true,

			buffer: std::ptr::null_mut(),

			positions: std::ptr::null_mut(),
			rotations: std::ptr::null_mut(),

			previous_positions: std::ptr::null_mut(),
			previous_rotations: std::ptr::null_mut(),
			flags: std::ptr::null_mut(),
		}
	}
}

impl GeometryState {
	/// Allocates buffers used by the geometry state
	/// # Safety
	/// Do not call this function more than once
	pub unsafe fn alloc(&mut self, flex: *mut NvFlexLibrary) {
		self.buffer = NvFlexAllocBuffer(
			flex,
			config::MAX_SHAPES,
			size_of::<NvFlexCollisionGeometry>() as i32,
			eNvFlexBufferHost,
		);

		self.positions = NvFlexAllocBuffer(
			flex,
			config::MAX_SHAPES,
			size_of::<Vector4>() as i32,
			eNvFlexBufferHost,
		);

		self.rotations = NvFlexAllocBuffer(
			flex,
			config::MAX_SHAPES,
			size_of::<Quat>() as i32,
			eNvFlexBufferHost,
		);

		self.previous_positions = NvFlexAllocBuffer(
			flex,
			config::MAX_SHAPES,
			size_of::<Vector4>() as i32,
			eNvFlexBufferHost,
		);

		self.previous_rotations = NvFlexAllocBuffer(
			flex,
			config::MAX_SHAPES,
			size_of::<Quat>() as i32,
			eNvFlexBufferHost,
		);

		self.flags = NvFlexAllocBuffer(
			flex,
			config::MAX_SHAPES,
			size_of::<i32>() as i32,
			eNvFlexBufferHost,
		);
	}

	pub fn get_count(&self) -> i32 {
		self.count
	}

	pub fn add_shape(
		&mut self,
		shape: NvFlexCollisionGeometry,
		pos: Vector4,
		rot: Quat,
		flag: i32,
	) {
		self.shapes.push(shape);

		let count = self.count as isize;

		unsafe {
			let geometry = NvFlexMap(self.buffer, eNvFlexMapWait) as *mut NvFlexCollisionGeometry;
			let positions = NvFlexMap(self.positions, eNvFlexMapWait) as *mut Vector4;
			let rotations = NvFlexMap(self.rotations, eNvFlexMapWait) as *mut Quat;
			let previous_positions =
				NvFlexMap(self.previous_positions, eNvFlexMapWait) as *mut Vector4;
			let previous_rotations =
				NvFlexMap(self.previous_rotations, eNvFlexMapWait) as *mut Quat;
			let flags = NvFlexMap(self.flags, eNvFlexMapWait) as *mut i32;

			geometry.offset(count).write(shape);

			positions.offset(count).write(pos);

			rotations.offset(count).write(rot);

			previous_positions.offset(count).write(pos);

			previous_rotations.offset(count).write(rot);

			flags.offset(count).write(flag);

			self.unmap();
		}

		self.count += 1;
		self.has_changes = true;
	}

	pub fn unmap(&self) {
		unsafe {
			NvFlexUnmap(self.buffer);
			NvFlexUnmap(self.positions);
			NvFlexUnmap(self.rotations);
			NvFlexUnmap(self.previous_positions);
			NvFlexUnmap(self.previous_rotations);
			NvFlexUnmap(self.flags);
		}
	}

	/// Pushes shape changes to the FleX state
	/// # Safety
	/// Make sure that all of the buffers have been unmapped before calling this.
	pub unsafe fn flush(&mut self, solver: *mut NvFlexSolver) {
		if !self.has_changes {
			return;
		}

		NvFlexSetShapes(
			solver,
			self.buffer,
			self.positions,
			self.rotations,
			self.previous_positions,
			self.previous_rotations,
			self.flags,
			self.count,
		);

		self.has_changes = false;
	}
}

impl Drop for GeometryState {
	fn drop(&mut self) {
		unsafe {
			NvFlexFreeBuffer(self.buffer);
			NvFlexFreeBuffer(self.positions);
			NvFlexFreeBuffer(self.rotations);
			NvFlexFreeBuffer(self.previous_positions);
			NvFlexFreeBuffer(self.previous_rotations);
			NvFlexFreeBuffer(self.flags);
		}
	}
}
