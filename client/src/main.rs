//! Client
#![feature(coverage_attribute)]
#![warn(rustdoc::private_doc_tests)]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
use std::{
    io::{self, stdout},
    panic,
};

use actix_settings::{BasicSettings, NoSettings, Settings};
use color_eyre::eyre;

use crossterm::event::EventStream;
use ratatui::backend::CrosstermBackend;
use tracing_error::ErrorLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};
mod mid;
mod term;
mod ui;

#[coverage(off)]
fn main() -> color_eyre::Result<()> {
    // manually create tokio runtime
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(
        #[coverage(off)]
        async {
            initialize_logging()?;
            install_hooks()?;
            term::enable()?;
            let state = mid::init("http://localhost:8080").await?;
            let res = ui::run(CrosstermBackend::new(stdout()), state, EventStream::new()).await;
            term::restore()?;
            res?;
            Ok(())
        },
    )
}

#[coverage(off)]
fn initialize_logging() -> color_eyre::Result<()> {
    let file_subscriber = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_writer(io::stdout)
        .with_target(false)
        .with_ansi(false)
        .with_filter(tracing_subscriber::filter::EnvFilter::from_default_env());
    tracing_subscriber::registry()
        .with(file_subscriber)
        .with(ErrorLayer::default())
        .init();
    Ok(())
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
