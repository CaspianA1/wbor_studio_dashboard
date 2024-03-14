use std::borrow::Cow;

use sdl2::ttf::{FontStyle, Hinting};

use crate::{
	spinitron::{model::SpinitronModelName, state::SpinitronState},

	texture::{FontInfo, TextureCreationInfo, TextDisplayInfo, TexturePool},

	utility_types::{
		vec2f::Vec2f,
		generic_result::GenericResult,
		dynamic_optional::DynamicOptional,
		update_rate::{UpdateRate, UpdateRateCreator}
	},

	window_tree::{
		ColorSDL,
		Window,
		WindowContents,
		WindowUpdaterParams,
		PossibleSharedWindowStateUpdater
	},

	window_tree_defs::{
		shared_window_state::SharedWindowState,
		twilio::{make_twilio_window, TwilioState},
		clock::{ClockHandConfig, ClockHandConfigs, ClockHands},
		spinitron::{make_spinitron_windows, SpinitronModelWindowInfo, SpinitronModelWindowsInfo}
	}
};

////////// TODO: maybe split `make_wbor_dashboard` into some smaller sub-functions

/* TODO:
- Rename all `Possible` types to `Maybe`s (incl. the associated variable names) (and all `inner-prefixed` vars too)
- Run `clippy`
*/

////////// This function loads the set of API keys

fn load_api_keys_json() -> GenericResult<serde_json::Value> {
	const API_KEY_JSON_PATH: &str = "assets/api_keys.json";

	let api_keys_file = match std::fs::read_to_string(API_KEY_JSON_PATH) {
		Ok(contents) => Ok(contents),

		Err(err) => Err(format!(
			"The API key file at path '{API_KEY_JSON_PATH}' could not be found. Official error: '{err}'."
		))
	}?;

	Ok(serde_json::from_str(&api_keys_file)?)
}

//////////

