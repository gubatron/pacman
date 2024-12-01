// export LIBRARY_PATH="/opt/homebrew/lib:$LIBRARY_PATH"
// export PKG_CONFIG_PATH="/opt/homebrew/lib/pkgconfig:$PKG_CONFIG_PATH"
// export C_INCLUDE_PATH="/opt/homebrew/include:$C_INCLUDE_PATH"
//
//
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::ttf::Font;
use std::time::Duration;

fn main() -> Result<(), String> {
    // Initialize SDL2
    let sdl_context = sdl2::init()?;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
    let video_subsystem = sdl_context.video()?;

    let font_path = "./font.ttf"; // Replace with a valid TTF font path
    let font = ttf_context.load_font(font_path, 24)?;

    // Create a window
    let window = video_subsystem
        .window("Pacman", 850, 850)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;
    let mut event_pump = sdl_context.event_pump()?;

    // Set the initial position of the circle
    let mut player_pos = (640.0, 360.0);
    let mut player_direction = (0.0, 0.0);
    let player_radius = 12.0;
    let speed = 15.0; // Pixels per second

    let mut last_time = std::time::Instant::now();

    'running: loop {
        let current_time = std::time::Instant::now();
        let dt = current_time.duration_since(last_time).as_secs_f32();
        last_time = current_time;

        // Handle events
        for event in event_pump.poll_iter() {
            if let Event::Quit { .. } = event {
                break 'running;
            }
        }

        handle_keypress(&mut player_direction, &event_pump);
        update_player_position(&mut player_pos, &player_direction, speed);

        // Clear the screen
        canvas.set_draw_color(Color::RGB(0, 0, 0)); // Purple background
        canvas.clear();

        render_player_position_hud(&mut canvas, &player_pos, &font);

        // Draw the circle
        draw_circle(
            &mut canvas,
            (player_pos.0 as f32, player_pos.1 as f32),
            player_radius as f32,
        )?;

        // Present the canvas
        canvas.present();

        // Delay to cap the frame rate at ~60 FPS
        std::thread::sleep(Duration::from_millis(16));
    }

    Ok(())
}

fn draw_circle(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    center: (f32, f32),
    radius: f32,
) -> Result<(), String> {
    let (cx, cy) = center;
    let r = radius as i32;
    for w in 0..r * 2 {
        for h in 0..r * 2 {
            let dx = r - w; // Horizontal offset
            let dy = r - h; // Vertical offset
            if dx * dx + dy * dy <= r * r {
                canvas.set_draw_color(Color::RGB(255, 255, 0)); // Red circle
                canvas.draw_point((cx as i32 + dx, cy as i32 + dy))?;
            }
        }
    }
    Ok(())
}

fn handle_keypress(player_direction: &mut (f32, f32), event_pump: &sdl2::EventPump) {
    // Handle key presses
    let keys: Vec<Keycode> = event_pump
        .keyboard_state()
        .pressed_scancodes()
        .filter_map(Keycode::from_scancode)
        .collect();

    if keys.contains(&Keycode::W) || keys.contains(&Keycode::Up) {
        player_direction.1 = -1.0;
        player_direction.0 = 0.0;
    }
    if keys.contains(&Keycode::S) || keys.contains(&Keycode::Down) {
        player_direction.1 = 1.0;
        player_direction.0 = 0.0;
    }
    if keys.contains(&Keycode::A) || keys.contains(&Keycode::Left) {
        player_direction.0 = -1.0;
        player_direction.1 = 0.0;
    }
    if keys.contains(&Keycode::D) || keys.contains(&Keycode::Right) {
        player_direction.0 = 1.0;
        player_direction.1 = 0.0;
    }
}

fn update_player_position(player_pos: &mut (f32, f32), player_direction: &(f32, f32), speed: f32) {
    player_pos.0 += speed * player_direction.0;
    player_pos.1 += speed * player_direction.1;
}

fn render_player_position_hud(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    player_pos: &(f32, f32),
    font: &Font,
) {
    let player_text = format!("Pos: ({:.1}, {:.1})", player_pos.0, player_pos.1);

    // Create a surface with the text
    let surface = font
        .render(&player_text)
        .blended(Color::RGB(255, 255, 255))
        .unwrap();

    // Create a texture from the surface
    let texture_creator = canvas.texture_creator();
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .unwrap();

    // Determine where to draw the text
    let text_width = surface.width();
    let text_height = surface.height();
    let window_width = canvas.viewport().width();

    // Draw the text at the top-right corner
    let dst_rect = Rect::new(
        (window_width - text_width) as i32 - 10, // 10px padding
        10,                                      // 10px from the top
        text_width,
        text_height,
    );

    canvas.copy(&texture, None, dst_rect).unwrap();
}
