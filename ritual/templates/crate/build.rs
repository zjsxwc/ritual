//! Default build script generated by `ritual`.
//! See the template at `ritual/templates/crate/build.rs`.
//!
//! You can replace this with a custom build script by placing `build.rs` file in
//! the crate template and using `Config::set_crate_template_path` to specify the template.
//! However, make sure to call `ritual_build::run()` or
//! use the other `ritual_build` API in the custom build script to
//! perform the necessary build steps.

fn main() {
    ritual_build::run()
}
