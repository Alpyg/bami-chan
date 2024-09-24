use async_trait::async_trait;
use songbird::{input::AuxMetadata, Event, EventContext, EventHandler};
use twilight_mention::Mention;
use twilight_model::id::{
    marker::{ChannelMarker, UserMarker},
    Id,
};
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder, ImageSource};

use crate::{utils::to_timestamp, Context};

#[derive(Debug)]
pub struct TrackPlayableHandler {
    pub user: Id<UserMarker>,
    pub channel_id: Id<ChannelMarker>,
    pub metadata: AuxMetadata,
    pub ctx: Context,
}

#[async_trait]
impl EventHandler for TrackPlayableHandler {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        let _controls = self
            .ctx
            .client
            .create_message(self.channel_id)
            .embeds(&vec![EmbedBuilder::new()
                .color(0xf04628)
                .title("Now playing")
                .field(
                    EmbedFieldBuilder::new(
                        "Song",
                        format!(
                            "[{}]({})",
                            self.metadata.title.as_ref().unwrap(),
                            self.metadata.source_url.as_ref().unwrap()
                        ),
                    )
                    .inline(),
                )
                .field(
                    EmbedFieldBuilder::new(
                        "Duration",
                        to_timestamp(self.metadata.duration.unwrap().as_secs()),
                    )
                    .inline(),
                )
                .field(
                    EmbedFieldBuilder::new("Requested by", format!("{}", self.user.mention()))
                        .inline(),
                )
                .image(ImageSource::url(self.metadata.thumbnail.as_ref().unwrap()).unwrap())
                .build()])
            .unwrap()
            .await
            .unwrap();

        None
    }
}
