#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/**
 * Application Entry Point.
 *
 * Initializes the backend library and begins the main execution loop.
 *
 * STRATEGY:
 * - Delegates execution to the tauri_app_lib library crate.
 * - Ensures correct platform-specific subsystem configuration.
 *
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 */

// NOTE (KleaSCM) main function must retain snake_case as it is the
// mandatory entry point for the Rust runtime.
fn main() {
	tauri_app_lib::Run()
}
