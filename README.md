# Jiggle Balls

![GitHub Workflow Status (with event)](https://img.shields.io/github/actions/workflow/status/hyperfocusaurus/jiggleballs/rust.yml)
[
![GitHub Release Date - Published_At](https://img.shields.io/github/release-date/hyperfocusaurus/jiggleballs?link=https%3A%2F%2Fgithub.com%2Fhyperfocusaurus%2Fjiggleballs%2Freleases%2Flatest&link=https%3A%2F%2Fgithub.com%2Fhyperfocusaurus%2Fjiggleballs%2Freleases%2Flatest)
](https://github.com/hyperfocusaurus/jiggleballs/releases/latest)

Yes, I know, it's a bit of a silly name.  Don't think too much about it.

This is a very small, simple, fast ball pit simulator with some simple controls.

Run it by downloading a binary release, build it using `cargo build`.

Uses a bare minimum set of dependencies, macroquads being the biggest dependency.

No runtime dependencies.  100% pure Rust at the application level.

# Control Scheme

```
Left Click: Sucks balls towards the mouse cursor
Right Click: Repels balls away from the mouse cursor

d: Show debug info (FPS and current "Jiggle")
=: Increase Jiggle amount (think of it as pressing the + key, but without needing to hold shift)
-: Decrease Jiggle Amount
g: Show GUI controls
s: Save settings from GUI controls into "config.toml"
l: Load settings from "config.toml"
r: Reset the ball field
f: Toggle fullscreen (note: may not work to turn fullscreen mode *off* due to a known issue)
q: Quit
```
