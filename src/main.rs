// so ssl2 and ssl2_fft are found by the rust compiler on macOS
// export LIBRARY_PATH="/opt/homebrew/lib:$LIBRARY_PATH"
// export PKG_CONFIG_PATH="/opt/homebrew/lib/pkgconfig:$PKG_CONFIG_PATH"
// export C_INCLUDE_PATH="/opt/homebrew/include:$C_INCLUDE_PATH"
//
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use sdl2::ttf::Font;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq)]
enum MazeTile {
    Empty,          // Open space (Pac-Man can move here)
    WallVertical,   // | (vertical wall)
    WallHorizontal, // - (horizontal wall)
    WallCornerNE,   // ⌜ (northeast corner)
    WallCornerNW,   // ⌝ (northwest corner)
    WallCornerSE,   // ⌞ (southeast corner)
    WallCornerSW,   // ⌟ (southwest corner)
}

fn main() -> Result<(), String> {
    // Initialize SDL2
    let sdl_context = sdl2::init()?;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
    let video_subsystem = sdl_context.video()?;

    let font_path = "./font.ttf"; // Replace with a valid TTF font path
    let font = ttf_context.load_font(font_path, 24)?;

    let player_radius = 12.0;
    let player_diameter = player_radius * 2.0;
    let speed = player_radius / 3.0; // Pixels per second
    let tile_width = player_radius * 2.0;
    let tile_height = player_radius * 2.0;

    let grid_width = tile_width * 34.0;
    let grid_height = tile_height * 34.0;

    // Create a window
    let window = video_subsystem
        .window("Pacman", grid_width as u32, grid_height as u32)
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
    let mut player_pos = (
        canvas.window().size().0 as f32 / 2.0,
        canvas.window().size().1 as f32 / 2.0,
    );
    let mut player_direction = (0.0, 0.0);
    let mut last_direction = player_direction;

    let mut tile_scents: HashMap<(usize, usize), PacmanScent> = HashMap::new();

    let mut pacman_mouth_angle = 45.0; // Degrees
    let mut pacman_mouth_opening = false;
    let mut pacman_mouth_speed = 5.0; // degrees per second (could increase as pacman gets faster)

    let maze = initialize_maze();

    'running: loop {
        //let current_time = std::time::Instant::now();
        //let dt = current_time.duration_since(last_time).as_secs_f32();
        //last_time = current_time;

        // Handle events
        for event in event_pump.poll_iter() {
            if let Event::Quit { .. } = event {
                break 'running;
            }
        }

        handle_keypress(&mut player_direction, &event_pump);

        // Light up the tile where the player is located
        let player_tile = get_tile(&player_pos, tile_width, tile_height, player_radius);
        if let Some(tile) = player_tile {
            light_up_tile(tile, 2000, &mut tile_scents);
        }

        // update_player_position(&mut player_pos, &player_direction, speed);
        update_player_position(
            &mut player_pos,
            &player_direction,
            speed,
            &maze,
            tile_width,
            tile_height,
            player_radius,
        );

        adjust_player_position_delete_me_later(
            tile_width,
            tile_height,
            &mut player_pos,
            &mut player_direction,
            &mut last_direction,
        );

        handle_player_screen_wrapping(
            &mut player_pos,
            player_diameter,
            canvas.window().size().0 as f32,
            canvas.window().size().1 as f32,
        );

        // Clear the screen
        clear_background(&mut canvas);

        // Draw the grid
        draw_grid(
            &mut canvas,
            tile_width,
            tile_height,
            grid_width,
            grid_height,
            1.0,
        )?;

        draw_maze(&mut canvas, &maze, tile_width, tile_height)?;

        // Draw the circle
        draw_pacman(
            &mut canvas,
            (player_pos.0, player_pos.1),
            player_radius,
            pacman_mouth_angle,
            player_direction,
        )?;

        // Update the mouth angle
        update_pacman_mouth_angle(
            &mut pacman_mouth_opening,
            &mut pacman_mouth_angle,
            pacman_mouth_speed,
        );

        // Update and draw lit tiles
        update_tile_scent(
            &mut canvas,
            tile_width,
            tile_height,
            &mut tile_scents,
            false,
        );

        // Render the player's position as text
        render_player_position_hud(
            &mut canvas,
            &player_pos,
            &font,
            tile_width,
            tile_height,
            player_radius,
        );

        // Present the canvas
        canvas.present();

        // Delay to cap the frame rate at ~60 FPS
        std::thread::sleep(Duration::from_millis(16));
    }

    Ok(())
} // main

