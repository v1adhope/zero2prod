use zero2prod::config::get_config;
use zero2prod::startup::App;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let cfg = get_config().expect("failed to read config");

    let app = App::build(cfg).await?;
    app.run_until_stopped().await?;

    Ok(())
}
