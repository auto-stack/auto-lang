//! Explicit composition for legacy SDK HTTP tests.
pub(crate) fn install() {
    auto_web::install_http_service_host_if_absent();
}
