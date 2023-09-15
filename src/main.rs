mod ascii;
mod cli;
mod ddc;
mod error;
use colored::Colorize;
use ddc::*;
use error::*;
use tracing_subscriber::prelude::*;

use crate::cli::*;

fn main() -> Result<()> {
    use clap::Parser;
    let cli = cli::Args::parse();
    if cli.verbosity > 0 {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_target(false)
                    .with_timer(tracing_subscriber::fmt::time::uptime())
                    .with_level(true)
                    .with_filter(tracing_subscriber::filter::LevelFilter::from(
                        match cli.verbosity {
                            0 => tracing::Level::INFO,
                            1 => tracing::Level::DEBUG,
                            _ => tracing::Level::TRACE,
                        },
                    )),
            )
            .init();
    }

    match cli.op {
        Op::Get => {
            let list = DisplayList::probe(true)?;
            for dinfo in list.iter() {
                tracing::info!("Found display: {}", dinfo.model());
                let display = dinfo.open()?;
                let backlight = display.backlight_get()?;
                println!(
                    "{}: {}/{}",
                    dinfo.model().green(),
                    backlight.current,
                    backlight.max
                );
            }
        }
        Op::Set { brightness } => {
            let list = DisplayList::probe(true)?;
            for dinfo in list.iter() {
                tracing::info!("Found display: {}", dinfo.model());
                let display = dinfo.open()?;
                display.backlight_set(brightness.into())?;
                let backlight = display.backlight_get()?;
                println!(
                    "{}: {}/{}",
                    dinfo.model().blue(),
                    backlight.current,
                    backlight.max
                );
            }
        }
    }
    Ok(())
}
