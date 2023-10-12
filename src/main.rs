use sdl2;

mod spinitron;
mod window_hierarchy;
mod dynamic_optional;
mod texture;
mod request;

use window_hierarchy::{
	ColorSDL, Vec2f, WindowContents, HierarchalWindow
};

/*
Worked from this in the beginning: https://blog.logrocket.com/using-sdl2-bindings-rust/

TODO:
- Maybe give a retro theme to everything
- Maybe put the bounding box definition one layer out (with the parent)
- Abstract the main loop out, so that just some data and fns are passed into it
- Check for no box intersections
- Put the box definitions in a JSON file
- Avoid screen burn-in somehow
*/

struct AppConfig<'a> {
	name: &'a str,
	width: u32,
	height: u32,
	fps: u32,
	bg_color: ColorSDL
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
	let config = AppConfig {
		name: "Recursive Box Demo",
		width: 800, height: 600, // The CRT aspect ratio
		fps: 60,
		bg_color: ColorSDL::RGB(50, 50, 50)
	};

	let sdl_context = sdl2::init()?;
	let sdl_video_subsystem = sdl_context.video()?;

	let mut sdl_event_pump = sdl_context.event_pump()?;

	let sdl_window = sdl_video_subsystem
		.window(config.name, config.width, config.height)
		.position_centered().opengl().build()
		.map_err(|e| e.to_string())?;

	let mut sdl_canvas = sdl_window.into_canvas().build().map_err(|e| e.to_string())?;

	let texture_creator = sdl_canvas.texture_creator();
	let mut texture_pool = texture::TexturePool::new(&texture_creator);

	////////// Getting the current spins and album texture, as a test

	let untrimmed_api_key = std::fs::read_to_string("assets/spinitron_api_key.txt")?;
	let api_key = untrimmed_api_key.trim();
	let spins = spinitron::get_recent_spins(api_key)?; // TODO: make the fallback equal to some text

	let fallback_contents = WindowContents::Texture(texture_pool.make_texture_from_path("assets/wbor_plane.bmp")?);
	let curr_album_contents = spinitron::get_curr_album_contents(&spins, &mut texture_pool, fallback_contents)?;

	//////////

	let album_cover = HierarchalWindow::new(
		None,
		None,
		curr_album_contents,
		Vec2f::new(0.4, 0.1),
		Vec2f::new(0.7, 0.9),
		None
	);

	let bird = HierarchalWindow::new(
		None,
		None,
		WindowContents::Texture(texture_pool.make_texture_from_path("assets/bird.bmp")?),
		Vec2f::new(0.1, 0.1),
		Vec2f::new(0.3, 0.9),
		None
	);

	let photo_box = HierarchalWindow::new(
		None,
		None,
		WindowContents::make_transparent_color(0, 255, 0, 0.8),
		Vec2f::new(0.01, 0.01),
		Vec2f::new(0.75, 0.5),
		Some(vec![album_cover, bird])
	);

	let mut example_window = HierarchalWindow::new(
		None,
		None,
		WindowContents::make_color(255, 0, 0),
		Vec2f::new(0.01, 0.01),
		Vec2f::new(0.99, 0.99),
		Some(vec![photo_box])
	);

	//////////

	let sleep_time = std::time::Duration::new(0, 1_000_000_000u32 / config.fps);
	let window_bounds = sdl2::rect::Rect::new(0, 0, config.width, config.height);

	'running: loop {
		for sdl_event in sdl_event_pump.poll_iter() {
			use sdl2::{event::Event, keyboard::Keycode};

			match sdl_event {
				Event::Quit {..} | Event::KeyDown {keycode: Some(Keycode::Escape), ..} => break 'running,
				_ => {}
			}
		}

		sdl_canvas.set_draw_color(config.bg_color); // TODO: remove eventually
		sdl_canvas.clear();

		window_hierarchy::render_windows_recursively(&mut example_window,
			&mut texture_pool, &mut sdl_canvas, window_bounds)?;

		sdl_canvas.present();

		// TODO: sleep for a variable amount of time (or use vsync)
		std::thread::sleep(sleep_time);
	}

	Ok(())
}
