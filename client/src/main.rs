//! Client
#![feature(coverage_attribute)]
#![warn(rustdoc::private_doc_tests)]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
use std::{
    io::{self, stdout},
    panic,
};

use actix_settings::Settings;
use color_eyre::eyre;

use crossterm::event::EventStream;
use ratatui::backend::CrosstermBackend;
use tracing_error::ErrorLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};
mod mid;
mod term;
mod ui;

const BACKGROUND: Color = Color::Reset;
const TEXT_COLOR: Color = Color::White;
const SELECTED_STYLE_FG: Color = Color::LightYellow;
const COMPLETED_TEXT_COLOR: Color = Color::Green;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    initialize_logging()?;
    install_hooks()?;
    let term = term::init(std::io::stdout())?;
    let res = run(term).await;
    term::restore()?;
    res
}
async fn run<W: io::Write>(mut term: term::Tui<W>) -> color_eyre::Result<()> {
    let settings = Settings::parse_toml("./server/Server.toml").unwrap();
    let state = mid::init(&format!("http://{}:{}", settings.actix.hosts[0].host, settings.actix.hosts[0].port)).await?;
    let events = EventStream::new();
    App::new(state).run(&mut term, events).await
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
