use macroquad::rand::{RandomRange,srand};
use macroquad::ui::root_ui;
use macroquad::prelude::*;
use macroquad::color::hsl_to_rgb;
use std::fmt::Write;

macro_rules! clamp {
    ($value: expr, $min: expr, $max:expr) => {
        std::cmp::min(std::cmp::max($value, $min), $max)
    }
}

const DEFAULT_WIDTH:i32 = 1920;
const DEFAULT_HEIGHT:i32 = 1080;
const EPSILON:f32 = 0.1;

fn get_random_value(min: i32, max: i32) -> i32 {
    i32::gen_range(min, max)
}

fn conf() -> Conf {
    Conf {
        window_title: String::from("Jiggle Balls"),
        window_width: DEFAULT_WIDTH,
        window_height: DEFAULT_HEIGHT,
        fullscreen: false,
        ..Default::default() 
    }
}

#[macroquad::main(conf)]
async fn main() {
    let mut width:i32 = DEFAULT_WIDTH;
    let mut height:i32 = DEFAULT_HEIGHT;

    let mut circles = Vec::new();
    let mut fullscreen_requested = false;
    let mut jiggle = 3;
    let mut draw_gui = true;
    srand(get_time() as u64);

    for _i in 0..1000 {
        let x = get_random_value(0, width);
        let y = get_random_value(0, height);
        let mut h = get_random_value(0, 100) as f32;
        h /= 100.0;
        let color = hsl_to_rgb(h, 1.0, 0.5);
        let circle_size = get_random_value(5, 15) as f32;
        let velocity = Vec2::new(0.0, 0.0);
        circles.push((x, y, circle_size, color, velocity));
    }
     
    loop {
        let delta_time = get_frame_time();

        if root_ui().active_window_focused() {
            show_mouse(false);
        }
        if fullscreen_requested {
            set_fullscreen(true);
            width = screen_width() as i32;
            height = screen_height() as i32;
            // maybe not needed?
            //rl.set_window_size(width, height);
            fullscreen_requested = !fullscreen_requested;
        }

        if is_key_pressed(KeyCode::Minus) {
            jiggle -= 1;
        }

        if is_key_pressed(KeyCode::Equal) {
            jiggle += 1;
        }

        if is_key_pressed(KeyCode::Q) {
            break;
        }

        if is_key_pressed(KeyCode::G) {
            draw_gui = !draw_gui;
        }
        
        if is_key_pressed(KeyCode::F) {
            fullscreen_requested = !fullscreen_requested;
        }

        clear_background(Color::from_rgba(0x00, 0x00, 0x00, 0xC0));
        let (mouse_x, mouse_y) = mouse_position();

        circles = circles.iter().map(|circ| {
            let (x, y, circle_size, color, velocity) = circ;
            // draw the circle before doing anything else - everything else is setting up for the
            // next frame
            draw_circle(*x as f32, *y as f32, *circle_size, *color);
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

            let mut mouse_gravity = 0.0;
            let mut mouse_distance = 100.0;
            // left click to invert gravity
            if is_mouse_button_down(MouseButton::Left) {
                mouse_gravity = -0.15;
                mouse_distance *= 3.0;
            } else if is_mouse_button_down(MouseButton::Right) {
                mouse_gravity = 2.0;
            }
            let mouse_x_dist = *x - mouse_x as i32;
            let mouse_y_dist = *y - mouse_y as i32;
            let mouse_dist = ((mouse_x_dist.pow(2) + mouse_y_dist.pow(2)) as f32).sqrt();
            if mouse_dist < mouse_distance {
                new_velocity += Vec2::new(mouse_x_dist as f32, mouse_y_dist as f32) * mouse_gravity;
            }

            new_x += jiggle_x;
            if new_x >= width-*circle_size as i32 || new_x <= *circle_size as i32 {
                new_velocity.x = -new_velocity.x;
            }

            new_y += jiggle_y;
            if new_y >= height-*circle_size as i32 || new_y <= *circle_size as i32 {
                new_velocity.y = -new_velocity.y;
            }
                
            (clamp!(new_x, *circle_size as i32, width-*circle_size as i32), 
             clamp!(new_y, *circle_size as i32, height-*circle_size as i32),
             *circle_size,
             *color,
             new_velocity)
        }).collect();

        draw_circle(mouse_x, mouse_y, 30.0, BLUE);

        // -- gui layer
        let mut s = String::new();
        write!(s, "Jiggle: {jiggle}").unwrap();
        draw_text(s.as_str(), 0.0, 20.0, 32.0, BLUE);
        let fps = get_fps();
        write!(s, "FPS: {fps}").unwrap();
        draw_text(s.as_str(), 0.0, 52.0, 32.0, BLUE);
        next_frame().await;
    }
}
