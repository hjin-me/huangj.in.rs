pub mod blog;

#[cfg(feature = "ssr")]
pub fn register_server_functions() {
    blog::register_server_functions();
}
