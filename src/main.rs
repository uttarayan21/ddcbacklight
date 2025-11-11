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

    fn filter_info(info: &DisplayInfo, identifier: &MonitorIdentifier) -> bool {
        if identifier.names.is_empty() && identifier.buses.is_empty() {
            true
        } else if !identifier.names.is_empty() {
            identifier
                .names
                .iter()
                .any(|name| info.drm().to_lowercase().contains(&name.to_lowercase()))
        } else {
            identifier
                .buses
                .iter()
                .any(|bus| info.io_path() == IOPath::I2C(*bus as i32))
        }
    }
    match cli.op {
        Op::GetBrightness { monitor } => {
            let list = DisplayList::probe(true)?;
            for dinfo in list.iter().filter(|info| filter_info(info, &monitor)) {
                tracing::info!("Found display: {}", dinfo.model());
                let display = dinfo.open()?;
                let backlight = display.backlight_get()?;
                println!(
                    "{:<15}:({:^8}) {:>3}/{:>3}",
                    dinfo.model().green(),
                    dinfo
                        .drm()
                        .split_once('-')
                        .map(|s| s.1.to_string())
                        .unwrap_or(dinfo.drm()),
                    backlight.current,
                    backlight.max
                );
            }
        }
        Op::SetBrightness {
            brightness,
            monitor,
        } => {
            let list = DisplayList::probe(true)?;
            for dinfo in list.iter().filter(|info| filter_info(info, &monitor)) {
                tracing::info!("Found display: {}", dinfo.model());
                let display = dinfo.open()?;
                display.backlight_set(brightness.into())?;
                let backlight = display.backlight_get()?;
                println!(
                    "{:<15}:({:^8}) {:>3}/{:>3}",
                    dinfo.model().green(),
                    dinfo
                        .drm()
                        .split_once('-')
                        .map(|s| s.1.to_string())
                        .unwrap_or(dinfo.drm()),
                    backlight.current,
                    backlight.max
                );
            }
        }
        Op::IncreaseBrightness { amount, monitor } => {
            let list = DisplayList::probe(true)?;
            for dinfo in list.iter().filter(|info| filter_info(info, &monitor)) {
                tracing::info!("Found display: {}", dinfo.model());
                let display = dinfo.open()?;
                let current_backlight = display.backlight_get()?;
                let new_brightness = std::cmp::min(100, current_backlight.current + amount as u16);
                display.backlight_set(new_brightness)?;
                let backlight = display.backlight_get()?;
                println!(
                    "{:<15}:({:^8}) {:>3}/{:>3}",
                    dinfo.model().green(),
                    dinfo
                        .drm()
                        .split_once('-')
                        .map(|s| s.1.to_string())
                        .unwrap_or(dinfo.drm()),
                    backlight.current,
                    backlight.max
                );
            }
        }
        Op::DecreaseBrightness { amount, monitor } => {
            let list = DisplayList::probe(true)?;
            for dinfo in list.iter().filter(|info| filter_info(info, &monitor)) {
                tracing::info!("Found display: {}", dinfo.model());
                let display = dinfo.open()?;
                let current_backlight = display.backlight_get()?;
                let new_brightness = current_backlight.current.saturating_sub(amount as u16);
                display.backlight_set(new_brightness)?;
                let backlight = display.backlight_get()?;
                println!(
                    "{:<15}:({:^8}) {:>3}/{:>3}",
                    dinfo.model().green(),
                    dinfo
                        .drm()
                        .split_once('-')
                        .map(|s| s.1.to_string())
                        .unwrap_or(dinfo.drm()),
                    backlight.current,
                    backlight.max
                );
            }
        }
        Op::GetInput { monitor } => {
            let list = DisplayList::probe(true)?;
            for dinfo in list.iter().filter(|info| filter_info(info, &monitor)) {
                tracing::info!("Found display: {} ({})", dinfo.model(), dinfo.io_path());
                let display = dinfo.open()?;
                let input = display.input()?;
                println!(
                    "{:<15}: {:?} (Connected as {}): {}",
                    dinfo.model().green(),
                    input,
                    dinfo
                        .drm()
                        .split_once('-')
                        .map(|s| s.1.to_string())
                        .unwrap_or(dinfo.drm()),
                    dinfo.io_path()
                );
            }
        }
        Op::SetInput { monitor, input } => {
            let list = DisplayList::probe(true)?;
            for dinfo in list.iter().filter(|info| filter_info(info, &monitor)) {
                tracing::info!("Found display: {} ({})", dinfo.model(), dinfo.io_path());
                let display = dinfo.open()?;
                display.set_input(input)?;
                let input = display.input()?;
                println!("{}: {:?}", dinfo.model().blue(), input);
            }
        }
        Op::Completions { shell } => {
            cli::completions(shell);
        }
    }
    Ok(())
}
