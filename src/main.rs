use color_eyre::Result;
use tracing::trace;

mod setup;

fn main() -> Result<()> {
    setup::setup()?;

    trace!("Hello world!");

    Ok(())
}
