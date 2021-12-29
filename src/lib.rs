#![allow(unused)]

use rglua::prelude::*;
use std::sync::atomic::{AtomicPtr, Ordering};

mod config;
mod helper;
mod state;
mod types;

use state::FlexState;

#[derive(Debug, thiserror::Error)]
enum OpenError {
	#[error("Failure when prepping NVFlex {0}")]
	FlexInit(#[from] state::InitError),
}

static STATE: AtomicPtr<FlexState> = AtomicPtr::new(std::ptr::null_mut());

#[lua_function]
fn get(l: LuaState) -> i32 {
	let state = STATE.load(Ordering::Relaxed);

	if let Some(state) = unsafe { state.as_ref() } {
		match unsafe { state.get() } {
			Some(data) => printgm!(l, "{:#?}", data),
			None => printgm!(l, "Couldn't get data."),
		}
		state.unmap();
	}

	0
}

#[lua_function]
pub fn get_particles(l: LuaState) -> i32 {
	let state = STATE.load(Ordering::Relaxed);

	match unsafe { state.as_ref() } {
		Some(state) => {
			if let Some(data) = unsafe { state.get() } {
				lua_createtable(l, data.len() as i32, 0);
				for (i, particle) in data.iter().enumerate() {
					lua_createtable(l, 0, 4); // -3 particle = {}

					lua_pushstring(l, cstr!("phase")); // -2
					lua_pushnumber(l, *particle.phase as f64); // -1
					lua_rawset(l, -3);

					lua_pushstring(l, cstr!("imass")); // -2
					lua_pushnumber(l, particle.pdata.3 as f64); // -1
					lua_rawset(l, -3);

					lua_pushstring(l, cstr!("velocity"));
					lua_createtable(l, 3, 0); // velocity. -3

					lua_pushnumber(l, particle.velocity.0 as f64); // -2
					lua_rawseti(l, -2, 1); // velocity[1] = particle.velocity.0

					lua_pushnumber(l, particle.velocity.1 as f64); // -2
					lua_rawseti(l, -2, 2); // velocity[2] = particle.velocity.1

					lua_pushnumber(l, particle.velocity.2 as f64); // -2
					lua_rawseti(l, -2, 3); // velocity[3] = particle.velocity.2

					lua_rawset(l, -3); // t.velocity = stack[ #stack - 1 ]

					lua_pushstring(l, cstr!("position"));
					lua_createtable(l, 3, 0); // position. -3

					lua_pushnumber(l, particle.pdata.0 as f64); // -2
					lua_rawseti(l, -2, 1); // position[1] = particle.pdata.0

					lua_pushnumber(l, particle.pdata.1 as f64); // -2
					lua_rawseti(l, -2, 2); // position[2] = particle.pdata.1

					lua_pushnumber(l, particle.pdata.2 as f64); // -2
					lua_rawseti(l, -2, 3); // position[3] = particle.pdata.2

					lua_rawset(l, -3); // t.position = stack[ #stack - 1 ]

					lua_rawseti(l, -2, i as i32 + 1); // particles[i + 1] = stack[#stack] (aka particle)
				}
				return 1;
			}
			state.unmap();
			0
		}
		None => 0,
	}
}

#[lua_function]
fn tick(_l: LuaState) -> i32 {
	let state = STATE.load(Ordering::Relaxed);

	if let Some(a) = unsafe { state.as_mut() } {
		a.tick();
	};

	0
}

fn open(l: LuaState) -> Result<(), OpenError> {
	let mut flex_state = Box::new(FlexState::new());
	flex_state.init()?;

	let flex_ptr = Box::into_raw(flex_state);
	STATE.store(flex_ptr, Ordering::Relaxed);

	let r = reg! [
		"get_state" => get_state,
		"get_particles" => get_particles,
		"get" => get
	];

	lua_getglobal(l, cstr!("hook"));
	lua_getfield(l, -1, cstr!("Add"));

	lua_remove(l, -2); // Remove 'hook' table from stack

	// Push arguments to hook.Add
	lua_pushstring(l, cstr!("Tick"));
	lua_pushstring(l, cstr!("GFluid_Tick"));
	lua_pushcfunction(l, tick);

	// Call hook.Add
	lua_call(l, 3, 0);

	luaL_register(l, cstr!("flex"), r.as_ptr());

	Ok(())
}

// Returns ref id
#[lua_function]
fn get_state(l: LuaState) -> i32 {
	let a = STATE.load(Ordering::SeqCst);
	printgm!(l, "{:?}", unsafe { a.as_ref() });
	0
}

#[gmod_open]
fn main(l: LuaState) -> i32 {
	match open(l) {
		Err(why) => {
			luaL_error(l, cstr!("Failed to initialize NVFlex: %s"), why);
		}
		Ok(_) => {
			printgm!(l, "Started nvflex!");
		}
	}

	0
}

#[gmod_close]
fn close(_l: LuaState) -> i32 {
	let ptr = STATE.load(Ordering::SeqCst);
	// Get box out of pointer to interact with flex state
	// This will be dropped automagically
	let _flex_state = unsafe { Box::<FlexState>::from_raw(ptr as *mut FlexState) };

	// Flex state will now run ``drop`` here since it is owned by the Box

	0
}
