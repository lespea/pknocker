mod opts;

use crate::opts::Target;
use anyhow::Result;
use clap::Parser;
use env_logger::Env;
use std::net::{SocketAddr, ToSocketAddrs};

fn main() -> Result<()> {
    // Setup logging
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // Parse the args
    let cli = opts::Opts::parse();
    let mut dst: SocketAddr = format!("{}:0", cli.dst)
        .to_socket_addrs()
        .expect("Bad dst")
        .next()
        .expect("No dst resolved");

    // If there is no global sleep wanted, don't bother with the logic
    let do_sleep = cli.global_sleep > 0;

    // Sleep to do
    let gsleep = Target::Sleep {
        secs: cli.global_sleep,
    };

    // If the current round should possibly insert a sleep statement.  Start
    // with true so we don't do an initial sleep
    let mut skip_sleep = true;

    for action in cli.targets.iter() {
        if do_sleep {
            let is_sleep = action.is_sleep();

            // Only insert a sleep if the last action wasn't a sleep (or start)
            // and the current action isn't sleep
            if is_sleep && !skip_sleep {
                gsleep.run(&mut dst)
            }

            // Don't run sleep in the next round if we're currently doing a sleep
            skip_sleep = !is_sleep;
        }

        // Run the current action
        action.run(&mut dst)
    }

    Ok(())
}
