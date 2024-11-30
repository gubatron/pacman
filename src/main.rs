use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::time::Duration;

fn main() -> Result<(), String> {
    // Initialize SDL2
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    // Create a window
    let window = video_subsystem
        .window("SDL2 Circle Movement", 1280, 720)
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
    let player_radius = 40.0;
    let speed = 300.0; // Pixels per second

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

        // Handle key presses
        let keys: Vec<Keycode> = event_pump
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect();

        if keys.contains(&Keycode::W) {
            player_pos.1 -= speed * dt;
        }
        if keys.contains(&Keycode::S) {
            player_pos.1 += speed * dt;
        }
        if keys.contains(&Keycode::A) {
            player_pos.0 -= speed * dt;
        }
        if keys.contains(&Keycode::D) {
            player_pos.0 += speed * dt;
        }

        // Clear the screen
        canvas.set_draw_color(Color::RGB(128, 0, 128)); // Purple background
        canvas.clear();

        // Draw the circle
        draw_circle(&mut canvas, player_pos, player_radius)?;

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
                canvas.set_draw_color(Color::RGB(255, 0, 0)); // Red circle
                canvas.draw_point((cx as i32 + dx, cy as i32 + dy))?;
            }
        }
    }
    Ok(())
}
