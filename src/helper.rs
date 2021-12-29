use nvflex_sys::*;

#[inline(always)]
#[allow(non_snake_case)]
pub fn NvFlexMakePhase(group: i32, particle_flags: i32) -> i32 {
	(group & eNvFlexPhaseGroupMask)
		| (particle_flags & eNvFlexPhaseFlagsMask)
		| eNvFlexPhaseShapeChannelMask
}

#[inline(always)]
#[allow(non_snake_case)]
pub fn NvFlexMakeShapeFlags(ty: NvFlexCollisionShapeType, dynamic: bool) -> i32 {
	ty | (if dynamic { eNvFlexShapeFlagDynamic } else { 0 }) | eNvFlexPhaseShapeChannelMask
}
