use sdl2::ttf::{FontStyle, Hinting};

use crate::{
	utility_types::{
		update_rate::UpdateRate,
		dynamic_optional::DynamicOptional,
		generic_result::GenericResult, vec2f::Vec2f
	},

	spinitron::state::SpinitronState,
	texture::{TexturePool, FontInfo, TextureCreationInfo},

	window_tree::{
		ColorSDL,
		Window,
		WindowContents,
		PossibleSharedWindowStateUpdater,
	},

	window_tree_defs::{
		shared_window_state::SharedWindowState,
		clock::{ClockHandConfig, ClockHandConfigs, ClockHands},
		twilio::make_twilio_window,
		spinitron::make_spinitron_windows
	}
};

////////// TODO: maybe split `make_wbor_dashboard` into some smaller sub-functions

/* TODO:
- Rename all `Possible` types to `Maybe`s (incl. the associated variable names) (and all `inner-prefixed` vars too)
- Run `clippy`
*/

////////// These are some API key utils

fn load_api_keys_json() -> GenericResult<serde_json::Value> {
	const API_KEY_JSON_PATH: &str = "assets/api_keys.json";

	let api_keys_file = match std::fs::read_to_string(API_KEY_JSON_PATH) {
		Ok(contents) => Ok(contents),

		Err(err) => Err(
			format!("The API key file at path '{}' could not be found. Official error: '{}'.",
			API_KEY_JSON_PATH, err)
		)
	}?;

	Ok(serde_json::from_str(&api_keys_file)?)
}

fn get_api_key<'a>(json: &'a serde_json::Value, name: &'a str) -> GenericResult<&'a str> {
	json[name].as_str().ok_or(format!("Could not find the API key with the name '{}' in the API key JSON", name).into())
}

//////////

// This returns a top-level window, shared window state, and a shared window state updater
pub fn make_wbor_dashboard(texture_pool: &mut TexturePool)
	-> GenericResult<(Window, DynamicOptional, PossibleSharedWindowStateUpdater)> {

	const FONT_INFO: FontInfo = FontInfo {
		path: "assets/fonts/Gohu/GohuFontuni14NerdFont-Regular.ttf",
		style: FontStyle::ITALIC,
		hinting: Hinting::Normal
	};

	////////// Loading in all the API keys

	let api_keys_json = load_api_keys_json()?;

	////////// Making the Spinitron windows

	let (individual_update_rate, shared_update_rate) = (
		UpdateRate::new(1.0),
		UpdateRate::new(1.0)
	);

	// This cannot exceed 0.5
	let model_window_size = Vec2f::new_from_one(0.4);

	let overspill_amount_to_right = -(model_window_size.x() * 2.0 - 1.0);
	let model_gap_size = overspill_amount_to_right / 3.0;

	let mut all_main_windows = make_spinitron_windows(
		model_window_size, model_gap_size, individual_update_rate
	);

	// TODO: make a temporary error window that pops up when needed

	////////// Making some static texture windows

	// TODO: make animated textures possible
	// TODO: remove a bunch of async TODOs, and just old ones in general

	let soup_height = model_gap_size * 1.5;

	// Updater, state, texture path, top left, size
	let static_texture_info = [
		(
			"assets/wbor_logo.png",
			Vec2f::ZERO,
			Vec2f::new(0.1, 0.05)
		),

		(
			"assets/wbor_soup.png",
			Vec2f::new(0.0, 1.0 - soup_height),
			Vec2f::new(model_gap_size, soup_height)
		)
	];

	all_main_windows.extend(static_texture_info.into_iter().map(|datum| {
		return Window::new(
			None,
			DynamicOptional::NONE,

			WindowContents::Texture(texture_pool.make_texture(
				&TextureCreationInfo::Path(datum.0),
			).unwrap()),

			None,

			datum.1,
			datum.2,
			None
		)
	}));

	////////// Making a clock window

	let (clock_hands, clock_window) = ClockHands::new_with_window(
		UpdateRate::ONCE_PER_FRAME,

		Vec2f::new(0.93, 0.0),
		Vec2f::new(0.07, 1.0),

		ClockHandConfigs {
			milliseconds: ClockHandConfig::new(0.01, 0.2, 0.5, ColorSDL::RGBA(255, 0, 0, 100)), // Milliseconds
			seconds: ClockHandConfig::new(0.01, 0.02, 0.48, ColorSDL::WHITE), // Seconds
			minutes: ClockHandConfig::new(0.01, 0.02, 0.35, ColorSDL::YELLOW), // Minutes
			hours: ClockHandConfig::new(0.01, 0.02, 0.2, ColorSDL::BLACK) // Hours
		},

		"assets/wbor_watch_dial.png",
		texture_pool
	)?;

	////////// Making a weather window

	// TODO: add a weather API to this
	let weather_window = Window::new(
		None,
		DynamicOptional::NONE,
		WindowContents::Color(ColorSDL::RGB(255, 0, 255)),
		None,
		Vec2f::ZERO,
		Vec2f::new(0.1, 1.0),
		None
	);

	////////// Making a twilio window

	let twilio_window = make_twilio_window(
		Vec2f::new(0.25, 0.0),
		Vec2f::new(0.5, 0.5),
		UpdateRate::new(1.0),
		ColorSDL::RGB(180, 180, 180),
		ColorSDL::RGB(20, 20, 20),
		get_api_key(&api_keys_json, "twilio_account_sid")?,
		get_api_key(&api_keys_json, "twilio_auth_token")?
	);

	////////// Making all of the main windows

	let small_edge_size = 0.015;

	let top_bar_window = Window::new(
		None,
		DynamicOptional::NONE,
		WindowContents::Color(ColorSDL::RGB(0, 0, 255)),
		None,
		Vec2f::new(small_edge_size, 0.01),
		Vec2f::new(1.0 - small_edge_size * 2.0, 0.06),
		Some(vec![clock_window, weather_window, twilio_window])
	);

	let main_window = Window::new(
		None,
		DynamicOptional::NONE,
		WindowContents::Color(ColorSDL::RGB(210, 180, 140)),
		None,
		Vec2f::new(small_edge_size, 0.08),
		Vec2f::new(1.0 - small_edge_size * 2.0, 0.9),
		Some(all_main_windows)
	);

	let all_windows = Window::new(
		None,
		DynamicOptional::NONE,
		WindowContents::Color(ColorSDL::RGB(0, 127, 0)),
		None,
		Vec2f::ZERO,
		Vec2f::ONE,
		Some(vec![top_bar_window, main_window])
	);

	////////// Defining the shared state

	let boxed_shared_state = DynamicOptional::new(
		SharedWindowState {
			clock_hands,
			spinitron_state: SpinitronState::new(get_api_key(&api_keys_json, "spinitron")?)?,
			font_info: FONT_INFO,
			fallback_texture_creation_info: TextureCreationInfo::Path("assets/wbor_no_texture_available.png"),
		}
	);

	fn shared_window_state_updater(state: &mut DynamicOptional) -> GenericResult<()> {
		let state: &mut SharedWindowState = state.get_inner_value_mut();
		state.spinitron_state.update()
	}

	//////////

	// TODO: past a certain point, make sure that the texture pool never grows

	Ok((
		all_windows,
		boxed_shared_state,
		Some((shared_window_state_updater, shared_update_rate))
	))
}
