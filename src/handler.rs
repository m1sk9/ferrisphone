use serenity::{
    all::{ActivityData, Context, EventHandler, Ready},
    async_trait,
};

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        let version = format!("v{}", env!("CARGO_PKG_VERSION"));
        ctx.set_activity(ActivityData::custom(format!("Running {}", version)).into());
        tracing::info!("Running {}, {} is connected!", version, ready.user.name);
    }
}
