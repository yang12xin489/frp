use crate::service::watchdog_service;

mod service {
    pub mod watchdog_service;
}
fn main() {
    watchdog_service::run();
}
