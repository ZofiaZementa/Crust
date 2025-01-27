#![allow(clippy::field_reassign_with_default)]

pub mod channel;
pub mod content;
pub mod error;
pub mod guild;
pub mod member;
pub mod message;

use channel::Channel;
use guild::Guild;
pub use harmony_rust_sdk::{
    api::exports::hrpc::url::Url,
    client::{api::auth::Session as InnerSession, AuthStatus, Client as InnerClient},
};
use harmony_rust_sdk::{
    api::{
        chat::{event::*, DeleteMessageRequest},
        harmonytypes::{Message as HarmonyMessage, UserStatus},
    },
    client::api::{
        chat::{
            message::{
                delete_message, send_message, update_message_text, SendMessage,
                SendMessageSelfBuilder, UpdateMessageTextRequest,
            },
            EventSource,
        },
        rest::FileId,
    },
};

use content::ContentStore;
use error::{ClientError, ClientResult};
use iced::Command;
use member::{Member, Members};
use message::{harmony_messages_to_ui_messages, Attachment, Content, Embed, MessageId};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Debug, Formatter},
    path::PathBuf,
    str::FromStr,
    sync::Arc,
    time::{Duration, Instant},
};

use crate::ui::component::event_history::SHOWN_MSGS_LIMIT;

use self::{
    guild::Guilds,
    message::{EmbedHeading, Message},
};

/// A sesssion struct with our requirements (unlike the `InnerSession` type)
#[derive(Clone, Deserialize, Serialize)]
pub struct Session {
    pub session_token: String,
    pub user_id: String,
    pub homeserver: String,
}

impl Debug for Session {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("user_id", &self.user_id)
            .field("homeserver", &self.homeserver)
            .finish()
    }
}

impl From<Session> for InnerSession {
    fn from(session: Session) -> Self {
        InnerSession {
            user_id: session.user_id.parse().unwrap(),
            session_token: session.session_token,
        }
    }
}

#[derive(Debug)]
pub enum PostProcessEvent {
    FetchProfile(u64),
    FetchGuildData(u64),
    FetchThumbnail(Attachment),
    GoToFirstMsgOnChannel(u64),
}

pub struct Client {
    inner: InnerClient,
    pub guilds: Guilds,
    pub members: Members,
    pub user_id: Option<u64>,
    content_store: Arc<ContentStore>,
}

impl Debug for Client {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Client")
            .field(
                "user_id",
                &format!(
                    "{:?}",
                    self.auth_status().session().map_or(0, |s| s.user_id)
                ),
            )
            .field("session_file", &self.content_store.session_file())
            .finish()
    }
}

impl Client {
    pub async fn new(
        homeserver_url: Url,
        session: Option<InnerSession>,
        content_store: Arc<ContentStore>,
    ) -> ClientResult<Self> {
        Ok(Self {
            guilds: Guilds::new(),
            members: Members::new(),
            user_id: session.as_ref().map(|s| s.user_id),
            content_store,
            inner: InnerClient::new(homeserver_url, session).await?,
        })
    }

    pub async fn logout(_inner: InnerClient, session_file: PathBuf) -> ClientResult<()> {
        tokio::fs::remove_file(session_file).await?;
        Ok(())
    }

    #[inline(always)]
    pub fn content_store(&self) -> &ContentStore {
        &self.content_store
    }

    #[inline(always)]
    pub fn content_store_arc(&self) -> Arc<ContentStore> {
        self.content_store.clone()
    }

    #[inline(always)]
    pub fn auth_status(&self) -> AuthStatus {
        self.inner.auth_status()
    }

    #[inline(always)]
    pub fn inner(&self) -> &InnerClient {
        &self.inner
    }

    #[inline(always)]
    pub fn get_guild(&mut self, guild_id: u64) -> Option<&mut Guild> {
        self.guilds.get_mut(&guild_id)
    }

