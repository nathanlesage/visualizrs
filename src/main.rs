#![warn(clippy::all)]
// #![deny(missing_docs)] <-- Useful if you wanna make sure to document everything

// Note, external crates are found by Cargo, not by "extern crate"

// And the graphics the OpenGL version to use as backend
use opengl_graphics::OpenGL;

// Change this to OpenGL::V2_1 if not working.
const OPENGL_VERSION: glutin_window::OpenGL = OpenGL::V3_2;

// Import the objects we need
mod application;

mod renderer;
mod user_interface;
mod traits;

mod audio;
use application::App;

fn main() {
    // Create a new game and run it.
    let mut app = App::boot(OPENGL_VERSION);

    // Let the infinity begin!
    app.main_loop();
}
