use miniquad::window::screen_size;
use quad_rand::{RandomRange, srand};
use macroquad::ui::root_ui;
use macroquad::ui::widgets::Window;
use macroquad::prelude::*;
use macroquad::color::hsl_to_rgb;
use macroquad::hash;
use std::fmt::Write;


const DEFAULT_WIDTH:f32 = 1920.0;
const DEFAULT_HEIGHT:f32 = 1080.0;

fn get_random_value<T: RandomRange>(min: T, max: T) -> T {
    T::gen_range(min, max)
}

fn conf() -> Conf {
    Conf {
        window_title: String::from("Jiggle Balls"),
        window_width: DEFAULT_WIDTH as i32,
        window_height: DEFAULT_HEIGHT as i32,
        fullscreen: false,
        ..Default::default() 
    }
}

fn gen_circle(circles: &mut Vec<(f32, f32, f32, Color, Vec2)>, width: f32, height: f32) {
        let x = get_random_value(0.0, width);
        let y = get_random_value(0.0, height);
        let mut h = get_random_value(0.0, 100.0);
        h /= 100.0;
        let color = hsl_to_rgb(h, 0.5, 0.5);
        let circle_size = get_random_value(5.0, 15.0);
        let velocity = Vec2::new(0.0, 0.0);
        circles.push((x, y, circle_size, color, velocity));
}

fn reset_circles (circles: &mut Vec<(f32, f32, f32, Color, Vec2)>, num_circles: u32, width: f32, height: f32)  {
    circles.clear();
    for _i in 0..num_circles {
        gen_circle(circles, width, height);
    }
}

