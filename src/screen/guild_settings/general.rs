use iced::{Column, Container, Row, Element, Radio, Text};
use iced_aw::{TabLabel};
use crate::screen::guild_settings::{Tab, Message as ParentMessage, Icon};
use crate::component::*;
use crate::client::Client;
use crate::{label, length};
use crate::style::{Theme, PADDING};
use iced_native::Widget;
use super::super::{
    ScreenMessage as TopLevelScreenMessage,
    Message as TopLevelMessage,
};
use crate::client::error::{ClientResult, ClientError};
use harmony_rust_sdk::client::api::chat::guild::{update_guild_information, UpdateGuildInformation};
use std::error::Error;

#[derive(Debug, Clone)]
pub enum GeneralMessage {
    NameChanged(String),
    NameButPressed(),
    NameButErr(ClientError),
    NameButSuccess(),
    Nothing
}

#[derive(Debug)]
pub struct GeneralTab {
    name_edit_state: text_input::State,
    name_edit_field: String,
    name_edit_but_state: button::State,
    loading_text: String,
    loading_show: bool,
    guild_id: u64
}

impl GeneralTab {

    pub fn new(guild_id: u64) -> Self {
        GeneralTab {
            name_edit_state: Default::default(),
            name_edit_field: Default::default(),
            name_edit_but_state: Default::default(),
            loading_text: Default::default(),
            loading_show: false,
            guild_id
        }
    }

    pub fn update(&mut self, message: GeneralMessage, client: &Client, guild_id: u64) -> Command<TopLevelScreenMessage> {
        match message {
            GeneralMessage::NameChanged(text) => {
                self.name_edit_field = text;
            },
            GeneralMessage::NameButPressed() => {
                self.loading_show = true;
                self.loading_text = "Updating ...".parse().unwrap();
                let current_name = self.name_edit_field.clone();
                let client_inner = client.inner().clone();
                let guild_id_inner = guild_id.clone();
                Command::perform(
                    async move {
                        let guild_info_req_builder = UpdateGuildInformation::new(guild_id_inner);
                        let guild_info_req = UpdateGuildInformation::new_guild_name(guild_info_req_builder,current_name);
                        return update_guild_information(&client_inner, guild_info_req).await
                    },
                        |result| {
                            result.map_or_else(
                                |err| {
                                    TopLevelScreenMessage::GuildSettings(ParentMessage::General(GeneralMessage::NameButErr(err.into())))
                                },
                                |ok| {
                                    TopLevelScreenMessage::GuildSettings(ParentMessage::General(GeneralMessage::NameButSuccess()))
                                }
                            );
                        }
                );
            },
            GeneralMessage::NameButErr(err) => {
                self.loading_show = false;
                TopLevelMessage::Error(Box::new(err.into()));
            },
            GeneralMessage::NameButSuccess() => {
                self.loading_text = "Name updated".to_string();
                self.loading_show = true;
            },
            _ => {}
        }
        Command::none()
    }
}

impl Tab for GeneralTab {

    fn title(&self) -> String {
        String::from("General")
    }

    fn tab_label(&self) -> TabLabel {
        //TabLabel::Text(self.title())
        TabLabel::IconText(Icon::CogAlt.into(), self.title())
    }

    fn content(&mut self, client: &Client, theme: Theme) -> Element<'_, ParentMessage> {
        if !self.loading_show {
            let content= Container::new(
                column(vec![

                    label!("Name").into(),
                    row(vec![
                        TextInput::new(&mut self.name_edit_state, client.guilds.get(&self.guild_id).unwrap().name.as_str(),
                                       self.name_edit_field.as_str(),
                                       |text| ParentMessage::General(GeneralMessage::NameChanged(text)))
                            .style(theme)
                            .padding(PADDING / 2)
                            .width(length!(= 300))
                            .into(),
                        Button::new(&mut self.name_edit_but_state,
                                    label!["Update"]
                        ).on_press(ParentMessage::General(GeneralMessage::NameButPressed()))
                            .style(theme).into(),
                    ]).into(), ]
                )
            );
            content.into()
        } else {
            let content= Container::new(
                column(vec![
                    label!(&self.loading_text).into(),
                    label!("Name").into(),
                    row(vec![
                        TextInput::new(&mut self.name_edit_state, client.guilds.get(&self.guild_id).unwrap().name.as_str(),
                                       self.name_edit_field.as_str(),
                                       |text| ParentMessage::General(GeneralMessage::NameChanged(text)))
                            .style(theme)
                            .padding(PADDING / 2)
                            .width(length!(= 300))
                            .into(),
                        Button::new(&mut self.name_edit_but_state,
                                    label!["Update"]
                        ).on_press(ParentMessage::General(GeneralMessage::NameButPressed()))
                            .style(theme).into(),
                    ]).into(), ]
                )
            );
            content.into()
        }

    }
}
