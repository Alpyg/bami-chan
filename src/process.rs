use std::mem;

use anyhow::bail;
use twilight_gateway::Event;
use twilight_model::application::interaction::{
    application_command::CommandData, Interaction, InteractionData,
};

use crate::{
    music::{PauseCommand, PlayCommand, ResumeCommand, SkipCommand, StopCommand},
    Context, PingCommand,
};

pub async fn process_interactions(event: Event, ctx: Context) {
    let mut interaction = match event {
        Event::InteractionCreate(interaction) => interaction.0,
        _ => return,
    };

    let data = match mem::take(&mut interaction.data) {
        Some(InteractionData::ApplicationCommand(data)) => *data,
        _ => {
            tracing::warn!("ignoring non-command interaction");
            return;
        }
    };

    if let Err(error) = handle_command(interaction, data, &ctx).await {
        tracing::error!(?error, "error while handling command");
    }
}

async fn handle_command(
    interaction: Interaction,
    data: CommandData,
    ctx: &Context,
) -> anyhow::Result<()> {
    match &*data.name {
        "ping" => PingCommand::handle(interaction, data, ctx).await,
        "play" => PlayCommand::handle(interaction, data, ctx).await,
        "pause" => PauseCommand::handle(interaction, data, ctx).await,
        "resume" => ResumeCommand::handle(interaction, data, ctx).await,
        "skip" => SkipCommand::handle(interaction, data, ctx).await,
        "stop" => StopCommand::handle(interaction, data, ctx).await,
        name => bail!("unknown command: {}", name),
    }
}
