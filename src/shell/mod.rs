#[cfg(not(all(target_os = "wasi", target_env = "p2")))]
mod native;
/// Dummy type for the native target
#[cfg(not(all(target_os = "wasi", target_env = "p2")))]
pub struct Runner;

#[cfg(all(target_os = "wasi", target_env = "p2"))]
mod wasm;
#[cfg(all(target_os = "wasi", target_env = "p2"))]
pub use wasm::Runner;

#[cfg(all(target_os = "wasi", target_env = "p2"))]
pub fn run_command(command: &str, args: &[String], runner: Option<&Runner>) -> anyhow::Result<()> {
    wasm::run_command(command, args, runner)?;
    Ok(())
}

#[cfg(not(all(target_os = "wasi", target_env = "p2")))]
pub fn run_command(command: &str, args: &[String], _runner: Option<&Runner>) -> anyhow::Result<()> {
    native::run_command(command, args)?;
    Ok(())
}