    pub fn get_channel(&mut self, guild_id: u64, channel_id: u64) -> Option<&mut Channel> {
        self.get_guild(guild_id)
            .map(|guild| guild.channels.get_mut(&channel_id))
            .flatten()
    }

    #[inline(always)]
    pub fn get_member(&mut self, user_id: u64) -> Option<&mut Member> {
        self.members.get_mut(&user_id)
    }

    pub fn send_msg_cmd(
        &mut self,
        guild_id: u64,
        channel_id: u64,
        retry_after: Duration,
        message: Message,
    ) -> Option<Command<crate::ui::screen::Message>> {
        use crate::ui::screen::Message;

        if let Some(channel) = self.get_channel(guild_id, channel_id) {
            if retry_after.as_secs() == 0 {
                channel.messages.push(message.clone());
            }

            let inner = self.inner().clone();

            Some(Command::perform(
                async move {
                    tokio::time::sleep(retry_after).await;

                    let msg = SendMessage::new(guild_id, channel_id)
                        .content(harmony_rust_sdk::api::harmonytypes::Content {
                            content: Some(message.content.clone().into()),
                        })
                        .echo_id(message.id.transaction_id().unwrap())
                        .overrides(message.overrides.clone().map(Into::into));

                    let send_result = send_message(&inner, msg).await;

                    match send_result {
                        Ok(resp) => Message::MessageSent {
                            message_id: resp.message_id,
                            transaction_id: message.id.transaction_id().unwrap(),
                            channel_id,
                            guild_id,
                        },
                        Err(err) => {
                            tracing::error!("error occured when sending message: {}", err);
                            Message::SendMessage {
                                message,
                                retry_after: retry_after + Duration::from_secs(1),
                                channel_id,
                                guild_id,
                            }
                        }
                    }
                },
                |retry| retry,
            ))
        } else {
            None
        }
    }

    pub fn delete_msg_cmd(
        &self,
        guild_id: u64,
        channel_id: u64,
        message_id: u64,
    ) -> Command<crate::ui::screen::Message> {
        use crate::ui::screen::Message;

        let inner = self.inner().clone();

        Command::perform(
            async move {
                delete_message(
                    &inner,
                    DeleteMessageRequest {
                        guild_id,
                        channel_id,
                        message_id,
                    },
                )
                .await
            },
            |result| {
                result.map_or_else(
                    |err| Message::Error(Box::new(err.into())),
                    |_| Message::Nothing,
                )
            },
        )
    }

    pub fn edit_msg_cmd(
        &self,
        guild_id: u64,
        channel_id: u64,
        message_id: u64,
        new_content: String,
    ) -> Command<crate::ui::screen::Message> {
        use crate::ui::screen::Message;

        let inner = self.inner().clone();

        Command::perform(
            async move {
                let result = update_message_text(
                    &inner,
                    UpdateMessageTextRequest {
                        guild_id,
                        channel_id,
                        message_id,
                        new_content,
                    },
                )
                .await;

                result.map_or_else(
                    |err| Message::MessageEdited {
                        guild_id,
                        channel_id,
                        message_id,
                        err: Some(Box::new(err.into())),
                    },
                    |_| Message::MessageEdited {
                        guild_id,
                        channel_id,
                        message_id,
                        err: None,
                    },
                )
            },
            |m| m,
        )
    }

