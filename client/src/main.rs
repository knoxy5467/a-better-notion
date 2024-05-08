//! Client
#![feature(coverage_attribute)]
#![feature(extract_if)]
#![warn(rustdoc::private_doc_tests)]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
use std::{io::stdout, panic};

use actix_settings::{NoSettings, Settings};
use color_eyre::eyre;
use crossterm::event::EventStream;
use ratatui::backend::CrosstermBackend;
use tracing::level_filters::LevelFilter;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};
mod mid;
mod term;
mod ui;
fn load_settings() -> Result<actix_settings::BasicSettings<NoSettings>, actix_settings::Error> {
    Settings::parse_toml("Server.toml")
}

#[coverage(off)]
fn main() -> color_eyre::Result<()> {
    // manually create tokio runtime
    let rt = tokio::runtime::Runtime::new().unwrap();
    let guard = initialize_tracing()?;
    install_hooks()?;
    rt.block_on(
        #[coverage(off)]
        async {
            tracing::info!("Starting Client");
            tracing::info!("Use RUST_LOG=debug for debug logs!");
            term::enable()?;
            let settings = load_settings().expect("could not load settings");
            let state = mid::init(&format!(
                "http://{}:{}",
                settings.actix.hosts[0].host, settings.actix.hosts[0].port
            ))?;
            let res = ui::run(CrosstermBackend::new(stdout()), state, EventStream::new()).await;
            term::restore()?;
            tracing::info!("Exiting...");
            res?;
            drop(guard); // must keep track of guard so log file is written correctly
            Ok(())
        },
    )
}

#[coverage(off)]
fn initialize_tracing() -> color_eyre::Result<WorkerGuard> {
    let path = std::path::Path::new("logs/rolling.log");
    let prefix = path.parent().unwrap();
    std::fs::create_dir_all(prefix).unwrap();
    let _ = std::fs::File::create("logs/rolling.log").expect("failed to clear file"); // truncate
    let file_appender = tracing_appender::rolling::never("logs", "rolling.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let file_subscriber = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_writer(non_blocking)
        .with_target(false)
        .with_ansi(false)
        .with_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with_filter(tracing_subscriber::filter::filter_fn(|e| e.is_event())); // prevent errors being logged (those are sent to console)
    tracing_subscriber::registry()
        .with(file_subscriber)
        .with(tracing_error::ErrorLayer::default())
        .init();
    Ok(guard)
}

/// This replaces the standard color_eyre panic and error hooks with hooks that
/// restore the terminal before printing the panic or error.
#[coverage(off)]
pub fn install_hooks() -> color_eyre::Result<()> {
    // add any extra configuration you need to the hook builder
    let hook_builder = color_eyre::config::HookBuilder::default();
    let (panic_hook, eyre_hook) = hook_builder.into_hooks();

    // used color_eyre's PanicHook as the standard panic hook
    let panic_hook = panic_hook.into_panic_hook();
    panic::set_hook(Box::new(
        #[coverage(off)]
        move |panic_info| {
            term::restore().unwrap();
            panic_hook(panic_info);
        },
    ));

    // use color_eyre's EyreHook as eyre's ErrorHook
    let eyre_hook = eyre_hook.into_eyre_hook();
    eyre::set_hook(Box::new(
        #[coverage(off)]
        move |error| {
            term::restore().unwrap();
            eyre_hook(error)
        },
    ))?;

    Ok(())
}

#[cfg(test)]
mod test_main {
    use super::*;
    #[test]
    fn test_load_settings() {
        load_settings().expect("failed to load settings");
    }
}