// Add this near the top of main.rs
fn initialize_maze() -> Vec<Vec<MazeTile>> {
    let width = 34;
    let height = 34;
    let mut maze = vec![vec![MazeTile::Empty; width]; height];

    // Simple test maze: a box around the edges
    for x in 0..width {
        maze[0][x] = MazeTile::WallHorizontal; // Top edge
        maze[height - 1][x] = MazeTile::WallHorizontal; // Bottom edge
    }
    for y in 0..height {
        maze[y][0] = MazeTile::WallVertical; // Left edge
        maze[y][width - 1] = MazeTile::WallVertical; // Right edge
    }
    // Corners
    maze[0][0] = MazeTile::WallCornerNW;
    maze[0][width - 1] = MazeTile::WallCornerNE;
    maze[height - 1][0] = MazeTile::WallCornerSW;
    maze[height - 1][width - 1] = MazeTile::WallCornerSE;

    // Add a few internal walls for testing
    maze[5][5] = MazeTile::WallVertical;
    maze[5][6] = MazeTile::WallVertical;
    maze[6][5] = MazeTile::WallHorizontal;
    maze[6][6] = MazeTile::WallCornerSE;

    maze
} // initialize_maze

fn draw_maze(
    canvas: &mut WindowCanvas,
    maze: &Vec<Vec<MazeTile>>,
    tile_width: f32,
    tile_height: f32,
) -> Result<(), String> {
    canvas.set_draw_color(Color::RGB(0, 0, 255)); // Blue walls

    let wall_thickness = (tile_width / 4.0) as u32; // Thinner walls

    for (y, row) in maze.iter().enumerate() {
        for (x, tile) in row.iter().enumerate() {
            let x_pos = (x as f32 * tile_width) as i32;
            let y_pos = (y as f32 * tile_height) as i32;

            match tile {
                MazeTile::WallVertical => {
                    canvas.fill_rect(Rect::new(
                        x_pos + (tile_width / 2.0) as i32 - (wall_thickness / 2) as i32,
                        y_pos,
                        wall_thickness,
                        tile_height as u32,
                    ))?;
                }
                MazeTile::WallHorizontal => {
                    canvas.fill_rect(Rect::new(
                        x_pos,
                        y_pos + (tile_height / 2.0) as i32 - (wall_thickness / 2) as i32,
                        tile_width as u32,
                        wall_thickness,
                    ))?;
                }
                MazeTile::WallCornerNW => {
                    // Top-left corner: vertical down from top, horizontal right
                    canvas.fill_rect(Rect::new(
                        x_pos + (tile_width / 2.0) as i32 - (wall_thickness / 2) as i32,
                        y_pos,
                        wall_thickness,
                        (tile_height / 2.0) as u32 + wall_thickness / 2,
                    ))?;
                    canvas.fill_rect(Rect::new(
                        x_pos,
                        y_pos + (tile_height / 2.0) as i32 - (wall_thickness / 2) as i32,
                        (tile_width / 2.0) as u32 + wall_thickness / 2,
                        wall_thickness,
                    ))?;
                }
                MazeTile::WallCornerNE => {
                    // Top-right corner: vertical down from top, horizontal left
                    canvas.fill_rect(Rect::new(
                        x_pos + (tile_width / 2.0) as i32 - (wall_thickness / 2) as i32,
                        y_pos,
                        wall_thickness,
                        (tile_height / 2.0) as u32 + wall_thickness / 2,
                    ))?;
                    canvas.fill_rect(Rect::new(
                        x_pos + (tile_width / 2.0) as i32 - (wall_thickness / 2) as i32,
                        y_pos + (tile_height / 2.0) as i32 - (wall_thickness / 2) as i32,
                        (tile_width / 2.0) as u32 + wall_thickness / 2,
                        wall_thickness,
                    ))?;
                }
                MazeTile::WallCornerSW => {
                    // Bottom-left corner: vertical up from bottom, horizontal right
                    canvas.fill_rect(Rect::new(
                        x_pos + (tile_width / 2.0) as i32 - (wall_thickness / 2) as i32,
                        y_pos + (tile_height / 2.0) as i32 - (wall_thickness / 2) as i32,
                        wall_thickness,
                        (tile_height / 2.0) as u32 + wall_thickness / 2,
                    ))?;
                    canvas.fill_rect(Rect::new(
                        x_pos,
                        y_pos + (tile_height / 2.0) as i32 - (wall_thickness / 2) as i32,
                        (tile_width / 2.0) as u32 + wall_thickness / 2,
                        wall_thickness,
                    ))?;
                }
                MazeTile::WallCornerSE => {
                    // Bottom-right corner: vertical up from bottom, horizontal left
                    canvas.fill_rect(Rect::new(
                        x_pos + (tile_width / 2.0) as i32 - (wall_thickness / 2) as i32,
                        y_pos + (tile_height / 2.0) as i32 - (wall_thickness / 2) as i32,
                        wall_thickness,
                        (tile_height / 2.0) as u32 + wall_thickness / 2,
                    ))?;
                    canvas.fill_rect(Rect::new(
                        x_pos + (tile_width / 2.0) as i32 - (wall_thickness / 2) as i32,
                        y_pos + (tile_height / 2.0) as i32 - (wall_thickness / 2) as i32,
                        (tile_width / 2.0) as u32 + wall_thickness / 2,
                        wall_thickness,
                    ))?;
                }
                MazeTile::Empty => {} // No drawing for empty tiles
            }
        }
    }
    Ok(())
}