    pub fn process_event(&mut self, event: Event) -> Vec<PostProcessEvent> {
        let mut post = Vec::new();

        match event {
            Event::SentMessage(message_sent) => {
                let echo_id = message_sent.echo_id;

                if let Some(message) = message_sent.message {
                    let guild_id = message.guild_id;
                    let channel_id = message.channel_id;
                    let message_id = message.message_id;

                    if let Some(channel) = self.get_channel(guild_id, channel_id) {
                        let message = Message::from(message);

                        if let Some(id) = message
                            .overrides
                            .as_ref()
                            .map(|overrides| overrides.avatar_url.clone())
                            .flatten()
                        {
                            post.push(PostProcessEvent::FetchThumbnail(Attachment {
                                kind: "image".into(),
                                ..Attachment::new_unknown(id)
                            }));
                        }

                        message.post_process(&mut post);

                        if let Some(msg) = channel
                            .messages
                            .iter_mut()
                            .find(|omsg| omsg.id == MessageId::Unack(echo_id))
                        {
                            *msg = message;
                        } else if let Some(msg) = channel
                            .messages
                            .iter_mut()
                            .find(|omsg| omsg.id == MessageId::Ack(message_id))
                        {
                            *msg = message;
                        } else {
                            channel.messages.push(message);
                        }

                        let disp = channel.messages.len();
                        if channel.looking_at_message >= disp.saturating_sub(SHOWN_MSGS_LIMIT) {
                            channel.looking_at_message = disp.saturating_sub(1);
                            post.push(PostProcessEvent::GoToFirstMsgOnChannel(channel_id));
                        }
                    }
                }
            }
            Event::DeletedMessage(MessageDeleted {
                guild_id,
                channel_id,
                message_id,
            }) => {
                if let Some(channel) = self.get_channel(guild_id, channel_id) {
                    if let Some(pos) = channel
                        .messages
                        .iter()
                        .position(|msg| msg.id == MessageId::Ack(message_id))
                    {
                        channel.messages.remove(pos);
                    }
                }
            }
            Event::EditedMessage(message_updated) => {
                let guild_id = message_updated.guild_id;
                let channel_id = message_updated.channel_id;

                if let Some(channel) = self.get_channel(guild_id, channel_id) {
                    if let Some(msg) = channel
                        .messages
                        .iter_mut()
                        .find(|message| message.id == MessageId::Ack(message_updated.message_id))
                    {
                        msg.content = Content::Text(message_updated.content);
                    }
                }
            }
            Event::DeletedChannel(ChannelDeleted {
                guild_id,
                channel_id,
            }) => {
                if let Some(guild) = self.get_guild(guild_id) {
                    guild.channels.remove(&channel_id);
                }
            }
            Event::EditedChannel(ChannelUpdated {
                guild_id,
                channel_id,
                name,
                update_name,
                previous_id,
                next_id,
                update_order,
                metadata: _,
                update_metadata: _,
            }) => {
                if let Some(channel) = self.get_channel(guild_id, channel_id) {
                    if update_name {
                        channel.name = name;
                    }
                }

                if update_order {
                    if let Some(guild) = self.get_guild(guild_id) {
                        guild.update_channel_order(previous_id, next_id, channel_id);
                    }
                }
            }
            Event::CreatedChannel(ChannelCreated {
                guild_id,
                channel_id,
                name,
                previous_id,
                next_id,
                is_category,
                metadata: _,
            }) => {
                if let Some(guild) = self.get_guild(guild_id) {
                    guild.channels.insert(
                        channel_id,
                        Channel {
                            is_category,
                            name,
                            loading_messages_history: false,
                            looking_at_message: 0,
                            messages: Vec::new(),
                            reached_top: false,
                        },
                    );
                    guild.update_channel_order(previous_id, next_id, channel_id);
                }
            }
            Event::Typing(Typing {
                guild_id,
                channel_id,
                user_id,
            }) => {
                if let Some(member) = self.get_member(user_id) {
                    member.typing_in_channel = Some((guild_id, channel_id, Instant::now()));
                }
            }
            Event::JoinedMember(MemberJoined {
                guild_id,
                member_id,
            }) => {
                if member_id == 0 {
                    return post;
                }

                if let Some(guild) = self.get_guild(guild_id) {
                    guild.members.insert(member_id);
                }

                if !self.members.contains_key(&member_id) {
                    post.push(PostProcessEvent::FetchProfile(member_id));
                }
            }
            Event::LeftMember(MemberLeft {
                guild_id,
                member_id,
                leave_reason: _,
            }) => {
                if let Some(guild) = self.get_guild(guild_id) {
                    guild.members.remove(&member_id);
                }
            }
            Event::ProfileUpdated(ProfileUpdated {
                user_id,
                new_username,
                update_username,
                new_avatar,
                update_avatar,
                new_status,
                update_status,
                is_bot: _,
                update_is_bot: _,
            }) => {
                let member = self.members.entry(user_id).or_default();
                if update_username {
                    member.username = new_username;
                }
                if update_status {
                    member.status = UserStatus::from_i32(new_status).unwrap();
                }
                if update_avatar {
                    let parsed = FileId::from_str(&new_avatar).ok();
                    member.avatar_url = parsed.clone();
                    if let Some(id) = parsed {
                        post.push(PostProcessEvent::FetchThumbnail(Attachment {
                            kind: "image".into(),
                            ..Attachment::new_unknown(id)
                        }));
                    }
                };
            }
            Event::GuildAddedToList(GuildAddedToList {
                guild_id,
                homeserver: _,
            }) => {
                self.guilds.insert(guild_id, Default::default());
                post.push(PostProcessEvent::FetchGuildData(guild_id));
            }
            Event::GuildRemovedFromList(GuildRemovedFromList {
                guild_id,
                homeserver: _,
            }) => {
                self.guilds.remove(&guild_id);
            }
            Event::DeletedGuild(GuildDeleted { guild_id }) => {
                self.guilds.remove(&guild_id);
            }
            Event::EditedGuild(GuildUpdated {
                guild_id,
                name,
                update_name,
                picture,
                update_picture,
                metadata: _,
                update_metadata: _,
            }) => {
                let guild = self.guilds.entry(guild_id).or_default();

                if update_name {
                    guild.name = name;
                }
                if update_picture {
                    let parsed = FileId::from_str(&picture).ok();
                    guild.picture = parsed.clone();
                    if let Some(id) = parsed {
                        post.push(PostProcessEvent::FetchThumbnail(Attachment {
                            kind: "image".into(),
                            ..Attachment::new_unknown(id)
                        }));
                    }
                }
            }
            x => todo!("implement {:?}", x),
        }

        post
    }

