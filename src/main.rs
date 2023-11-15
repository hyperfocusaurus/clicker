use std::fmt::Write;
use raylib::prelude::*;
use raylib::core::color::{Color};
use raylib::core::misc::{get_random_value};

macro_rules! clamp {
    ($value: expr, $min: expr, $max:expr) => {
        std::cmp::min(std::cmp::max($value, $min), $max)
    }
}

fn main() {
    let mut width:i32 = 1920;
    let mut height:i32 = 1080;
    let (mut rl, thread) = raylib::init()
        .size(width, height)
        .title("Jiggle Balls")
        .build();

    let mut circles = Vec::new();
    let mut _hue: f32 = 0.0;
    const _EPSILON: f32 = 0.5;
    let mut fullscreen_needed = false;
    let mut jiggle = 3;

    for _i in 0..32 {
        let x = get_random_value(0, width);
        let y = get_random_value(0, height);
        let h = get_random_value::<i32>(0, 360) as f32;
        let color = Color::color_from_hsv(h, 1.0, 1.0);
        let circle_size = get_random_value::<i32>(5, 15) as f32;
        circles.push((x, y, circle_size, color));
    }
     
    while !rl.window_should_close() {
        if rl.is_window_focused() {
            rl.hide_cursor();
        }
        if fullscreen_needed {
            rl.toggle_fullscreen();
            rl.set_window_size(width, height);
            fullscreen_needed = !fullscreen_needed;
        }

        let mut d = rl.begin_drawing(&thread);

        if d.is_key_pressed(KeyboardKey::KEY_MINUS) {
            jiggle -= 1;
        }

        if d.is_key_pressed(KeyboardKey::KEY_EQUAL) {
            jiggle += 1;
        }
        
        if d.is_key_pressed(KeyboardKey::KEY_F) {
            fullscreen_needed = !fullscreen_needed;
            if fullscreen_needed {
                width = d.get_screen_width();
                height = d.get_screen_height();
            } else {
                width = 1920;
                height = 1080;
            }
        }

        d.clear_background(Color::BLACK);

        circles = circles.iter().map(|circ| {
            let (x, y, circle_size, color) = circ;
            let mut jiggle_x:i32 = get_random_value(-jiggle, jiggle);
            let mut jiggle_y:i32 = get_random_value(-jiggle, jiggle);

            d.draw_circle(*x, *y, *circle_size, color);

            // collision detection
            for other in &circles[..] {
                if *other == *circ {
                    continue;
                }
                // sqrt (pow(abs(other_x - x), 2) + pow(abs(other_y - y), 2))
                let (other_x, other_y, other_size, _) = other;
                let x_dist = (other_x - *x).abs();
                let y_dist = (other_y - *y).abs();
                let dist = ((x_dist.pow(2) + y_dist.pow(2)) as f32).sqrt();
                if dist < (circle_size + other_size)/2.0 {
                    jiggle_x = -jiggle_x;
                    jiggle_y = -jiggle_y;
                }
            }

            (clamp!(*x+jiggle_x, *circle_size as i32, width - *circle_size as i32), 
             clamp!(*y+jiggle_y, *circle_size as i32, height - *circle_size as i32),
             *circle_size,
             *color)
        }).collect();
        let mut s = String::new();
        write!(s, "Jiggle: {jiggle}").unwrap();
        d.draw_text(s.as_str(), 0, 0, 32, Color::BLUE);
    }
}
