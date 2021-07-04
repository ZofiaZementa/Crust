use crate::style::Theme;
use crate::screen::Client;
use iced::{Column, Container, Element, Radio, Text};
use iced_aw::{TabLabel};
use crate::screen::guild_settings::{Tab, Message as ParentMessage, Icon};
use crate::component::*;
use crate::style::PADDING;
use crate::length;
use harmony_rust_sdk::{
    api::chat::InviteId,
    client::api::chat::invite::{create_invite,CreateInvite,delete_invite,DeleteInvite,},
};



#[derive(Debug, Clone)]
pub enum InviteMessage {
    Nothing,
}

#[derive(Debug, Default)]
pub struct InviteTab {
    invite_name_state: text_input::State,
    create_invite_but_state: button::State,
}

impl InviteTab {

    pub fn update(&mut self, message: InviteMessage) {
        match message {
            _ => {}
        }
    }
}

impl Tab for InviteTab {

    fn title(&self) -> String {
        String::from("Invites")
    }

    fn tab_label(&self) -> TabLabel {
        //TabLabel::Text(self.title())
        TabLabel::IconText(Icon::Heart.into(), self.title())
    }

    fn content(&mut self, _: &Client, theme: Theme) -> Element<'_, ParentMessage> {
        let content = Container::new(
            row(vec![
                TextInput::new(&mut self.invite_name_state,
                    "Enter invite name...",
                    "",
                    |_| ParentMessage::Invite(InviteMessage::Nothing)
                ).style(theme)
                    .padding(PADDING / 2)
                    .width(length!(= 300))
                 .into(),
                Button::new(&mut self.create_invite_but_state,
                    label!["Create"]
                ).style(theme)
                    .into(),
            ])
        );

        content.into()
    }
}