fn adjust_player_position_delete_me_later(
    tile_width: f32,
    tile_height: f32,
    player_pos: &mut (f32, f32),
    player_direction: &mut (f32, f32),
    last_direction: &mut (f32, f32),
) {
    // Check if the direction has changed
    if *player_direction != *last_direction {
        if player_direction.0 != 0.0 {
            // Moving horizontally, re-center y position
            player_pos.0 = ((player_pos.0 / tile_width).round() * tile_width) + tile_width / 2.0;
        }
        if player_direction.1 != 0.0 {
            // Moving vertically, re-center x position
            player_pos.1 = ((player_pos.1 / tile_height).round() * tile_height) + tile_height / 2.0;
        }
        *last_direction = *player_direction;
    }
}

fn clear_background(canvas: &mut WindowCanvas) {
    canvas.set_draw_color(Color::RGB(0, 0, 0)); // Purple background
    canvas.clear();
}

fn update_pacman_mouth_angle(
    pacman_mouth_opening: &mut bool,
    pacman_mouth_angle: &mut f32,
    pacman_mouth_speed: f32,
) {
    if *pacman_mouth_opening {
        *pacman_mouth_angle += pacman_mouth_speed;
        if *pacman_mouth_angle >= 70.0 {
            *pacman_mouth_angle = 70.0;
            *pacman_mouth_opening = false;
        }
    } else {
        *pacman_mouth_angle -= pacman_mouth_speed;
        if *pacman_mouth_angle <= 5.0 {
            *pacman_mouth_angle = 5.0;
            *pacman_mouth_opening = true;
        }
    }
}

fn draw_grid(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    tile_width: f32,
    tile_height: f32,
    grid_width: f32,
    grid_height: f32,
    grid_line_thickness: f32,
) -> Result<(), String> {
    canvas.set_draw_color(Color::RGB(255, 255, 255)); // White lines

    // Draw vertical lines
    for col in 0..=grid_width as i32 {
        let x = (col as f32 * tile_width) as i32;
        canvas.fill_rect(Rect::new(
            x,
            0,
            grid_line_thickness as u32,
            (grid_height * tile_height) as u32,
        ))?;
    }

    // Draw horizontal lines
    for row in 0..=grid_height as i32 {
        let y = (row as f32 * tile_height) as i32;
        canvas.fill_rect(Rect::new(
            0,
            y,
            (grid_width * tile_width) as u32,
            grid_line_thickness as u32,
        ))?;
    }

    Ok(())
}

