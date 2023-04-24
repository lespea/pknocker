use anyhow::{anyhow, Error, Result};
use clap::Parser;
use std::net::IpAddr;
use std::num::{NonZeroU16, NonZeroU8};
use std::str::FromStr;

#[derive(Parser, Clone, Debug, Eq, PartialEq)]
#[clap(author, version, about)]
pub(crate) struct Opts {
    ip: IpAddr,

    #[clap(use_value_delimiter = true, required = true, value_parser = parse_target)]
    targets: Vec<Target>,
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub(crate) enum Target {
    Tcp { port: NonZeroU16 },
    Udp { port: NonZeroU16 },
    Sleep { secs: NonZeroU8 },
}

impl TryFrom<&str> for Target {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        parse_target(value)
    }
}

fn parse_target(value: &str) -> Result<Target> {
    if value.is_empty() {
        return Err(Error::msg("Empty target"));
    }

    let (first, rest) = value.split_at(1);
    match first.chars().next().unwrap() {
        't' | 'T' => Ok(Target::Tcp {
            port: NonZeroU16::from_str(rest)?,
        }),
        'u' | 'U' => Ok(Target::Udp {
            port: NonZeroU16::from_str(rest)?,
        }),
        's' | 'S' => Ok(Target::Sleep {
            secs: NonZeroU8::from_str(rest)?,
        }),

        '0'..='9' => Ok(Target::Tcp {
            port: NonZeroU16::from_str(value)?,
        }),

        _ => Err(anyhow!("Unknown target: '{value}'")),
    }
}
