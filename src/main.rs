use raylib::prelude::*;
use raylib::ffi::{MouseButton};
use raylib::core::color::{Color};

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(1920, 1080)
        .title("Hello, World")
        .build();

    let mut circles = Vec::new();
    let mut color = Color::RED;
    let mut hue: f32 = 0.0;
    const EPSILON: f32 = 0.5;
    let mut circle_size: f32 = 10.0;
     
    while !rl.window_should_close() {
        if rl.is_window_focused() {
            rl.hide_cursor();
        }
        let mut d = rl.begin_drawing(&thread);
        let x = d.get_mouse_x();
        let y = d.get_mouse_y();

        if d.is_key_pressed(KeyboardKey::KEY_EQUAL) {
            if d.is_key_down(KeyboardKey::KEY_LEFT_CONTROL) {
                circles = circles.iter().map(|circ| {
                    let (x, y, color, circle_size) = circ;
                    (*x, *y, *color, circle_size + EPSILON * 10.0)
                }).collect();
            }
            circle_size += EPSILON * 10.0;
        }

        if d.is_key_pressed(KeyboardKey::KEY_MINUS) {
            if d.is_key_down(KeyboardKey::KEY_LEFT_CONTROL) {
                circles = circles.iter().map(|circ| {
                    let (x, y, color, circle_size) = circ;
                    (*x, *y, *color, circle_size - EPSILON * 10.0)
                }).collect();
            }
            circle_size -= EPSILON * 10.0;
        }
         
        d.clear_background(Color::WHITE);
        if d.is_mouse_button_down(MouseButton::MOUSE_LEFT_BUTTON) {
            hue += EPSILON;
            if hue > 360.0 {
                hue = 0.0;
            }
            color = Color::color_from_hsv(hue, 1.0, 1.0);
            circles.push((x, y, color, circle_size));
        }
        circles.iter().for_each(|circ| {
            let (x, y, color, circle_size) = circ;
            d.draw_circle(*x, *y, *circle_size, color);
        });
        d.draw_circle(x, y, circle_size, color);
    }
}
