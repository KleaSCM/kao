/**
 * Build Script for Tauri Backend.
 *
 * Configures the build-time environment and invokes tauri-build
 * to generate necessary platform-specific assets.
 *
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 */

// NOTE (KleaSCM) main function must retain snake_case as it is the
// mandatory entry point for the Rust build process.
fn main() {
	tauri_build::build()
}
