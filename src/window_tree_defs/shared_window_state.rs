use crate::{
    window_tree_defs::clock::ClockHands,
    texture::TextureCreationInfo,
    spinitron::state::SpinitronState,
};

pub struct SharedWindowState<'a> {
	pub clock_hands: ClockHands,
	pub spinitron_state: SpinitronState,

	// This is used whenever a texture can't be loaded
	pub fallback_texture_creation_info: TextureCreationInfo<'a>
}