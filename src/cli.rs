use clap::*;

#[derive(Debug, Parser)]
pub struct Args {
    #[clap(subcommand)]
    pub op: Op,
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbosity: u8,
}

#[derive(Debug, clap::Args)]
pub struct MonitorIdentifier {
    #[arg(short, long, group = "identifier")]
    pub buses: Vec<u8>,
    #[arg(short, long, group = "identifier")]
    pub names: Vec<String>,
}

#[derive(Debug, Subcommand)]
pub enum Op {
    #[clap(name = "set")]
    SetBrightness {
        brightness: u8,
        #[clap(flatten)]
        monitor: MonitorIdentifier,
    },
    #[clap(name = "get")]
    GetBrightness {
        #[clap(flatten)]
        monitor: MonitorIdentifier,
    },
    #[clap(name = "inc")]
    IncreaseBrightness {
        #[arg(default_value = "10")]
        amount: u8,
        #[clap(flatten)]
        monitor: MonitorIdentifier,
    },
    #[clap(name = "dec")]
    DecreaseBrightness {
        #[arg(default_value = "10")]
        amount: u8,
        #[clap(flatten)]
        monitor: MonitorIdentifier,
    },
    SetInput {
        #[clap(flatten)]
        monitor: MonitorIdentifier,
        input: crate::ddc::Input,
    },
    GetInput {
        #[clap(flatten)]
        monitor: MonitorIdentifier,
    },
    Completions {
        shell: clap_complete::Shell,
    },
}

impl ValueEnum for crate::ddc::Input {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::HDMI(1), Self::HDMI(2), Self::DP(1), Self::DP(2)]
    }

    fn to_possible_value(&self) -> Option<builder::PossibleValue> {
        Some(match self {
            Self::HDMI(1) => builder::PossibleValue::new("HDMI-1"),
            Self::HDMI(2) => builder::PossibleValue::new("HDMI-2"),
            Self::DP(1) => builder::PossibleValue::new("DP-1"),
            Self::DP(2) => builder::PossibleValue::new("DP-2"),
            _ => return None,
        })
    }
}

pub(crate) fn completions(shell: clap_complete::Shell) {
    let mut command = Args::command();
    clap_complete::aot::generate(
        shell,
        &mut command,
        env!("CARGO_BIN_NAME"),
        &mut std::io::stdout(),
    );
}
