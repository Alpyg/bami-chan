extern crate tracing;

use std::{
    collections::HashMap,
    env,
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, Ordering},
    },
};

use dotenv::dotenv;
use process::process_interactions;
use songbird::{Songbird, shards::TwilightMap, tracks::TrackHandle};
use tracing::Level;
use twilight_cache_inmemory::{InMemoryCache, InMemoryCacheBuilder, ResourceType};
use twilight_gateway::{
    CloseFrame, ConfigBuilder, Event, EventTypeFlags, Intents, Shard, StreamExt, create_recommended,
};
use twilight_http::Client as HttpClient;
use twilight_interactions::command::CreateCommand;
use twilight_model::id::{Id, marker::GuildMarker};
use twilight_standby::Standby;

mod music;
mod ping;
mod process;
mod utils;

use music::{PauseCommand, PlayCommand, ResumeCommand, SkipCommand, StopCommand};
use ping::*;

static SHUTDOWN: AtomicBool = AtomicBool::new(false);

pub type Context = Arc<ContextRef>;

#[derive(Debug)]
pub struct ContextRef {
    pub client: Arc<HttpClient>,
    pub cache: Arc<InMemoryCache>,
    pub http: reqwest::Client,
    pub standby: Standby,
    pub songbird: Songbird,
    pub trackdata: RwLock<HashMap<Id<GuildMarker>, TrackHandle>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let token = env::var("DISCORD_TOKEN")?;

    // Initialize logging with tracing
    tracing_subscriber::fmt()
        .compact()
        .with_max_level(Level::INFO)
        .init();

    // Initialize twilight http client and gateway configuartion
    let http = Arc::new(HttpClient::new(token.clone()));
    let config = ConfigBuilder::new(token.clone(), Intents::all()).build();

    // Register global commands
    let commands = [
        PingCommand::create_command().into(),
        PlayCommand::create_command().into(),
        PauseCommand::create_command().into(),
        ResumeCommand::create_command().into(),
        SkipCommand::create_command().into(),
        StopCommand::create_command().into(),
    ];
    let application = http.current_user_application().await?.model().await?;
    let interaction_client = http.interaction(application.id);

    tracing::info!("logged as {} with ID {}", application.name, application.id);

    // TODO: Change to global later
    if let Err(error) = interaction_client
        .set_guild_commands(
            Id::<GuildMarker>::new(env::var("SERVER_ID")?.parse()?),
            &commands,
        )
        .await
    {
        tracing::error!(?error, "failed to register commands");
    }

    // Start gateway shards
    let shards: Vec<Shard> = create_recommended(&http, config, |_id, builder| builder.build())
        .await?
        .collect();
    let shards_len = shards.len();
    let mut senders = Vec::with_capacity(shards_len);
    let mut tasks = Vec::with_capacity(shards_len);

    let ctx = Arc::new(ContextRef {
        client: http.clone(),
        http: reqwest::Client::new(),
        cache: Arc::new(
            InMemoryCacheBuilder::new()
                .resource_types(ResourceType::MESSAGE | ResourceType::VOICE_STATE)
                .build(),
        ),
        standby: Standby::new(),
        songbird: Songbird::twilight(
            Arc::new(TwilightMap::new(
                shards
                    .iter()
                    .map(|s| (s.id().number(), s.sender()))
                    .collect(),
            )),
            http.current_user().await?.model().await?.id,
        ),
        trackdata: Default::default(),
    });

    for shard in shards {
        senders.push(shard.sender());
        tasks.push(tokio::spawn(runner(shard, ctx.clone())));
    }

    tokio::signal::ctrl_c().await?;
    SHUTDOWN.store(true, Ordering::Relaxed);
    for sender in senders {
        _ = sender.close(CloseFrame::NORMAL);
    }

    for jh in tasks {
        _ = jh.await;
    }

    Ok(())
}

async fn runner(mut shard: Shard, ctx: Context) {
    while let Some(event) = shard.next_event(EventTypeFlags::all()).await {
        let Ok(event) = event else {
            tracing::warn!(source = ?event.unwrap_err(), "error recceiving event");
            continue;
        };

        match event {
            Event::GatewayClose(_) if SHUTDOWN.load(Ordering::Relaxed) => break,
            _ => {}
        }

        ctx.cache.update(&event);
        ctx.songbird.process(&event).await;

        tracing::info!(kind = ?event.kind(), shard = ?shard.id().number(), "received event");
        tokio::spawn(process_interactions(event, ctx.clone()));
    }
}