// This returns a top-level window, shared window state, and a shared window state updater
pub fn make_wbor_dashboard(texture_pool: &mut TexturePool,
	sdl_window_size_in_pixels: (u32, u32),
	update_rate_creator: UpdateRateCreator)
	-> GenericResult<(Window, DynamicOptional, PossibleSharedWindowStateUpdater)> {

	////////// Defining some shared global variables

	// TODO: find a font that works with both emojis and normal text
	const FONT_INFO: FontInfo = FontInfo {
		path: "assets/fonts/Gohu/GohuFontuni14NerdFont-Regular.ttf",
		style: FontStyle::NORMAL,
		hinting: Hinting::Normal
	};

	let top_bar_window_size_y = 0.1;
	let main_windows_gap_size = 0.01;

	let theme_color_1 = ColorSDL::RGB(249, 236, 210);
	let shared_update_rate = update_rate_creator.new_instance(15.0);
	let api_keys_json = load_api_keys_json()?;

	let get_api_key = |name| -> GenericResult<&str> {
		api_keys_json[name].as_str().ok_or_else(|| format!("Could not find the API key with the name '{name}' in the API key JSON").into())
	};

	////////// Defining the Spinitron window extents

	// Note: `tl` = top left
	let spin_tl = Vec2f::new_scalar(main_windows_gap_size);
	let spin_size = Vec2f::new_scalar(0.55);
	let spin_text_height = 0.03;
	let spin_tr = spin_tl.x() + spin_size.x();

	let persona_tl = Vec2f::new(spin_tr + main_windows_gap_size, spin_tl.y());
	let persona_size = Vec2f::new_scalar(0.1);

	let show_tl = Vec2f::new(persona_tl.x() + persona_size.x() + main_windows_gap_size, spin_tl.y());
	let show_size = Vec2f::new_scalar(1.0 - show_tl.x() - main_windows_gap_size);

	let show_text_tl = Vec2f::translate(&(spin_tl + spin_size), 0.03, -0.2);
	let show_text_size = Vec2f::new(0.37, 0.05);

	let welcome_sign_size = Vec2f::new(persona_size.x(), persona_size.y() * 0.2);

	// TODO: make a type for the top-left/size combo (and add useful utility functions from there)

	//////////

	let all_model_windows_info = [
		SpinitronModelWindowsInfo {
			model_name: SpinitronModelName::Spin,
			text_color: theme_color_1,

			texture_window: Some(SpinitronModelWindowInfo {
				tl: spin_tl,
				size: spin_size,
				border_color: Some(theme_color_1)
			}),

			text_window: Some(SpinitronModelWindowInfo {
				tl: Vec2f::translate_y(&spin_tl, spin_size.y()),
				size: Vec2f::new(spin_size.x(), spin_text_height),
				border_color: Some(theme_color_1)
			})
		},

		SpinitronModelWindowsInfo {
			model_name: SpinitronModelName::Playlist,
			text_color: theme_color_1,
			texture_window: None,
			text_window: None
		},

		// Putting show before persona here so that the persona text is drawn over
		SpinitronModelWindowsInfo {
			model_name: SpinitronModelName::Show,
			text_color: theme_color_1,

			texture_window: Some(SpinitronModelWindowInfo {
				tl: show_tl,
				size: show_size,
				border_color: Some(theme_color_1)
			}),

			text_window: Some(SpinitronModelWindowInfo {
				tl: show_text_tl,
				size: show_text_size,
				border_color: Some(theme_color_1)
			})
		},

		SpinitronModelWindowsInfo {
			model_name: SpinitronModelName::Persona,
			text_color: theme_color_1,

			texture_window: Some(SpinitronModelWindowInfo {
				tl: persona_tl,
				size: persona_size,
				border_color: Some(theme_color_1)
			}),

			text_window: Some(SpinitronModelWindowInfo {
				tl: persona_tl,
				size: welcome_sign_size,
				border_color: Some(theme_color_1)
			})
		}
	];

	// The Spinitron windows update at the same rate as the shared update rate
	let mut all_main_windows = make_spinitron_windows(
		&all_model_windows_info, shared_update_rate
	);

	// TODO: make a temporary error window that pops up when needed (or log it somehow; but just handle it)

	////////// Making some static texture windows

	// TODO: make animated textures possible

	// Texture path, top left, size
	let static_texture_info = [
		("assets/wbor_logo.png", Vec2f::new(0.7, 0.75), Vec2f::new(0.1, 0.05)),
		("assets/wbor_soup.png", Vec2f::new(0.85, 0.72), Vec2f::new(0.06666666, 0.1))
	];

	all_main_windows.extend(static_texture_info.into_iter().map(|datum| {
		Window::new(
			None,
			DynamicOptional::NONE,

			WindowContents::Texture(texture_pool.make_texture(
				&TextureCreationInfo::Path(datum.0)
			).unwrap()),

			None,

			datum.1,
			datum.2,
			None
		)
	}));

	////////// Making a clock window

	let clock_size_x = top_bar_window_size_y;
	let clock_tl = Vec2f::new(1.0 - clock_size_x, 0.0);
	let clock_size = Vec2f::new(clock_size_x, 1.0);

	let (clock_hands, clock_window) = ClockHands::new_with_window(
		UpdateRate::ONCE_PER_FRAME,
		clock_tl,
		clock_size,

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

	/* TODO:
	- Actually implement this
	- Make the general structure of the text updater fns less repetitive
	*/

	fn weather_updater_fn((window, texture_pool, shared_state, area_drawn_to_screen): WindowUpdaterParams) -> GenericResult<()> {
		let weather_changed = true;
		let weather_string = "Rain (32f). So cold. ";
		let weather_text_color = ColorSDL::BLACK;

		let inner_shared_state: &SharedWindowState = shared_state.get_inner_value();

		let texture_creation_info = TextureCreationInfo::Text((
			inner_shared_state.font_info,

			TextDisplayInfo {
				text: Cow::Borrowed(weather_string),
				color: weather_text_color,

				scroll_fn: |secs_since_unix_epoch| {
					let repeat_rate_secs = 3.0;
					let base_scroll = (secs_since_unix_epoch % repeat_rate_secs) / repeat_rate_secs;
					(1.0 - base_scroll, true)
				},

				max_pixel_width: area_drawn_to_screen.width(),
				pixel_height: area_drawn_to_screen.height()
			}
		));

		window.update_texture_contents(
			weather_changed,
			texture_pool,
			&texture_creation_info,
			&inner_shared_state.fallback_texture_creation_info
		)?;

		Ok(())
	}

	let weather_update_rate = update_rate_creator.new_instance(60.0);

	let weather_window = Window::new(
		Some((weather_updater_fn, weather_update_rate)),
		DynamicOptional::NONE,
		WindowContents::Color(ColorSDL::RGB(255, 0, 255)),
		Some(ColorSDL::RED),
		Vec2f::ZERO,
		Vec2f::new(0.2, 0.2),
		None
	);

	////////// Making a Twilio window

	let twilio_state = TwilioState::new(
		get_api_key("twilio_account_sid")?,
		get_api_key("twilio_auth_token")?,
		6,
		chrono::Duration::hours(30)
	);

	let twilio_window = make_twilio_window(
		&twilio_state,
		shared_update_rate,

		Vec2f::new(0.58, 0.45), Vec2f::new(0.4, 0.27),

		0.025,
		WindowContents::Color(ColorSDL::RGB(0, 200, 0)),

		Vec2f::new(0.1, 0.45),
		theme_color_1, ColorSDL::RED,

		WindowContents::Texture(
			texture_pool.make_texture(&TextureCreationInfo::Path("assets/wbor_text_bubble.png"))?
		),
	);

	all_main_windows.push(twilio_window);

	////////// Making all of the main windows

	let main_window_tl_y = main_windows_gap_size + top_bar_window_size_y + main_windows_gap_size;
	let main_window_size_y = 1.0 - main_window_tl_y - main_windows_gap_size;
	let x_width_from_main_window_gap_size = 1.0 - main_windows_gap_size * 2.0;

	let top_bar_window = Window::new(
		None,
		DynamicOptional::NONE,
		WindowContents::Color(ColorSDL::RGB(128, 0, 32)),
		None,
		Vec2f::new_scalar(main_windows_gap_size),
		Vec2f::new(x_width_from_main_window_gap_size, top_bar_window_size_y),
		Some(vec![clock_window, weather_window])
	);

	let main_window = Window::new(
		None,
		DynamicOptional::NONE,

		WindowContents::Texture(texture_pool.make_texture(
			&TextureCreationInfo::Path("assets/wbor_dashboard_background.png")
		)?),

		Some(theme_color_1),
		Vec2f::new(main_windows_gap_size, main_window_tl_y),
		Vec2f::new(x_width_from_main_window_gap_size, main_window_size_y),
		Some(all_main_windows)
	);

	////////// Making the highest-level window (and accounting for window stretching)

	let size_pixels = (sdl_window_size_in_pixels.0 as f32, sdl_window_size_in_pixels.1 as f32);

	let (mut tl, mut size) = (Vec2f::ZERO, Vec2f::ONE);

	if size_pixels.0 < size_pixels.1 {
		size.set_y(size_pixels.0 / size_pixels.1);
		tl.set_y(tl.y() + (1.0 - size.y()) * 0.5);
	}
	else {
		size.set_x(size_pixels.1 / size_pixels.0);
		tl.set_x(tl.x() + (1.0 - size.x()) * 0.5);
	}

	let all_windows = Window::new(
		None,
		DynamicOptional::NONE,
		WindowContents::Color(ColorSDL::RGB(0, 128, 128)),
		None,
		tl,
		size,
		Some(vec![top_bar_window, main_window])
	);

	////////// Defining the shared state

	let boxed_shared_state = DynamicOptional::new(
		SharedWindowState {
			clock_hands,
			spinitron_state: SpinitronState::new(get_api_key("spinitron")?)?,
			twilio_state,
			font_info: &FONT_INFO,
			fallback_texture_creation_info: TextureCreationInfo::Path("assets/wbor_no_texture_available.png")
		}
	);

	fn shared_window_state_updater(state: &mut DynamicOptional, texture_pool: &mut TexturePool) -> GenericResult<()> {
		let state = state.get_inner_value_mut::<SharedWindowState>();
		state.spinitron_state.update()?;
		state.twilio_state.update(texture_pool)
	}

	//////////

	Ok((
		all_windows,
		boxed_shared_state,
		Some((shared_window_state_updater, shared_update_rate))
	))
}
