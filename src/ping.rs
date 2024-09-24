use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
    application::interaction::{application_command::CommandData, Interaction},
    http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::Context;

#[derive(Debug, CommandModel, CreateCommand)]
#[command(name = "ping", desc = "ping")]
pub struct PingCommand {}

impl PingCommand {
    pub async fn handle(
        interaction: Interaction,
        _data: CommandData,
        ctx: &Context,
    ) -> anyhow::Result<()> {
        let client = ctx.client.interaction(interaction.application_id);
        let response = InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(
                InteractionResponseDataBuilder::new()
                    .content("pong")
                    .build(),
            ),
        };

        client
            .create_response(interaction.id, &interaction.token, &response)
            .await?;

        Ok(())
    }
}
