use tracing_subscriber::prelude::*;

pub(crate) fn init() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let layer_filter = tracing_subscriber::EnvFilter::from_default_env();
    // .add_directive("debug".parse()?)
    // .add_directive("autonix=trace".parse()?);

    let layer_fmt = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr)
        // .without_time()
        .with_line_number(true)
        .compact();

    tracing_subscriber::registry()
        .with(layer_filter)
        .with(layer_fmt)
        .init();

    Ok(())
}