    pub fn process_get_message_history_response(
        &mut self,
        guild_id: u64,
        channel_id: u64,
        messages: Vec<HarmonyMessage>,
        reached_top: bool,
    ) -> Vec<PostProcessEvent> {
        let mut post = Vec::new();
        let mut messages = harmony_messages_to_ui_messages(messages);

        for message in &messages {
            message.post_process(&mut post);
        }

        for overrides in messages.iter().flat_map(|msg| msg.overrides.as_ref()) {
            if let Some(id) = overrides.avatar_url.clone() {
                post.push(PostProcessEvent::FetchThumbnail(Attachment {
                    kind: "image".into(),
                    ..Attachment::new_unknown(id)
                }));
            }
        }

        if let Some(channel) = self.get_channel(guild_id, channel_id) {
            messages.append(&mut channel.messages);
            channel.messages = messages;
            channel.reached_top = reached_top;
        }

        post
    }

    pub fn subscribe_to(&self) -> Vec<EventSource> {
        let mut subs = self
            .guilds
            .keys()
            .map(|guild_id| EventSource::Guild(*guild_id))
            .collect::<Vec<_>>();
        subs.push(EventSource::Homeserver);
        subs
    }
}

fn post_heading(post: &mut Vec<PostProcessEvent>, embed: &Embed) {
    let mut inner = |h: Option<&EmbedHeading>| {
        if let Some(id) = h.map(|h| h.icon.clone()).flatten() {
            post.push(PostProcessEvent::FetchThumbnail(Attachment {
                kind: "image".into(),
                ..Attachment::new_unknown(id)
            }));
        }
    };
    inner(embed.header.as_ref());
    inner(embed.footer.as_ref());
}
