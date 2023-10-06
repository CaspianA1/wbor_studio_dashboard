use sdl2;

pub mod window_hierarchy;

use window_hierarchy::{
	ColorSDL, Vec2f, WindowContents,
	HierarchalWindow, render_windows_recursively
};

pub mod spinitron;

// Working from this: https://blog.logrocket.com/using-sdl2-bindings-rust/

struct AppConfig<'a> {
	name: &'a str,
	width: u32,
	height: u32,
	fps: u32,
	bg_color: ColorSDL
}

// TODO: maybe give a retro theme to everything

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
	let config = AppConfig {
		name: "Recursive Box Demo",
		width: 800,
		height: 600,
		fps: 60,
		bg_color: ColorSDL::RGB(50, 50, 50)
	};

	let sdl_context = sdl2::init()?;
	let sdl_video_subsystem = sdl_context.video()?;

	let mut event_pump = sdl_context.event_pump()?;
	let sdl_window_bounds = sdl2::rect::Rect::new(0, 0, config.width, config.height);

	let sdl_window = sdl_video_subsystem
		.window(config.name, config.width, config.height)
		.position_centered().opengl().build()
		.map_err(|e| e.to_string())?;

	let mut sdl_canvas = sdl_window.into_canvas().build().map_err(|e| e.to_string())?;
	let texture_creator = sdl_canvas.texture_creator();

	let sleep_time = std::time::Duration::new(0, 1_000_000_000u32 / config.fps);

	////////// Getting the current spins and album texture, as a test

	let untrimmed_api_key = std::fs::read_to_string("assets/spinitron_api_key.txt").unwrap();
	let api_key = untrimmed_api_key.trim();

	let spins = spinitron::get_recent_spins(api_key)?; // TODO: make the fallback equal to some text
	let fallback_contents = WindowContents::make_texture_from_path("assets/wbor_plane.bmp", &texture_creator);
	let curr_album_contents = spinitron::get_curr_album_contents(&spins, &texture_creator, fallback_contents)?;

	//////////

	/* TODO:
	- Maybe put the bounding box definition one layer out (with the parent)
	- Abstract the main loop out, so that just some data and fns are passed into it
	- Check for no box intersections
	- Put the box definitions in a JSON file
	- Avoid screen burn-in somehow
	*/

	// This can be extended to update textures for things like album covers
	fn get_new_color_for_blue_box() -> Option<WindowContents<'static>> {
		// None
		Some(WindowContents::make_color(0, 0, 127))
	}

	let album_cover = HierarchalWindow::new(
		None,
		curr_album_contents,
		Vec2f::new(0.1, 0.1),
		Vec2f::new(0.3, 0.9),
		None
	);

	let bird = HierarchalWindow::new(
		None,
		WindowContents::make_texture_from_path("assets/bird.bmp", &texture_creator),
		Vec2f::new(0.4, 0.1),
		Vec2f::new(0.7, 0.9),
		None
	);

	let photo_box = HierarchalWindow::new(
		None,
		WindowContents::make_transparent_color(0, 255, 0, 0.8),
		Vec2f::new(0.01, 0.01),
		Vec2f::new(0.75, 0.5),
		Some(vec![album_cover, bird])
	);

	let blue_box = HierarchalWindow::new(
		Some(get_new_color_for_blue_box),
		WindowContents::make_color(0, 0, 255),
		Vec2f::new(0.1, 0.6),
		Vec2f::new(0.9, 0.9),
		None
	);

	let mut example_window = HierarchalWindow::new(
		None,
		WindowContents::make_color(255, 0, 0),
		Vec2f::new(0.01, 0.01),
		Vec2f::new(0.99, 0.99),
		Some(vec![photo_box, blue_box])
	);

	//////////

	'running: loop {
		for event in event_pump.poll_iter() {
			use sdl2::event::Event;
			use sdl2::keyboard::Keycode;

			match event {
				Event::Quit {..} | Event::KeyDown {keycode: Some(Keycode::Escape), ..} => break 'running,
				_ => {}
			}
		}

		sdl_canvas.set_draw_color(config.bg_color); // TODO: remove this eventually
		sdl_canvas.clear();

		render_windows_recursively(&mut example_window, &mut sdl_canvas, sdl_window_bounds);

		sdl_canvas.present();

		std::thread::sleep(sleep_time);
	}

	Ok(())
}