#[macroquad::main(conf)]
async fn main() {
    let mut width:f32 = DEFAULT_WIDTH;
    let mut height:f32 = DEFAULT_HEIGHT;
    let mut show_gui = false;

    let mut circles = Vec::new();
    let mut is_fullscreen = false;
    let mut jiggle:f32 = 3.0;
    let mut mouse_repel_force = 2.0;
    let mut mouse_attract_force:f32 = 0.15;
    let mut mouse_attract_distance:f32 = 100.0;
    let mut medium_viscosity = 100.0;
    let mut num_circles = 1000;
    let mut num_circles_ui:f32 = 1000.0;
    let mut gravity_enabled = false;
    let mut particle_repel_force = 10.0;

    let ui_font = load_ttf_font("OfficeCodePro-Regular.ttf").await.expect("Could not load UI font");

    srand(get_time() as u64);

    reset_circles(&mut circles, num_circles, width, height);
    let hud_textparams = TextParams {
        font: Some(&ui_font),
        font_size: 32, 
        color: BLUE,
        ..Default::default()
    }; 
    loop {
        let delta_time = get_frame_time();

        num_circles_ui = num_circles_ui.floor();
        num_circles = num_circles_ui as u32;

        if circles.len() < num_circles.try_into().unwrap() {
            for _ in 1..num_circles - circles.len() as u32 {
                gen_circle(&mut circles, width, height);
            }
        } else if circles.len() > num_circles.try_into().unwrap() {
            circles.drain((num_circles as usize)..);
        }
        // truncate some of the floats that deal with pixel values so they're more realistic
        jiggle = jiggle.trunc();
        mouse_attract_distance = mouse_attract_distance.trunc();

        (width, height) = screen_size();
        if !show_gui {
            show_mouse(false);
        } else {
            show_mouse(true);
        }

        if is_key_pressed(KeyCode::Minus) {
            jiggle -= 1.0;
        }

        if is_key_pressed(KeyCode::Equal) {
            jiggle += 1.0;
        }

        if is_key_pressed(KeyCode::Q) {
            break;
        }

        if is_key_pressed(KeyCode::F) {
            is_fullscreen = !is_fullscreen;
            set_fullscreen(is_fullscreen);
            // maybe not needed?
            //request_new_screen_size(width, height);
        }

        if is_key_pressed(KeyCode::G) {
            show_gui = !show_gui;
        }

        if is_key_pressed(KeyCode::R) {
            reset_circles(&mut circles, num_circles, width, height);
        }

        clear_background(Color::from_rgba(0x00, 0x00, 0x00, 0xC0));
        let (mouse_x, mouse_y) = mouse_position();

        circles = circles.iter().map(|circ| {
            let (x, y, circle_size, color, velocity) = circ;
            // draw the circle before doing anything else - everything else is setting up for the
            // next frame
            draw_circle(*x, *y, *circle_size, *color);
            let jiggle_x:f32 = get_random_value(-(jiggle), jiggle);
            let jiggle_y:f32 = get_random_value(-(jiggle), jiggle);
            let mut new_x = *x;
            let mut new_y = *y;
            let mut new_velocity = velocity.clone();

            if gravity_enabled {
                new_velocity.y += 98.1;
            }

            let mut new_pos = vec2(new_x, new_y);
            new_pos += *velocity * delta_time;
            if velocity.x != 0.0 || velocity.y != 0.0 {
                new_velocity -= velocity.normalize() * delta_time * medium_viscosity;
            }

            if new_velocity.x < 0.01 && new_velocity.x > -0.01 {
                new_velocity.x = 0.0;
            }
            if new_velocity.y < 0.01 && new_velocity.y > -0.01 {
                new_velocity.y = 0.0;
            }
                

            new_x = new_pos.x;
            new_y = new_pos.y;


            // collision detection
            for other in &circles[..] {
                if other == circ {
                    continue;
                }
                // sqrt (pow(abs(other_x - x), 2) + pow(abs(other_y - y), 2))
                let (other_x, other_y, other_size, _, _) = other;
                let dist = vec2(*other_x, *other_y).distance_squared(vec2(*x, *y));
                if dist < (*circle_size + other_size)*(*circle_size + other_size) {
                    let x_dist = *other_x - *x;
                    let y_dist = *other_y - *y;
                    new_velocity -= vec2(x_dist, y_dist).normalize() * dist.sqrt() * delta_time * particle_repel_force;
                }
            }

            // disable the mouse interaction while the gui is on screen
            if !show_gui {
                let mut mouse_gravity = 0.0;
                let mut mouse_distance = mouse_attract_distance;
                if is_mouse_button_down(MouseButton::Left) {
                    mouse_gravity = -mouse_attract_force;
                    mouse_distance *= 3.0;
                } else if is_mouse_button_down(MouseButton::Right) {
                    mouse_gravity = mouse_repel_force;
                }
                let mouse_x_dist = *x - mouse_x;
                let mouse_y_dist = *y - mouse_y;
                let mouse_dist = ((mouse_x_dist.powi(2) + mouse_y_dist.powi(2)) as f32).sqrt();
                if mouse_dist < mouse_distance {
                    new_velocity += Vec2::new(mouse_x_dist as f32, mouse_y_dist as f32) * mouse_gravity;
                }
            }

            new_x += jiggle_x;
            if new_x >= width - *circle_size || new_x <= *circle_size {
                new_velocity.x = -(new_velocity.x / 2.0);
            }

            new_y += jiggle_y;
            if new_y >= height - *circle_size || new_y <= *circle_size {
                new_velocity.y = -(new_velocity.y / 2.0);
            }
                
            (new_x.clamp(*circle_size, width - *circle_size), 
             new_y.clamp(*circle_size, height - *circle_size),
             *circle_size,
             *color,
             new_velocity)
        }).collect();

        draw_circle(mouse_x, mouse_y, 30.0, BLUE);


        // -- gui layer
        draw_rectangle(0.0, 0.0, 200.0, 80.0, Color::from_rgba(0x00, 0xFF, 0xFF, 0xA0));
        let mut s = String::new();
        write!(s, "Jiggle: {jiggle}").unwrap();
        draw_text_ex(s.as_str(), 0.0, 32.0, hud_textparams.clone());
        s.clear();
        let fps = get_fps();
        s.clear();
        write!(s, "FPS: {fps}").unwrap();
        draw_text_ex(s.as_str(), 0.0, 64.0, hud_textparams.clone());

        if show_gui {
            Window::new(hash!(), vec2(width - 620., 20.), vec2(420., 200.))
                .label("Controls")
                .close_button(false)
                .ui(&mut root_ui(), |ui| {
                    ui.slider(
                        hash!(),
                        "Jiggle",
                        0.0 .. 100.0,
                        &mut jiggle,
                    );
                    ui.slider(
                        hash!(),
                        "push force",
                        1.0 .. 15.0,
                        &mut mouse_repel_force,
                    );
                    ui.slider(
                        hash!(),
                        "pull force",
                        0.05 .. 1.0,
                        &mut mouse_attract_force,
                    );
                    ui.slider(
                        hash!(),
                        "pull dist.",
                        1.0 .. 500.0,
                        &mut mouse_attract_distance,
                    );
                    ui.slider(
                        hash!(),
                        "drag coef.",
                        0.0 .. 250.0,
                        &mut medium_viscosity,
                    );
                    ui.slider(
                        hash!(),
                        "ball count",
                        1.0 .. 3000.0,
                        &mut num_circles_ui,
                    );
                    ui.slider(
                        hash!(),
                        "ball repel",
                        1.0 .. 100.0,
                        &mut particle_repel_force,
                    );
                    ui.checkbox(
                        hash!(),
                        "gravity",
                        &mut gravity_enabled,
                    );
                });
        }

        next_frame().await;
    }
}
