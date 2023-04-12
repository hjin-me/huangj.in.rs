pub mod blog;
#[cfg(feature = "ssr")]
pub fn register_server_functions() {
    let _ = blog::register_server_functions();
}
