#[derive(Debug, Clone)]
pub struct StartupError {
    pub stage: String,
    pub code: String,
    pub message: String,
    pub suggestion: String,
}

/// Temporary startup probe before real libmpv bootstrap lands.
/// Set VPLAYER_FORCE_STARTUP_FATAL=1 to simulate a startup fatal branch.
pub fn startup_probe() -> Result<(), StartupError> {
    let forced = std::env::var("VPLAYER_FORCE_STARTUP_FATAL")
        .ok()
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if forced {
        return Err(StartupError {
            stage: "libmpv_init".to_string(),
            code: "MPV_INIT_FAILED".to_string(),
            message: "Startup probe failed to initialize libmpv runtime.".to_string(),
            suggestion: "Disable VPLAYER_FORCE_STARTUP_FATAL or check runtime dependencies."
                .to_string(),
        });
    }

    Ok(())
}
