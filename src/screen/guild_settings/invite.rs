use super::GuildMetaData;
use super::super::Screen as TopLevelScreen;
use crate::{
    length,
    label_button,
    component::*,
    screen::{
        guild_settings::{Icon, Message as ParentMessage, Tab},
        Client, Message as TopLevelMessage, ScreenMessage as TopLevelScreenMessage
    },
    style::Theme,
};
use harmony_rust_sdk::{
    api::exports::hrpc::url::Url,
    client::api::chat::invite::{
        create_invite, delete_invite, get_guild_invites_response::Invite, CreateInviteRequest,
        DeleteInvite,
    },
};
use iced::{Column, Element};
use iced_aw::TabLabel;

#[derive(Debug, Clone)]
pub enum InviteMessage {
    InviteNameChanged(String),
    InviteUsesChanged(String),
    CreateInvitePressed,
    InviteCreated((String, i32)),
    InvitesLoaded(Vec<Invite>),
    GoBack(),
    Nothing,
}

#[derive(Debug, Default)]
pub struct InviteTab {
    invite_name_state: text_input::State,
    invite_name_value: String,
    invite_uses_state: text_input::State,
    invite_uses_value: String,
    create_invite_but_state: button::State,
    invite_list_state: scrollable::State,
    back_but_state: button::State,
}

// TODO delete invites
// TODO clear invite field on succesful invite creation

impl InviteTab {
    pub fn update(
        &mut self,
        message: InviteMessage,
        client: &Client,
        meta_data: &mut GuildMetaData,
        guild_id: u64,
    ) -> Command<TopLevelMessage> {
        match message {
            InviteMessage::InviteNameChanged(s) => {
                self.invite_name_value = s;
            }
            InviteMessage::InviteUsesChanged(s) => {
                self.invite_uses_value = s;
            }
            InviteMessage::CreateInvitePressed => {
                let inner = client.inner().clone();
                let uses = self.invite_uses_value.clone();
                let name = self.invite_name_value.clone();
                return Command::perform(
                    async move {
                        let uses: i32 = uses.parse().expect("Possible uses has to be int");
                        let request = CreateInviteRequest {
                            name,
                            possible_uses: uses,
                            guild_id,
                        };
                        let name = create_invite(&inner, request).await?.name;
                        Ok(TopLevelMessage::ChildMessage(TopLevelScreenMessage::GuildSettings(ParentMessage::Invite(
                            InviteMessage::InviteCreated((name, uses)),
                        ))))
                    },
                    |result| result.unwrap_or_else(|err| TopLevelMessage::Error(Box::new(err))),
                );
            }
            InviteMessage::InvitesLoaded(invites) => {
                meta_data.invites = Some(invites);
            }
            InviteMessage::InviteCreated((name, uses)) => {
                let new_invite = Invite {
                    invite_id: name,
                    possible_uses: uses,
                    use_count: 0,
                };
                if let Some(invites) = &mut meta_data.invites {
                    invites.push(new_invite);
                } else {
                    meta_data.invites = Some(vec![new_invite]);
                }
            }
            InviteMessage::GoBack() => {
                return TopLevelScreen::push_screen_cmd(TopLevelScreen::Main(
                    Box::new( super::super::MainScreen::default()),
                ));
            },
            _ => {}
        }

        Command::none()
    }
}

impl Tab for InviteTab {
    fn title(&self) -> String {
        String::from("Invites")
    }

    fn tab_label(&self) -> TabLabel {
        TabLabel::IconText(Icon::Heart.into(), self.title())
    }

    fn content(
        &mut self,
        client: &Client,
        meta_data: &mut GuildMetaData,
        theme: Theme,
        _: &ThumbnailCache,
    ) -> Element<'_, ParentMessage> {
        let mut widgets = vec![];
        let mut back = label_button!(&mut self.back_but_state, "Back").style(theme);
        back = back.on_press(ParentMessage::Invite(InviteMessage::GoBack()));
        if let Some(invites) = &meta_data.invites {
            let mut invite_list = Scrollable::new(&mut self.invite_list_state).width(length!(+));
            let mut invite_url_column = vec![label!["Invite Id"].into()];
            let mut possible_uses_column = vec![label!["Possible uses"].into()];
            let mut uses_column = vec![label!["Uses"].into()];
            let homeserver_url = client.inner().homeserver_url();
            let mut url;
            if let Some(port) = homeserver_url.port() {
                url = Url::parse(
                    format!("harmony://{}:{}/", homeserver_url.host().unwrap(), port).as_str(),
                )
                .unwrap();
            } else {
                url = Url::parse(format!("harmony://{}/", homeserver_url.host().unwrap()).as_str())
                    .unwrap();
            }
            url.set_scheme("harmony").unwrap();
            for cur_invite in invites {
                url.set_path(&cur_invite.invite_id);
                invite_url_column.push(label![url.as_str().clone()].into());
                possible_uses_column.push(label![cur_invite.possible_uses.to_string()].into());
                uses_column.push(label![cur_invite.use_count.to_string()].into());
            }
            invite_list = invite_list.push(row(vec![
                column(invite_url_column).width(length!(+)).into(),
                column(possible_uses_column).width(length!(= 200)).into(),
                column(uses_column).width(length!(= 80)).into(),
            ]));
            widgets.push(invite_list.into());
        } else {
            widgets.push(label!["Fetching invites"].into());
        }

        widgets.push(
            row(vec![
                TextInput::new(
                    &mut self.invite_name_state,
                    "Enter invite name...",
                    self.invite_name_value.as_str(),
                    |s| ParentMessage::Invite(InviteMessage::InviteNameChanged(s)),
                )
                .style(theme)
                .into(),
                TextInput::new(
                    &mut self.invite_uses_state,
                    "Enter possible uses...",
                    self.invite_uses_value.as_str(),
                    |s| ParentMessage::Invite(InviteMessage::InviteUsesChanged(s)),
                )
                .width(length!(= 200))
                .style(theme)
                .into(),
                Button::new(&mut self.create_invite_but_state, label!["Create"])
                    .style(theme)
                    .on_press(ParentMessage::Invite(InviteMessage::CreateInvitePressed))
                    .into(),
            ])
            .into(),
        );
        widgets.push(back.into());

        Column::with_children(widgets).into()
    }
}