fn draw_pacman(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    center: (f32, f32),
    radius: f32,
    mouth_angle: f32,
    player_direction: (f32, f32),
) -> Result<(), String> {
    let (cx, cy) = center;
    let r = radius as i32;

    // Helper function to compute minimal angle difference
    fn angle_diff(a: f32, b: f32) -> f32 {
        let mut diff = a - b;
        while diff < -180.0 {
            diff += 360.0;
        }
        while diff > 180.0 {
            diff -= 360.0;
        }
        diff.abs()
    }

    // Determine the desired angle based on player direction
    let desired_angle = if player_direction.0 == 1.0 {
        0.0
    } else if player_direction.0 == -1.0 {
        180.0
    } else if player_direction.1 == -1.0 {
        -90.0
    } else if player_direction.1 == 1.0 {
        90.0
    } else {
        0.0 // Default to facing right if no direction
    };

    for w in 0..r * 2 {
        for h in 0..r * 2 {
            let dx = w - r; // Horizontal offset from center
            let dy = h - r; // Vertical offset from center

            // Convert dx and dy to få2
            let dx_f32 = dx as f32;
            let dy_f32 = dy as f32;

            // Compute angle from positive x-axis to point (dx, dy)
            let angle = dy_f32.atan2(dx_f32).to_degrees();
            let delta_angle = angle_diff(angle, desired_angle);

            // Check if the point is outside the mouth's opening
            if dx * dx + dy * dy <= r * r && delta_angle > mouth_angle / 2.0 {
                canvas.set_draw_color(Color::RGB(255, 255, 0)); // Yellow circle
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

// fn update_player_position(player_pos: &mut (f32, f32), player_direction: &(f32, f32), speed: f32) {
//     player_pos.0 += speed * player_direction.0;
//     player_pos.1 += speed * player_direction.1;
// }

fn update_player_position(
    player_pos: &mut (f32, f32),
    player_direction: &(f32, f32),
    speed: f32,
    maze: &Vec<Vec<MazeTile>>,
    tile_width: f32,
    tile_height: f32,
    player_radius: f32,
) {
    let next_x = player_pos.0 + speed * player_direction.0;
    let next_y = player_pos.1 + speed * player_direction.1;

    let next_tile = get_tile(&(next_x, next_y), tile_width, tile_height, player_radius);
    let can_move = if let Some((col, row)) = next_tile {
        if col < maze[0].len() && row < maze.len() {
            maze[row][col] == MazeTile::Empty
        } else {
            true // Allow movement outside maze (for wrapping)
        }
    } else {
        true // No tile means edge case, allow for now
    };

    if can_move {
        player_pos.0 = next_x;
        player_pos.1 = next_y;
    }
}

fn render_player_position_hud(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    player_pos: &(f32, f32),
    font: &Font,
    tile_width: f32,
    tile_height: f32,
    player_radius: f32,
) {
    let tile = get_tile(player_pos, tile_width, tile_height, player_radius);
    let mut tile_text = String::from("Tile: (???, ???)");
    if let Some(tile) = tile {
        tile_text = format!("Tile: ({:02}, {:02})", tile.0, tile.1);
    }

    let player_text = format!(
        "Pos: ({:03}, {:03}) {}",
        player_pos.0 as i32, player_pos.1 as i32, tile_text
    );

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

    // Draw the text in the top-right corner
    let dst_rect = Rect::new(
        (window_width - text_width) as i32 - 10, // 10px padding
        10,                                      // 10px from the top
        text_width,
        text_height,
    );

    canvas.copy(&texture, None, dst_rect).unwrap();
}

fn handle_player_screen_wrapping(
    player_pos: &mut (f32, f32),
    player_diameter: f32,
    screen_width: f32,
    screen_height: f32,
) {
    let radius = player_diameter / 2.0;
    // west border
    if player_pos.0 < -player_diameter {
        player_pos.0 = screen_width + radius;
    }
    // east border
    else if player_pos.0 > screen_width + player_diameter {
        player_pos.0 = -radius;
    }
    // north border
    else if player_pos.1 < -player_diameter {
        player_pos.1 = screen_height + radius;
    }
    // south border
    else if player_pos.1 > screen_height + player_diameter {
        player_pos.1 = -radius;
    }
}

fn get_tile(
    pos: &(f32, f32),
    tile_width: f32,
    tile_height: f32,
    player_radius: f32,
) -> Option<(usize, usize)> {
    let (x, y) = pos;
    let col = (x / tile_width).floor() as usize;
    let row = (y / tile_height).floor() as usize;

    let tile_center_x = (col as f32 + 0.5) * tile_width;
    let tile_center_y = (row as f32 + 0.5) * tile_height;

    let distance_x = (x - tile_center_x).abs();
    let distance_y = (y - tile_center_y).abs();

    if distance_x <= player_radius && distance_y <= player_radius {
        Some((col, row))
    } else {
        None
    }
}

struct PacmanScent {
    start_time: Instant,
    duration: Duration,
}

fn light_up_tile(
    tile_pos: (usize, usize),
    duration: u64,
    tile_lights: &mut HashMap<(usize, usize), PacmanScent>,
) {
    let now = Instant::now();
    let duration = Duration::from_millis(duration);
    tile_lights.insert(
        tile_pos,
        PacmanScent {
            start_time: now,
            duration,
        },
    );
}

fn update_tile_scent(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    tile_width: f32,
    tile_height: f32,
    tile_lights: &mut HashMap<(usize, usize), PacmanScent>,
    render_scent: bool,
) {
    let now = Instant::now();
    tile_lights.retain(|&(col, row), tile_light| {
        let elapsed = now.duration_since(tile_light.start_time);
        if elapsed < tile_light.duration {
            if render_scent {
                let progress = elapsed.as_secs_f32() / tile_light.duration.as_secs_f32();
                let brightness = (1.0 - progress) * 200.0;
                canvas.set_draw_color(Color::RGB(
                    brightness as u8,
                    brightness as u8,
                    brightness as u8,
                ));
                let x = (col as f32 * tile_width) as i32;
                let y = (row as f32 * tile_height) as i32;
                canvas
                    .fill_rect(Rect::new(x, y, tile_width as u32, tile_height as u32))
                    .unwrap();
            }
            true
        } else {
            false
        }
    });
}
