//! Explicit composition for legacy SDK HTTP tests.
pub(super) fn install() {
    auto_web::install_http_service_host_if_absent();
}
