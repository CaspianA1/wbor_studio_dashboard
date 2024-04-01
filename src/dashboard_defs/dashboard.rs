use std::borrow::Cow;
use sdl2::ttf::{FontStyle, Hinting};

use crate::{
	texture::{FontInfo, TextureCreationInfo, TexturePool},
	spinitron::{model::SpinitronModelName, state::SpinitronState},

	utility_types::{
		json_utils,
		vec2f::Vec2f,
		dynamic_optional::DynamicOptional,
		generic_result::{GenericResult, MaybeError},
		update_rate::{UpdateRate, UpdateRateCreator}
	},

	window_tree::{
		ColorSDL,
		Window,
		WindowContents,
		PossibleSharedWindowStateUpdater
	},

	dashboard_defs::{
		error::make_error_window,
		credit::make_credit_window,
		weather::make_weather_window,
		shared_window_state::SharedWindowState,
		twilio::{make_twilio_window, TwilioState},
		clock::{ClockHandConfig, ClockHandConfigs, ClockHands},
		spinitron::{make_spinitron_windows, SpinitronModelWindowInfo, SpinitronModelWindowsInfo}
	}
};

////////// TODO: maybe split `make_dashboard` into some smaller sub-functions

/* TODO:
- Rename all `Possible` types to `Maybe`s (incl. the associated variable names) (and all `inner-prefixed` vars too)
- Make plain texture creation less verbose through a wrapper function
*/

#[derive(serde::Deserialize)]
struct ApiKeys {
	spinitron: String,
	openweathermap: String,
	twilio_account_sid: String,
	twilio_auth_token: String
}

//////////

// This returns a top-level window, shared window state, and a shared window state updater
pub fn make_dashboard(
	texture_pool: &mut TexturePool,
	update_rate_creator: UpdateRateCreator)
	-> GenericResult<(Window, DynamicOptional, PossibleSharedWindowStateUpdater)> {

	////////// Defining some shared global variables

	// TODO: find a font that works with both emojis and normal text
	const FONT_INFO: FontInfo = FontInfo {
		path: "assets/fonts/Gohu/GohuFontuni14NerdFont-Regular.ttf",
		style: FontStyle::NORMAL,
		hinting: Hinting::Normal,
		maybe_outline_width: None
	};

	let top_bar_window_size_y = 0.1;
	let main_windows_gap_size = 0.01;

	let theme_color_1 = ColorSDL::RGB(249, 236, 210);
	let shared_update_rate = update_rate_creator.new_instance(15.0);
	let api_keys: ApiKeys = json_utils::load_from_file("assets/api_keys.json")?;

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
				&TextureCreationInfo::Path(Cow::Borrowed(datum.0))
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

	let weather_window = make_weather_window(
		Vec2f::ZERO,
		Vec2f::new(0.4, 0.3),
		update_rate_creator,
		&api_keys.openweathermap,
		"Brunswick",
		"ME",
		"US"
	);

	////////// Making a Twilio window

	let twilio_state = TwilioState::new(
		&api_keys.twilio_account_sid,
		&api_keys.twilio_auth_token,
		6,
		chrono::Duration::try_days(5).ok_or("Could not build message history duration!")?,
		false
	);

	let twilio_window = make_twilio_window(
		&twilio_state,

		// This is how often the history windows check for new messages (this is low so that it'll be fast in the beginning)
		update_rate_creator.new_instance(0.25),

		Vec2f::new(0.58, 0.45), Vec2f::new(0.4, 0.27),

		0.025,
		WindowContents::Color(ColorSDL::RGB(0, 200, 0)),

		Vec2f::new(0.1, 0.45),
		theme_color_1, theme_color_1,

		WindowContents::Texture(
			texture_pool.make_texture(&TextureCreationInfo::Path(Cow::Borrowed("assets/wbor_text_bubble.png")))?
		),
	);

	all_main_windows.push(twilio_window);

	////////// Making an error window

	let error_window = make_error_window(
		Vec2f::new(0.015, 0.938),
		Vec2f::new(0.465, 0.05),
		update_rate_creator.new_instance(2.0),
		WindowContents::Color(ColorSDL::RGBA(255, 0, 0, 160)),
		ColorSDL::BLUE
	);

	all_main_windows.push(error_window);

	////////// Making a credit windoww

	let credit_window = make_credit_window(
		Vec2f::new(0.85, 0.97),
		Vec2f::new(0.15, 0.03),
		ColorSDL::RGB(210, 180, 140),
		"By Caspian Ahlberg"
	);

	all_main_windows.push(credit_window);

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
			&TextureCreationInfo::Path(Cow::Borrowed("assets/wbor_dashboard_background.png"))
		)?),

		Some(theme_color_1),
		Vec2f::new(main_windows_gap_size, main_window_tl_y),
		Vec2f::new(x_width_from_main_window_gap_size, main_window_size_y),
		Some(all_main_windows)
	);

	////////// Making the highest-level window

	let all_windows = Window::new(
		None,
		DynamicOptional::NONE,
		WindowContents::Color(ColorSDL::RGB(0, 128, 128)),
		None,
		Vec2f::ZERO,
		Vec2f::ONE,
		Some(vec![top_bar_window, main_window])
	);

	////////// Defining the shared state

	let boxed_shared_state = DynamicOptional::new(
		SharedWindowState {
			clock_hands,
			spinitron_state: SpinitronState::new(&api_keys.spinitron)?,
			twilio_state,
			font_info: &FONT_INFO,
			fallback_texture_creation_info: TextureCreationInfo::Path(Cow::Borrowed("assets/wbor_no_texture_available.png")),
			curr_dashboard_error: None
		}
	);

	fn shared_window_state_updater(state: &mut DynamicOptional, texture_pool: &mut TexturePool) -> MaybeError {
		let state = state.get_mut::<SharedWindowState>();

		let mut error = None;

		// More continual updaters can be added here
		let success_states_and_names = [
			(state.spinitron_state.update()?, "Spinitron"),
			(state.twilio_state.update(texture_pool)?, "Twilio (messaging)")
		];

		for (succeeded, name) in success_states_and_names {
			if !succeeded {
				if let Some(already_error) = &mut error {
					*already_error += ", and ";
					*already_error += name;
				}
				else {
					error = Some(format!("Internal dashboard error from {name}"))
				}
			}
		}

		if let Some(inner_error) = &mut error {
			*inner_error += "! ";
		}

		state.curr_dashboard_error = error;

		Ok(())
	}

	//////////

	Ok((
		all_windows,
		boxed_shared_state,
		Some((shared_window_state_updater, shared_update_rate))
	))
}
