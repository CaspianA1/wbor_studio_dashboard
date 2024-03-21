/* TODO:
- Actually implement this
- Make the general structure of the text updater fns less repetitive
*/

use std::borrow::Cow;

use crate::{
	// request,

	texture::{TextDisplayInfo, TextureCreationInfo},

	utility_types::{
        vec2f::Vec2f,
        update_rate::UpdateRateCreator,
        generic_result::GenericResult,
		dynamic_optional::DynamicOptional
	},

	window_tree::{
		ColorSDL,
        Window,
        WindowContents,
        WindowUpdaterParams
	},

	window_tree_defs::shared_window_state::SharedWindowState
};

// TODO: fill this with stuff
struct WeatherWindowState {
	api_key: String,
	location: String
}

pub fn weather_updater_fn((window, texture_pool, shared_state, area_drawn_to_screen): WindowUpdaterParams) -> GenericResult<()> {
	let weather_changed = true;
	let weather_string = "Rain (32f). So cold. ";
	let weather_text_color = ColorSDL::BLACK;

	/*
	- 1000 API calls free every day
	- That's 1000 per 24 hrs
	- Our 41.666 per hour, or around once per 1.444 minutes
	- To make stuff easy, do once every 2 minutes
	- TODO: do it once every 10 minutes (that's how frequently the data updates: https://openweathermap.org/appid)
	*/

	// let individual_window_state = window.get_state::<WeatherWindowState>();
	let inner_shared_state = shared_state.get_inner_value::<SharedWindowState>();

	// TODO: perhaps don't build request urls, just request objects directly
	/*
	let url = request::build_url("https://api.openweathermap.org/data/2.5/weather",
		&[],

		&[
			("q", Cow::Borrowed(&individual_window_state.location)),
			("appid", Cow::Borrowed(&individual_window_state.api_key)),
			("units", Cow::Borrowed("metric"))
		]
	)?;
	*/

	//////////

	// TODO: why are all the damn fields optional?

	/*
	#[derive(serde::Deserialize, Debug)] // TODO: remove `Debug`
	struct WeatherDesc1 {
		feels_like: f32,
		temp: f32,
		pressure: i32,
		humidity: i32,
		temp_min: f32,
		temp_max: f32
	}

	#[derive(serde::Deserialize, Debug)] // TODO: remove `Debug`
	struct WeatherDesc2 {
		description: String,
		icon: String,
		id: i32,
		main: String,
		// visibility: i32
	}
	// TODO: vary the wind things returned (may sometimes be rain)
	#[derive(serde::Deserialize, Debug)] // TODO: remove `Debug`
	struct WindDesc {
		deg: Option<i32>,
		gust: Option<f32>,
		speed: Option<f32>
	}

	#[derive(serde::Deserialize, Debug)] // TODO: remove `Debug`
	struct CloudsDesc {
		all: i32
	}

	#[derive(serde::Deserialize, Debug)] // TODO: remove `Debug`
	struct RainDesc {
		// all: i32

		#[serde(rename = "1h")]
		one_hour: f32
	}

	#[derive(serde::Deserialize, Debug)] // TODO: remove `Debug`
	struct SnowDesc {
		all: i32
	}

	#[derive(serde::Deserialize, Debug)] // TODO: remove `Debug`
	struct WeatherInfo {
		main: WeatherDesc1,
		weather: [WeatherDesc2; 1],

		wind: Option<WindDesc>,
		clouds: Option<CloudsDesc>,
		rain: Option<RainDesc>,
		snow: Option<SnowDesc>
	}
	*/

	//////////

	/*
	let json = request::as_json(request::get(&url))?;
	let w: WeatherInfo = serde_json::from_value(json)?;
	*/

	/*
	Deciding what data to show (I don't want to go overboard):
	1. (MOST IMPORTANT) An emoji for the given icon (I have this data)
	2. (SECOND-MOST IMPORTANT) What temperature it feels like
	3. (MID-LATER) If there's high pressure or humidity, say "It's a scorcher!" Or "It's a hot one today!".
	4. (LATER) If it's windy, show the wind gust and speed (same for rain, snow, etc.)
	*/

	let texture_creation_info = TextureCreationInfo::Text((
		inner_shared_state.font_info,

		TextDisplayInfo {
			text: Cow::Borrowed(weather_string),
			color: weather_text_color,

			scroll_fn: |seed, _| {
				let repeat_rate_secs = 3.0;
				let base_scroll = (seed % repeat_rate_secs) / repeat_rate_secs;
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
	)
}

// Note: the state code can be empty here!
pub fn make_weather_window(update_rate_creator: &UpdateRateCreator, api_key: &str,
	city_name: &str, state_code: &str, country_code: &str) -> Window {

	const UPDATE_RATE_SECS: f32 = 60.0 * 10.0; // Once every 10 minutes (this is how frequent the weather data is)

	let weather_update_rate = update_rate_creator.new_instance(UPDATE_RATE_SECS);
	let location = [city_name, state_code, country_code].join(",");

	Window::new(
		Some((weather_updater_fn, weather_update_rate)),
		DynamicOptional::new(WeatherWindowState {api_key: api_key.to_string(), location}),
		WindowContents::Color(ColorSDL::RGB(255, 0, 255)),
		Some(ColorSDL::RED),
		Vec2f::ZERO,
		Vec2f::new(0.2, 0.2),
		None
	)
}