use std::process::Command;

use anyhow::bail;
use regex::Regex;
use songbird::{
    input::{Compose, YoutubeDl},
    Event, TrackEvent,
};
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_mention::Mention;
use twilight_model::{
    application::interaction::{application_command::CommandData, Interaction},
    http::interaction::{InteractionResponse, InteractionResponseType},
    id::{marker::UserMarker, Id},
};
use twilight_util::{
    builder::{
        embed::{EmbedBuilder, ImageSource},
        InteractionResponseDataBuilder,
    },
    snowflake::Snowflake,
};

use crate::{music::events::TrackPlayableHandler, Context};

#[derive(Debug, CommandModel, CreateCommand)]
#[command(name = "play", desc = "Add a track to the queue.")]
pub struct PlayCommand {
    #[command(desc = "url or search term")]
    pub query: String,
}

impl PlayCommand {
    pub async fn handle(
        interaction: Interaction,
        data: CommandData,
        ctx: &Context,
    ) -> anyhow::Result<()> {
        let client = ctx.client.interaction(interaction.application_id);
        let guild_id = interaction.guild_id.unwrap();
        let command = PlayCommand::from_interaction(data.into())?;

        let response = InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(
                InteractionResponseDataBuilder::new()
                    .content("Processing")
                    .build(),
            ),
        };
        client
            .create_response(interaction.id, &interaction.token, &response)
            .await?;

        // Join voice channel
        if ctx
            .cache
            .voice_state(
                Id::<UserMarker>::new(interaction.application_id.id()),
                guild_id,
            )
            .is_none()
        {
            if let Some(voice_state) = ctx
                .cache
                .voice_state(interaction.author_id().unwrap(), guild_id)
            {
                let channel_id = voice_state.channel_id();

                tracing::debug!("joining voice channel {} in guild {}", channel_id, guild_id);

                match ctx.songbird.join(guild_id, channel_id).await {
                    Ok(_) => {}
                    Err(error) => {
                        tracing::error!(?error, "join voice channel");
                        client
                            .update_response(&interaction.token)
                            .content(Some("Failed to join voice channel"))?
                            .await?;

                        bail!(error);
                    }
                }
            } else {
                tracing::error!("You are not in a voice channel {}", guild_id);

                client
                    .update_response(&interaction.token)
                    .content(Some("You are not in a voice channel"))?
                    .await?;

                bail!("You are not in a voice channel");
            }
        };

        let mut to_queue = Vec::<YoutubeDl>::new();
        if !command.query.starts_with("http") {
            to_queue.push(YoutubeDl::new_search(
                ctx.http.clone(),
                command.query.clone(),
            ));
        } else if command.query.contains("playlist") {
            let output = Command::new("yt-dlp")
                .args(["-j", "--flat-playlist", &command.query])
                .output();

            let raw_list = match output {
                Ok(list) => String::from_utf8(list.stdout).unwrap(),
                Err(e) => bail!("yt-dlp error {}", e),
            };

            let re = Regex::new(r#""url": "(https://www.youtube.com/watch\?v=[A-Za-z0-9]{11})""#)
                .unwrap();

            let urls: Vec<String> = re
                .captures_iter(&raw_list)
                .map(|cap| cap[1].to_string())
                .collect();

            for url in urls {
                to_queue.push(YoutubeDl::new(ctx.http.clone(), url));
            }
        } else {
            to_queue.push(YoutubeDl::new(ctx.http.clone(), command.query));
        }

        for src in to_queue.iter_mut() {
            if let Ok(metadata) = src.aux_metadata().await {
                if let Some(call_lock) = ctx.songbird.get(guild_id) {
                    let track;
                    {
                        let mut call = call_lock.lock().await;
                        track = call.enqueue_input(src.clone().into()).await;
                    }

                    ctx.client
                        .create_message(interaction.channel.as_ref().unwrap().id)
                        .embeds(&vec![EmbedBuilder::new()
                            .color(0xf04628)
                            .title(metadata.title.as_ref().unwrap())
                            .url(metadata.source_url.as_ref().unwrap())
                            .thumbnail(
                                ImageSource::url(metadata.thumbnail.as_ref().unwrap()).unwrap(),
                            )
                            .description(format!(
                                "Requested by {}",
                                interaction.author().unwrap().mention(),
                            ))
                            .build()])
                        .unwrap()
                        .await
                        .unwrap();

                    track
                        .add_event(
                            Event::Track(TrackEvent::Playable),
                            TrackPlayableHandler {
                                channel_id: interaction.channel.as_ref().unwrap().id,
                                metadata: metadata.clone(),
                                user: interaction.author().unwrap().id,
                                ctx: ctx.clone(),
                            },
                        )
                        .unwrap();

                    tracing::info!("Queued track {}", &metadata.title.unwrap())
                } else {
                    tracing::error!("Bami is not in a voice channel");
                    bail!("Bami is not in a voice channel");
                }
            } else {
                client
                    .update_response(&interaction.token)
                    .content(Some("Error processing your request"))?
                    .await?;
            }
        }

        client
            .update_response(&interaction.token)
            .content(Some(format!("Qeueued {} songs", to_queue.len()).as_str()))?
            .await?;

        Ok(())
    }
}
