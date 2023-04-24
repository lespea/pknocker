use anyhow::{anyhow, Error, Result};
use clap::Parser;
use log::info;
use std::net::{Shutdown, SocketAddr, TcpStream, UdpSocket};
use std::num::NonZeroU16;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

#[derive(Parser, Clone, Debug, Eq, PartialEq)]
#[clap(author, version, about)]
pub struct Opts {
    /// How long to sleep between each target
    #[clap(long, default_value = "0")]
    pub global_sleep: u8,

    /// The IP to connect to
    pub dst: String,

    /// The list of target ports (or delays) to connect to
    #[clap(use_value_delimiter = true, required = true, value_parser = parse_target)]
    pub targets: Vec<Target>,
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Target {
    Tcp { port: NonZeroU16 },
    Udp { port: NonZeroU16 },
    Sleep { secs: u8 },
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
        's' | 'S' | 'p' | 'P' => Ok(Target::Sleep {
            secs: rest.parse()?,
        }),

        '0'..='9' => Ok(Target::Tcp {
            port: NonZeroU16::from_str(value)?,
        }),

        ch => Err(anyhow!("Unknown ident '{ch}' for the value '{value}'")),
    }
}

static CONN_TIMEOUT: Duration = Duration::from_secs(2);

impl Target {
    pub fn run(&self, dst: &mut SocketAddr) {
        match self {
            Self::Sleep { secs } => {
                if *secs > 0 {
                    let d = Duration::from_secs(*secs as u64);
                    info!("Sleeping for {secs} seconds");
                    sleep(d);
                }
            }

            Self::Tcp { port } => {
                dst.set_port(port.get());

                info!("TCP {dst}");
                let _ = TcpStream::connect_timeout(dst, CONN_TIMEOUT)
                    .and_then(|s| s.shutdown(Shutdown::Both));
            }

            Self::Udp { port } => {
                let baddr = if dst.is_ipv4() { "0.0.0.0:0" } else { "[::]:0" };

                let sock = UdpSocket::bind(baddr).expect("Couldn't bind local udp sock");
                sock.set_write_timeout(Some(CONN_TIMEOUT))
                    .unwrap();

                dst.set_port(port.get());
                info!("UDP {dst}");
                let _ = sock.send_to(&[], *dst);
            }
        };
    }

    #[inline]
    pub fn is_sleep(&self) -> bool {
        !matches!(self, Self::Sleep { .. })
    }
}
