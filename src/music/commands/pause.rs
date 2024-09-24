use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_mention::Mention;
use twilight_model::{
    application::interaction::{application_command::CommandData, Interaction},
    http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::Context;

#[derive(Debug, CommandModel, CreateCommand)]
#[command(name = "pause", desc = "Pause the current track.")]
pub struct PauseCommand {}

impl PauseCommand {
    pub async fn handle(
        interaction: Interaction,
        _data: CommandData,
        ctx: &Context,
    ) -> anyhow::Result<()> {
        let client = ctx.client.interaction(interaction.application_id);
        let guild_id = interaction.guild_id.unwrap();

        tracing::debug!(
            "pause command in channel {} by {}",
            interaction.channel.clone().unwrap().id,
            interaction.author().unwrap().mention()
        );

        if let Some(call_lock) = ctx.songbird.get(guild_id) {
            let call = call_lock.lock().await;
            call.queue().pause()?;
        }

        let response = InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(
                InteractionResponseDataBuilder::new()
                    .content("Pausing")
                    .build(),
            ),
        };

        client
            .create_response(interaction.id, &interaction.token, &response)
            .await?;

        // TODO: Pause

        Ok(())
    }
}
