use std::fmt::Write;
use raylib::prelude::*;
use raylib::core::color::{Color};
use raylib::core::misc::{get_random_value};

macro_rules! clamp {
    ($value: expr, $min: expr, $max:expr) => {
        std::cmp::min(std::cmp::max($value, $min), $max)
    }
}

const DEFAULT_WIDTH:i32 = 1920;
const DEFAULT_HEIGHT:i32 = 1080;
const EPSILON:f32 = 0.1;

fn main() {
    let mut width:i32 = DEFAULT_WIDTH;
    let mut height:i32 = DEFAULT_HEIGHT;
    let (mut rl, thread) = raylib::init()
        .size(width, height)
        .transparent()
        .title("Jiggle Balls")
        .build();

    let mut circles = Vec::new();
    let mut fullscreen_requested = false;
    let mut jiggle = 3;

    for _i in 0..1000 {
        let x = get_random_value(0, width);
        let y = get_random_value(0, height);
        let h = get_random_value::<i32>(0, 360) as f32;
        let color = Color::color_from_hsv(h, 1.0, 1.0);
        let circle_size = get_random_value::<i32>(5, 15) as f32;
        let velocity = Vector2::new(0.0, 0.0);
        circles.push((x, y, circle_size, color, velocity));
    }
     
    loop {
        let delta_time = rl.get_frame_time();
        if rl.window_should_close() {
            break;
        }

        if rl.is_window_focused() {
            rl.hide_cursor();
        }
        if fullscreen_requested {
            rl.toggle_fullscreen();
            rl.set_window_size(width, height);
            fullscreen_requested = !fullscreen_requested;
        }

        let mut d = rl.begin_drawing(&thread);

        if d.is_key_pressed(KeyboardKey::KEY_MINUS) {
            jiggle -= 1;
        }

        if d.is_key_pressed(KeyboardKey::KEY_EQUAL) {
            jiggle += 1;
        }

        if d.is_key_pressed(KeyboardKey::KEY_Q) {
            break;
        }
        
        if d.is_key_pressed(KeyboardKey::KEY_F) {
            fullscreen_requested = !fullscreen_requested;
            if fullscreen_requested {
                width = d.get_screen_width();
                height = d.get_screen_height();
            } else {
                width = DEFAULT_WIDTH;
                height = DEFAULT_HEIGHT;
            }
        }

        d.clear_background(Color::new(0x00, 0x00, 0x00, 0xA0));
        let mouse_x = d.get_mouse_x();
        let mouse_y = d.get_mouse_y();

        circles = circles.iter().map(|circ| {
            let (x, y, circle_size, color, velocity) = circ;
            // draw the circle before doing anything else - everything else is setting up for the
            // next frame
            d.draw_circle(*x, *y, *circle_size, color);
            let jiggle_x:i32 = get_random_value(-jiggle, jiggle);
            let jiggle_y:i32 = get_random_value(-jiggle, jiggle);
            let mut new_x = *x;
            let mut new_y = *y;
            let mut new_velocity = velocity.clone();
            const DRAG_COEFFICIENT: f32 = 1000.0;

            // "drag" simulation
            if velocity.x != 0.0 {
                new_x += (velocity.x * delta_time).round() as i32;
                if new_velocity.x < 0.0 {
                    new_velocity.x += (EPSILON * delta_time * DRAG_COEFFICIENT).powi(2);
                } else {
                    new_velocity.x -= (EPSILON * delta_time * DRAG_COEFFICIENT).powi(2);
                }
                // if we're within EPSILON of zero, make us zero instead
                // this stops us flapping around zero
                if (new_velocity.x - EPSILON).abs() < EPSILON {
                    new_velocity.x = 0.0;
                }
            }

            // "drag" simulation
            if velocity.y != 0.0 {
                new_y += (velocity.y * delta_time).round() as i32;
                if new_velocity.y < 0.0 {
                    new_velocity.y += (EPSILON * delta_time * DRAG_COEFFICIENT).powi(2);
                } else {
                    new_velocity.y -= (EPSILON * delta_time * DRAG_COEFFICIENT).powi(2);
                }
                // if we're within EPSILON of zero, make us zero instead
                // this stops us flapping around zero
                if (new_velocity.y - EPSILON).abs() < EPSILON {
                    new_velocity.y = 0.0;
                }
            }


            // collision detection
            for other in &circles[..] {
                if *other == *circ {
                    continue;
                }
                // sqrt (pow(abs(other_x - x), 2) + pow(abs(other_y - y), 2))
                let (other_x, other_y, other_size, _, _) = other;
                let x_dist = other_x - *x;
                let y_dist = other_y - *y;
                let dist = ((x_dist.pow(2) + y_dist.pow(2)) as f32).sqrt();
                if dist < (*circle_size + other_size) {
                    new_velocity.x -= x_dist as f32;
                    new_velocity.y -= y_dist as f32;
                }
            }

            const MOUSE_GRAVITY: f32 = 2.0;
            let mouse_x_dist = *x - mouse_x;
            let mouse_y_dist = *y - mouse_y;
            let mouse_dist = ((mouse_x_dist.pow(2) + mouse_y_dist.pow(2)) as f32).sqrt();
            if mouse_dist < 100.0 {
                new_velocity += Vector2::new(mouse_x_dist as f32, mouse_y_dist as f32) * MOUSE_GRAVITY;
            }
            new_x += jiggle_x;

            if new_x >= width || new_x <= 0 {
                new_velocity.x = -new_velocity.x;
            }

            new_y += jiggle_y;
            if new_y >= height || new_y <= 0 {
                new_velocity.y = -new_velocity.y;
            }
                
            (clamp!(new_x, 0, width), 
             clamp!(new_y, 0, height),
             *circle_size,
             *color,
             new_velocity)
        }).collect();

        d.draw_circle(mouse_x, mouse_y, 30.0, Color::BLUE);

        let mut s = String::new();
        write!(s, "Jiggle: {jiggle}").unwrap();
        d.draw_text(s.as_str(), 0, 0, 32, Color::BLUE);
        d.draw_fps(0, 32);
    }
}
