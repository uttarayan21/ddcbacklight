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
        Op::GetBrightnexs => {
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
        Op::SetBrightness { brightness } => {
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
        Op::GetInput { bus } => {
            let list = DisplayList::probe(true)?;
            for dinfo in list.iter().filter(|info| {
                bus.map(|bus| info.io_path() == IOPath::I2C(bus as i32))
                    .unwrap_or(true)
            }) {
                tracing::info!("Found display: {} ({})", dinfo.model(), dinfo.io_path());
                let display = dinfo.open()?;
                let input = display.input()?;
                println!(
                    "{}: {:?}: {}",
                    dinfo.model().green(),
                    input,
                    dinfo.io_path()
                );
            }
        }
        Op::SetInput { bus, input } => {
            let list = DisplayList::probe(true)?;
            if let Some(dinfo) = list
                .iter()
                .find(|info| info.io_path() == IOPath::I2C(bus as i32))
            {
                tracing::info!("Found display: {}", dinfo.model());
                let display = dinfo.open()?;
                display.set_input(input)?;
                let input = display.input()?;
                println!("{}: {:?}", dinfo.model().blue(), input);
            }
        }
    }
    Ok(())
}
