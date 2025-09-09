#[cfg(not(all(target_os = "wasi", target_env = "p2")))]
mod native;
#[cfg(all(target_os = "wasi", target_env = "p2"))]
mod wasm;

pub fn run_command(command: &str) {
    #[cfg(all(target_os = "wasi", target_env = "p2"))]
    wasm::run_command(command);
    #[cfg(not(all(target_os = "wasi", target_env = "p2")))]
    native::run_command(command);
}
