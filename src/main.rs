use anyhow::Ok;
use serenity::all::GatewayIntents;
use tracing::info;

use crate::{handler::Handler, store::config::Config};

mod handler;
mod store;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let config = Config::load()?;

    let subscriber = tracing_subscriber::fmt()
        .with_max_level(if config.general.debug {
            tracing::Level::TRACE
        } else {
            tracing::Level::INFO
        })
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    info!("Configuration loaded. (debug={})", config.general.debug);

    let token = std::env::var("DISCORD_API_KEY")?;
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
    let mut client = serenity::Client::builder(token, intents)
        .event_handler(Handler)
        .await?;

    client.start().await?;
    Ok(())
}
