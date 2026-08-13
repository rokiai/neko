use tracing_subscriber::EnvFilter;

pub fn init_logging() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| "neko=info,tauri=warn".into()),
        )
        .try_init();
}
