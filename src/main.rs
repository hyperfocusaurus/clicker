#![windows_subsystem = "windows"]
mod quadtree;
use crate::quadtree::Quadtree;
use futures::executor;
use hound::WavSpec;
use macroquad::audio::{load_sound_from_bytes, play_sound_once};
use macroquad::color::hsl_to_rgb;
use macroquad::hash;
use macroquad::prelude::*;
use macroquad::ui::root_ui;
use macroquad::ui::widgets::Window;
use miniquad::window::screen_size;
use quad_rand::{rand, srand, RandomRange};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Write;
use std::fs;
use std::io::{BufWriter, Cursor};
use toml::Value;

const DEFAULT_WIDTH: f32 = 1920.0;
const DEFAULT_HEIGHT: f32 = 1080.0;

macro_rules! toast_hash {
    ($($x:expr),*) => {{
        (rand() as u64) << 32 | rand() as u64
    }};
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Circle(f32, f32, f32, Color, Vec2);

trait RoundToDigits {
    fn round_to_digits(self, digits: u32) -> Self;
}

impl RoundToDigits for f32 {
    fn round_to_digits(self, digits: u32) -> f32 {
        let factor = 10.0_f32.powi(digits as i32);
        (self * factor).round() / factor
    }
}

fn get_random_value<T: RandomRange>(min: T, max: T) -> T {
    T::gen_range(min, max)
}

fn bounding_box(width: f32, height: f32) -> Rect {
    Rect::new(0.0, 0.0, width, height)
}

fn beep(beep_note: f32, beep_duration: f32) {
    let beep_freq = 440.0 * 2.0f32.powf(beep_note / 12.0);
    let mut sound = gen_sine_wave(beep_freq, 44100, beep_duration);
    attenuate(&mut sound, 0.05);
    // convert the raw sound data into a .wav equivalent series of bytes for macroquad to play
    let sound_wave = raw_to_wave(&sound, 1);
    let sound = executor::block_on(load_sound_from_bytes(sound_wave.as_slice())).unwrap();
    play_sound_once(&sound);
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

fn gen_circle(width: f32, height: f32, min_size: f32, max_size: f32) -> Circle {
    let x = get_random_value(0.0, width);
    let y = get_random_value(0.0, height);
    let mut h = get_random_value(0.0, 100.0);
    h /= 100.0;
    let color = hsl_to_rgb(h, 0.5, 0.5);
    let circle_size = get_random_value(min_size, max_size);
    let velocity = Vec2::new(0.0, 0.0);
    Circle(x, y, circle_size, color, velocity)
}

fn reset_circles(
    circles: &mut Vec<Box<Circle>>,
    num_circles: u32,
    width: f32,
    height: f32,
    min_size: f32,
    max_size: f32,
) {
    circles.clear();
    for _i in 0..num_circles {
        let circle = Box::new(gen_circle(width, height, min_size, max_size));
        circles.push(circle);
    }
}

fn raw_to_wave(raw: &[f32], channels: u16) -> Vec<u8> {
    let spec = WavSpec {
        channels,
        sample_rate: 44100,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut buf = Vec::new();
    let mut writer = hound::WavWriter::new(BufWriter::new(Cursor::new(&mut buf)), spec).unwrap();
    for sample in raw {
        writer.write_sample(*sample).unwrap();
    }
    writer.finalize().unwrap();
    buf
}

fn attenuate(sound: &mut [f32], attenuation: f32) {
    for sample in sound {
        *sample *= attenuation;
    }
}

// generate a sine wave for playback through macroquad's audio system
fn gen_sine_wave(freq: f32, sample_rate: u32, duration: f32) -> Vec<f32> {
    let mut samples = Vec::new();
    let mut t = 0.0;
    while t < duration {
        let sample = (t * freq * 2.0 * std::f32::consts::PI).sin();
        samples.push(sample);
        t += 1.0 / sample_rate as f32;
    }
    samples
}

#[derive(Deserialize, Serialize, Clone)]
struct JiggleBallsConfig {
    min_circle_size: f32,
    max_circle_size: f32,
    audio_enabled: bool,
    is_fullscreen: bool,
    jiggle: f32,
    mouse_repel_force: f32,
    mouse_attract_force: f32,
    mouse_attract_distance: f32,
    medium_viscosity: f32,
    num_circles: u32,
    num_circles_ui: f32,
    gravity_enabled: bool,
    particle_repel_force: f32,
    allow_ball_intersection: bool,
    draw_velocities: bool,
    boids: bool,
    boids_box_size: f32,
    separation_distance: f32,
    separation_weight: f32,
    alignment_weight: f32,
    cohesion_weight: f32,
    avoid_walls_weight: f32,
    avoid_walls_distance: f32,
    boid_amount: f32,
    max_velocity: f32,
    starfield: bool,
    warp_factor: f32,
    monochrome: bool,
}

fn load_config (config: &mut JiggleBallsConfig) {
    match fs::read_to_string("config.toml") {
        Ok(config_str) => {
            let loaded_config: HashMap<String, Value> = toml::from_str(config_str.as_str())
                .map_err(|err| {
                    println!("Could not read config file: {}", err);
                })
                .unwrap();
            for (key, value) in loaded_config {
                match key.as_str() {
                    "min_circle_size" => {
                        config.min_circle_size = value.try_into().unwrap();
                    }
                    "max_circle_size" => {
                        config.max_circle_size = value.try_into().unwrap();
                    }
                    "audio_enabled" => {
                        config.audio_enabled = value.as_bool().unwrap();
                    }
                    "is_fullscreen" => {
                        config.is_fullscreen = value.as_bool().unwrap();
                    }
                    "jiggle" => {
                        config.jiggle = value.try_into().unwrap();
                    }
                    "mouse_repel_force" => {
                        config.mouse_repel_force = value.try_into().unwrap();
                    }
                    "mouse_attract_force" => {
                        config.mouse_attract_force = value.try_into().unwrap();
                    }
                    "mouse_attract_distance" => {
                        config.mouse_attract_distance = value.try_into().unwrap();
                    }
                    "medium_viscosity" => {
                        config.medium_viscosity = value.try_into().unwrap();
                    }
                    "num_circles" => {
                        config.num_circles = value.as_integer().unwrap() as u32;
                    }
                    "num_circles_ui" => {
                        config.num_circles_ui = value.try_into().unwrap();
                    }
                    "gravity_enabled" => {
                        config.gravity_enabled = value.as_bool().unwrap();
                    }
                    "particle_repel_force" => {
                        config.particle_repel_force = value.try_into().unwrap();
                    }
                    "allow_ball_intersection" => {
                        config.allow_ball_intersection = value.as_bool().unwrap();
                    }
                    "draw_velocities" => {
                        config.draw_velocities = value.as_bool().unwrap();
                    }
                    "boids" => {
                        config.boids = value.as_bool().unwrap();
                    }
                    "boids_box_size" => {
                        config.boids_box_size = value.try_into().unwrap();
                    }
                    "separation_distance" => {
                        config.separation_distance = value.try_into().unwrap();
                    }
                    "separation_weight" => {
                        config.separation_weight = value.try_into().unwrap();
                    }
                    "alignment_weight" => {
                        config.alignment_weight = value.try_into().unwrap();
                    }
                    "cohesion_weight" => {
                        config.cohesion_weight = value.try_into().unwrap();
                    }
                    "avoid_walls_weight" => {
                        config.avoid_walls_weight = value.try_into().unwrap();
                    }
                    "avoid_walls_distance" => {
                        config.avoid_walls_distance = value.try_into().unwrap();
                    }
                    "boid_amount" => {
                        config.boid_amount = value.try_into().unwrap();
                    }
                    "max_velocity" => {
                        config.max_velocity = value.try_into().unwrap();
                    }
                    "starfield" => {
                        config.starfield = value.as_bool().unwrap();
                    }
                    "warp_factor" => {
                        config.warp_factor = value.try_into().unwrap();
                    }
                    "monochrome" => {
                        config.monochrome = value.as_bool().unwrap();
                    }
                    _ => {
                        println!("Unknown config key: {}", key);
                    }
                }
            }
        }
        Err(err) => {
            println!(
                "No config file found, using default values (error was: {})",
                err
            );
        }
    }
}

#[macroquad::main(conf)]
async fn main() {
    let mut width: f32 = DEFAULT_WIDTH;
    let mut height: f32 = DEFAULT_HEIGHT;
    let mut show_gui = false;
    let mut show_debug_gui = false;
    let mut mouse_last_frame = vec2(0.0, 0.0);

    let mut circles = Vec::new();
    let mut circles_quadtree = Quadtree::new(Rect::new(0.0, 0.0, width, height));

    // the default values for all the configurable stuff
    let mut config = JiggleBallsConfig {
        min_circle_size: 5.0,
        max_circle_size: 50.0,
        audio_enabled: true,
        is_fullscreen: false,
        jiggle: 3.0,
        mouse_repel_force: 2.0,
        mouse_attract_force: 0.15,
        mouse_attract_distance: 100.0,
        medium_viscosity: 100.0,
        num_circles: 1000,
        num_circles_ui: 1000.0,
        gravity_enabled: false,
        particle_repel_force: 30.0,
        allow_ball_intersection: false,
        draw_velocities: false,
        boids: false,
        boids_box_size: 10.0,
        separation_distance: 10.0,
        separation_weight: 1.0,
        alignment_weight: 1.0,
        cohesion_weight: 1.0,
        avoid_walls_weight: 0.01,
        avoid_walls_distance: 100.0,
        boid_amount: 10.0,
        max_velocity: 50.0,
        starfield: false,
        warp_factor: 1.0,
        monochrome: false,
    };

    load_config(&mut config);

    let ui_font = load_ttf_font("OfficeCodePro-Regular.ttf")
        .await
        .expect("Could not load UI font");

    srand(get_time() as u64);

    set_fullscreen(config.is_fullscreen);
    if config.is_fullscreen {
        (width, height) = screen_size();
        request_new_screen_size(width, height);
    }
    reset_circles(
        &mut circles,
        config.num_circles,
        width,
        height,
        config.min_circle_size,
        config.max_circle_size,
    );
    circles_quadtree.clear(bounding_box(width, height));
    for circ in circles.clone() {
        circles_quadtree.insert(circ.clone());
    }
    let hud_textparams = TextParams {
        font: Some(&ui_font),
        font_size: 32,
        color: BLUE,
        ..Default::default()
    };

    let mut toast_messages: HashMap<u64, (u32, String)> = HashMap::new();

    let mut starfield_origin = vec2(width / 2.0, height / 2.0);

    loop {
        let delta_time = get_frame_time();
        config.num_circles_ui = config.num_circles_ui.floor();
        config.num_circles = config.num_circles_ui as u32;
        if config.min_circle_size > config.max_circle_size {
            config.min_circle_size = config.max_circle_size;
        }
        if config.max_circle_size < config.min_circle_size {
            config.max_circle_size = config.min_circle_size;
        }
        config.min_circle_size = config.min_circle_size.round_to_digits(1);
        config.max_circle_size = config.max_circle_size.round_to_digits(1);

        if circles.len() < config.num_circles.try_into().unwrap() {
            for _ in 1..config.num_circles - circles.len() as u32 {
                let circle = Box::new(gen_circle(
                    width,
                    height,
                    config.min_circle_size,
                    config.max_circle_size,
                ));
                circles_quadtree.insert(circle.clone());
                circles.push(circle);
            }
        } else if circles.len() > config.num_circles.try_into().unwrap() {
            circles.drain((config.num_circles as usize)..);
        }

        // truncate some of the floats that deal with pixel values so they're more realistic
        config.jiggle = config.jiggle.trunc();
        config.mouse_attract_distance = config.mouse_attract_distance.trunc();

        (width, height) = screen_size();
        if !show_gui {
            show_mouse(false);
        } else {
            show_mouse(true);
        }

        if is_key_pressed(KeyCode::Minus) {
            config.jiggle -= 1.0;
        }

        if is_key_pressed(KeyCode::Equal) {
            config.jiggle += 1.0;
        }

        if is_key_pressed(KeyCode::Q) {
            break;
        }

        if is_key_pressed(KeyCode::F) {
            config.is_fullscreen = !config.is_fullscreen;
            set_fullscreen(config.is_fullscreen);
            // maybe not needed?
            request_new_screen_size(width, height);
        }

        if is_key_pressed(KeyCode::S) {
            let config_str = toml::to_string(&config)
                .map_err(|err| {
                    println!("Could not serialize config: {}", err);
                })
                .unwrap();
            let _ = fs::write("config.toml", config_str.as_str()).map_err(|err| {
                println!("Could not save config: {}", err);
            });
            toast_messages.insert(hash!(), (60, "Saved config".to_string()));
        }

        if is_key_pressed(KeyCode::L) {
            load_config(&mut config);
            toast_messages.insert(hash!(), (60, "Loaded config".to_string()));
        }

        if is_key_pressed(KeyCode::G) {
            show_gui = !show_gui;
        }

        if is_key_pressed(KeyCode::D) {
            show_debug_gui = !show_debug_gui;
        }

        if is_key_pressed(KeyCode::R) {
            reset_circles(
                &mut circles,
                config.num_circles,
                width,
                height,
                config.min_circle_size,
                config.max_circle_size,
            );
            circles_quadtree.clear(bounding_box(width, height));
            for circ in circles.clone() {
                circles_quadtree.insert(circ.clone());
            }
            starfield_origin = vec2(width / 2.0, height / 2.0);
            toast_messages.insert(toast_hash!(), (60, "Reset".to_string()));
        }

        clear_background(Color::from_rgba(0x00, 0x00, 0x00, 0xC0));
        let (mouse_x, mouse_y) = mouse_position();

        // mouse control for starfield
        if config.starfield {
            // when the user drags the mouse, move the starfield in the opposite direction,
            // by moving the starfield origin point
            if is_mouse_button_down(MouseButton::Left) {
                let mouse_x_dist = mouse_x - mouse_last_frame.x;
                let mouse_y_dist = mouse_y - mouse_last_frame.y;
                starfield_origin -= vec2(mouse_x_dist, mouse_y_dist) * delta_time * 100.0;
            }
        }

        circles = circles
            .iter()
            .map(|circ| {
                let Circle(mut x, mut y, circle_size, color, velocity) = **circ;
                // draw the circle before doing anything else - everything else is setting up for the
                // next frame
                if !config.allow_ball_intersection {
                    let query_range = Rect::new(x - 50.0, y - 50.0, 100.0, 100.0);
                    let results = circles_quadtree.query(query_range);
                    for other in results {
                        if other == *circ {
                            continue;
                        }
                        // sqrt (pow(abs(other_x - x), 2) + pow(abs(other_y - y), 2))
                        let Circle(other_x, other_y, other_size, _, _) = *other;
                        let dist = vec2(other_x, other_y).distance(vec2(x, y));
                        if dist < (circle_size + other_size) {
                            let x_dist = other_x - x;
                            let y_dist = other_y - y;
                            x -= x_dist / 2.0;
                            y -= y_dist / 2.0;
                        }
                    }
                }
                if config.monochrome {
                    draw_circle(x, y, circle_size, WHITE);
                } else {
                    draw_circle(x, y, circle_size, color);
                }

                if config.draw_velocities {
                    let Vec2 {
                        x: next_x,
                        y: next_y,
                    } = vec2(x, y) + velocity;
                    let mut color = RED;
                    if config.monochrome {
                        color = WHITE;
                    }
                    draw_line(x, y, next_x, next_y, 1.0, color);
                }


                let jiggle_x: f32 = get_random_value(-(config.jiggle), config.jiggle);
                let jiggle_y: f32 = get_random_value(-(config.jiggle), config.jiggle);
                let mut new_x = x;
                let mut new_y = y;
                let mut new_velocity = velocity.clone();

                if config.gravity_enabled {
                    new_velocity.y += 9.81;
                }

                let mut new_pos = vec2(new_x, new_y);
                new_pos += velocity * delta_time;
                if !config.starfield && velocity.x != 0.0 || velocity.y != 0.0 {
                    new_velocity -= velocity.normalize() * delta_time * config.medium_viscosity;
                }

                if new_velocity.x < 0.01 && new_velocity.x > -0.01 {
                    new_velocity.x = 0.0;
                }
                if new_velocity.y < 0.01 && new_velocity.y > -0.01 {
                    new_velocity.y = 0.0;
                }

                // if starfield is enabled, move away from the center of the screen
                if config.starfield {
                    let from_center = vec2(x, y) - starfield_origin;
                    let dist = from_center.length();
                    if dist < circle_size {
                        // pick a random direction to move in if you're really close to the center
                        let mut rand_dir = vec2(0.0, 0.0);
                        while rand_dir.x == 0.0 && rand_dir.y == 0.0 {
                            rand_dir = vec2(
                                get_random_value(-1.0, 1.0),
                                get_random_value(-1.0, 1.0),
                            );
                        }
                        new_velocity = rand_dir * delta_time * 10_f32.powf(config.warp_factor / 2.0);
                    } else {
                        // move away from the center
                        new_velocity = from_center * delta_time * 10_f32.powf(config.warp_factor / 2.0);
                    }
                }

                if config.boids {
                    let query_range = Rect::new(
                        x - (config.boids_box_size / 2.0),
                        y - (config.boids_box_size / 2.0),
                        config.boids_box_size,
                        config.boids_box_size,
                    );
                    let results = circles_quadtree.query(query_range);
                    if results.len() > 0 {
                        let mut separation = Vec2::default();
                        let mut alignment = Vec2::default();
                        let mut cohesion = Vec2::default();

                        for other in &results {
                            if *other == *circ {
                                continue; // Skip self
                            }

                            let Circle(other_x, other_y, _, _, other_velocity) = **other;
                            let to_other = vec2(other_x, other_y) - vec2(x, y);

                            // Separation: Move away from close neighbors
                            if to_other.length() < config.separation_distance {
                                separation -= to_other.normalize();
                            }

                            // Alignment: Align with the average velocity of neighbors
                            alignment += other_velocity;

                            // Cohesion: Move towards the average position of neighbors
                            cohesion += vec2(other_x, other_y);
                        }

                        if results.len() > 1 {
                            // turn alignment and cohesion into mean averages of position/velocity
                            alignment /= (results.len() - 1) as f32;
                            cohesion /= (results.len() - 1) as f32;
                            separation /= (results.len() - 1) as f32;
                        }
                        let mut avoid_walls = vec2(0.0, 0.0);
                        if x < config.avoid_walls_distance
                            || x > width - config.avoid_walls_distance
                            || y < config.avoid_walls_distance
                            || y > height - config.avoid_walls_distance
                        {
                            avoid_walls = vec2(width / 2.0, height / 2.0) - vec2(x, y);
                            avoid_walls *= config.avoid_walls_weight;
                        }
                        separation *= config.separation_weight;
                        alignment *= config.alignment_weight;
                        cohesion = (cohesion - vec2(x, y)).normalize() * config.cohesion_weight;

                        new_velocity += (separation + alignment + cohesion + avoid_walls)
                            * delta_time
                            * config.boid_amount;
                    }
                }

                // if the magnitude of the velocity > max_velocity, clamp it back to that
                if !config.starfield && new_velocity.length() > config.max_velocity {
                    new_velocity = new_velocity.normalize() * config.max_velocity;
                }

                new_x = new_pos.x;
                new_y = new_pos.y;

                // collision detection
                // todo: figure out good values for the search field
                if !config.starfield {
                    let query_range = Rect::new(x - 10.0, y - 10.0, 20.0, 20.0);
                    let results = circles_quadtree.query(query_range);
                    for other in results {
                        if other == *circ {
                            continue;
                        }
                        // sqrt (pow(abs(other_x - x), 2) + pow(abs(other_y - y), 2))
                        let Circle(other_x, other_y, other_size, _, _) = *other;
                        let dist = vec2(other_x, other_y).distance(vec2(x, y));
                        if dist < (circle_size + other_size) {
                            let x_dist = other_x - x;
                            let y_dist = other_y - y;
                            new_velocity -= vec2(x_dist, y_dist).normalize()
                                * dist
                                * delta_time
                                * config.particle_repel_force;
                        }
                    }
                }

                // disable the mouse interaction while the gui is on screen
                if !show_gui && !config.starfield {
                    let mut mouse_gravity = 0.0;
                    let mut mouse_distance = config.mouse_attract_distance;
                    if is_mouse_button_down(MouseButton::Left) {
                        mouse_gravity = -config.mouse_attract_force;
                        mouse_distance *= 3.0;
                    } else if is_mouse_button_down(MouseButton::Right) {
                        mouse_gravity = config.mouse_repel_force;
                    }
                    let mouse_x_dist = x - mouse_x;
                    let mouse_y_dist = y - mouse_y;
                    let mouse_dist = ((mouse_x_dist.powi(2) + mouse_y_dist.powi(2)) as f32).sqrt();
                    if mouse_dist < mouse_distance {
                        new_velocity +=
                            Vec2::new(mouse_x_dist as f32, mouse_y_dist as f32) * mouse_gravity;
                    }
                }

                new_velocity.x += jiggle_x;
                if new_x >= width - circle_size || new_x <= circle_size {
                    if config.audio_enabled {
                        let beep_tone = 100.0 / (circle_size);
                        beep(beep_tone, 0.05);
                    }
                    new_velocity.x = -(new_velocity.x / 2.0);
                }

                new_velocity.y += jiggle_y;
                if new_y >= height - circle_size || new_y <= circle_size {
                    if config.audio_enabled {
                        let beep_tone = 100.0 / (circle_size);
                        beep(beep_tone, 0.05);
                    }
                    new_velocity.y = -(new_velocity.y / 2.0);
                }

                // make the walls actively "push" circles away
                if new_y >= height - circle_size {
                    if config.starfield {
                        Vec2{x: new_x, y: new_y} = starfield_origin;
                        new_x += jiggle_x;
                        new_y += jiggle_y;
                    } else {
                        new_velocity.y -= 1.0;
                    }
                }
                if new_x >= width - circle_size {
                    if config.starfield {
                        Vec2{x: new_x, y: new_y} = starfield_origin;
                        new_x += jiggle_x;
                        new_y += jiggle_y;
                    } else {
                        new_velocity.x -= 1.0;
                    }
                }
                if new_y <= circle_size {
                    if config.starfield {
                        Vec2{x: new_x, y: new_y} = starfield_origin;
                        new_x += jiggle_x;
                        new_y += jiggle_y;
                    } else {
                        new_velocity.y += 1.0;
                    }
                }
                if new_x <= circle_size {
                    if config.starfield {
                        Vec2{x: new_x, y: new_y} = starfield_origin;
                        new_x += jiggle_x;
                        new_y += jiggle_y;
                    } else {
                        new_velocity.x += 1.0;
                    }
                }

                new_x = new_x.clamp(circle_size, width - circle_size);
                new_y = new_y.clamp(circle_size, height - circle_size);
                let new_circ = Box::new(Circle(new_x, new_y, circle_size, color, new_velocity));
                circles_quadtree.replace(circ.clone(), new_circ.clone());
                new_circ
            })
            .collect();

        circles.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());

        draw_circle(mouse_x, mouse_y, 5.0, BLUE);

        // -- gui layer
        if show_debug_gui {
            draw_rectangle(
                0.0,
                0.0,
                200.0,
                80.0,
                Color::from_rgba(0x00, 0xFF, 0xFF, 0xA0),
            );
            let mut s = String::new();
            write!(s, "Jiggle: {}", config.jiggle).unwrap();
            draw_text_ex(s.as_str(), 0.0, 32.0, hud_textparams.clone());
            s.clear();
            let fps = get_fps();
            s.clear();
            write!(s, "FPS: {fps}").unwrap();
            draw_text_ex(s.as_str(), 0.0, 64.0, hud_textparams.clone());
            let quadtree_bounds = circles_quadtree.get_bounds();
            draw_rectangle_lines(
                quadtree_bounds.x,
                quadtree_bounds.y,
                quadtree_bounds.w,
                quadtree_bounds.h,
                1.0,
                RED,
            );
        }
        // render toast messages (temporary messages that fade out after a few frames)
        let mut toast_index = 0;
        for (hash, (frame_count, msg)) in toast_messages.clone() {
            let TextDimensions {
                width: toast_text_width,
                height: toast_text_height,
                ..
            } = measure_text(msg.as_str(), Some(&ui_font), 32, 1.0);
            let toast_width = toast_text_width + 20.0;
            let toast_height = toast_text_height + 20.0;
            let toast_y = (toast_index + 1) as f32 * toast_height;
            let toast_x = 20.0;
            let toast_alpha_scale_factor = 0xFF / 60;
            let toast_alpha = (frame_count * toast_alpha_scale_factor).clamp(0, 0xFF);
            draw_rectangle(
                toast_x,
                toast_y + 10.0 - toast_height,
                toast_width,
                toast_height,
                Color::from_rgba(0x00, 0x00, 0x00, toast_alpha as u8),
            );
            draw_text_ex(
                msg.as_str(),
                toast_x,
                toast_y,
                TextParams {
                    font: Some(&ui_font),
                    font_size: 32,
                    color: Color::from_rgba(0xFF, 0xFF, 0xFF, toast_alpha as u8),
                    ..Default::default()
                },
            );
            if frame_count == 0 {
                toast_messages.remove(&hash);
            } else {
                toast_messages.insert(hash, (frame_count - 1, msg));
            }
            toast_index += 1;
        }

        if show_gui {
            let mut window_height = 400.0;
            if config.boids {
                window_height += 150.0;
            }
            Window::new(hash!(), vec2(width - 620., 20.), vec2(420., window_height))
                .label("Controls")
                .close_button(false)
                .ui(&mut root_ui(), |ui| {
                    ui.checkbox(hash!(), "audio", &mut config.audio_enabled);
                    ui.slider(hash!(), "Jiggle", 0.0..100.0, &mut config.jiggle);
                    ui.slider(
                        hash!(),
                        "min ball size",
                        1.0..50.0,
                        &mut config.min_circle_size,
                    );
                    ui.slider(
                        hash!(),
                        "max ball size",
                        1.0..100.0,
                        &mut config.max_circle_size,
                    );
                    ui.slider(
                        hash!(),
                        "push force",
                        1.0..15.0,
                        &mut config.mouse_repel_force,
                    );
                    ui.slider(
                        hash!(),
                        "pull force",
                        0.05..1.0,
                        &mut config.mouse_attract_force,
                    );
                    ui.slider(
                        hash!(),
                        "pull dist.",
                        1.0..500.0,
                        &mut config.mouse_attract_distance,
                    );
                    ui.slider(
                        hash!(),
                        "drag coef.",
                        0.0..250.0,
                        &mut config.medium_viscosity,
                    );
                    ui.slider(
                        hash!(),
                        "ball count",
                        1.0..15000.0,
                        &mut config.num_circles_ui,
                    );
                    ui.slider(
                        hash!(),
                        "ball repel",
                        1.0..100.0,
                        &mut config.particle_repel_force,
                    );
                    ui.checkbox(hash!(), "gravity", &mut config.gravity_enabled);
                    ui.checkbox(
                        hash!(),
                        "allow phasing",
                        &mut config.allow_ball_intersection,
                    );
                    ui.checkbox(hash!(), "draw vel.", &mut config.draw_velocities);
                    ui.slider(hash!(), "speed lim.", 10.0..500.0, &mut config.max_velocity);
                    if !config.starfield {
                        ui.checkbox(hash!(), "boids", &mut config.boids);
                    }
                    if config.boids {
                        ui.slider(hash!(), "box size", 1.0..50.0, &mut config.boids_box_size);
                        ui.slider(
                            hash!(),
                            "sep. amt",
                            10.0..100.0,
                            &mut config.separation_distance,
                        );
                        ui.slider(hash!(), "sep. wt.", 0.1..5.0, &mut config.separation_weight);
                        ui.slider(
                            hash!(),
                            "align. wt.",
                            0.1..5.0,
                            &mut config.alignment_weight,
                        );
                        ui.slider(hash!(), "coh. wt.", 0.1..5.0, &mut config.cohesion_weight);
                        ui.slider(hash!(), "boid wt.", 1.0..50.0, &mut config.boid_amount);
                        ui.slider(
                            hash!(),
                            "avoid wl",
                            0.01..0.5,
                            &mut config.avoid_walls_weight,
                        );
                        ui.slider(
                            hash!(),
                            "avoid wl dist.",
                            10.0..500.0,
                            &mut config.avoid_walls_distance,
                        );  
                    } else {
                        ui.checkbox(hash!(), "starfield", &mut config.starfield);
                        if config.starfield {
                            ui.slider(hash!(), "warp factor", 1.0..10.0, &mut config.warp_factor);
                            ui.checkbox(hash!(), "monochrome", &mut config.monochrome);
                        } 
                    }
                });
        }

        mouse_last_frame = vec2(mouse_x, mouse_y);

        next_frame().await;
    }
}
