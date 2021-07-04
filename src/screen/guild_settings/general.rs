use crate::screen::guild_settings::TabLabel;
use super::super::Screen as TopLevelScreen;
use super::GuildMetaData;
use crate::client::Client;
use crate::component::*;
use crate::screen::guild_settings::{Message as ParentMessage, Tab};
use crate::screen::main::Message::GuildChanged;
use crate::screen::select_upload_files;
use crate::screen::Message::MessageEdited;
use crate::style::{Theme, PADDING};
use crate::{label, length, label_button};
use iced_native::Widget;
use super::super::{
    ScreenMessage as TopLevelScreenMessage,
    Message as TopLevelMessage,
};
use crate::client::error::{ClientResult, ClientError};
use client::harmony_rust_sdk::client::api::chat::guild::{update_guild_information, UpdateGuildInformation};
use std::error::Error;
use iced_aw::Icon;

#[derive(Debug, Clone)]
pub enum GeneralMessage {
    NameChanged(String),
    NameButPressed(),
    NameButErr(ClientError),
    NameButSuccess(),
    GoBack(),
    UploadGuildImage(),
    Nothing,
}

#[derive(Debug)]
pub struct GeneralTab {
    name_edit_state: text_input::State,
    name_edit_field: String,
    name_edit_but_state: button::State,
    icon_edit_but_state: button::State,
    back_but_state: button::State,
    loading_text: Option<String>,
    guild_id: u64,
}

impl GeneralTab {
    pub fn new(guild_id: u64) -> Self {
        GeneralTab {
            name_edit_state: Default::default(),
            name_edit_field: Default::default(),
            name_edit_but_state: Default::default(),
            icon_edit_but_state: Default::default(),
            back_but_state: Default::default(),
            loading_text: None,
            guild_id,
        }
    }

    pub fn update(
        &mut self,
        message: GeneralMessage,
        client: &Client,
        _: &mut GuildMetaData,
        guild_id: u64,
    ) -> Command<TopLevelMessage> {
        match message {
            GeneralMessage::NameChanged(text) => {
                self.name_edit_field = text;
            }
            GeneralMessage::NameButPressed() => {
                self.loading_text = Some("Updating ...".parse().unwrap());
                let current_name = self.name_edit_field.clone();
                let client_inner = client.inner().clone();
                let guild_id_inner = guild_id.clone();
                return Command::perform(
                    async move {
                        let guild_info_req_builder = UpdateGuildInformation::new(guild_id_inner);
                        let guild_info_req = UpdateGuildInformation::new_guild_name(
                            guild_info_req_builder,
                            current_name,
                        );
                        return update_guild_information(&client_inner, guild_info_req).await;
                    },
                    |result| {
                        result.map_or_else(
                            |err| {
                                TopLevelMessage::ChildMessage(TopLevelScreenMessage::GuildSettings(ParentMessage::General(
                                    GeneralMessage::NameButErr(err.into()),
                                )))
                            },
                            |_| {
                                TopLevelMessage::ChildMessage(TopLevelScreenMessage::GuildSettings(ParentMessage::General(
                                    GeneralMessage::NameButSuccess(),
                                )))
                            },
                        )
                    },
                );
            }
            GeneralMessage::NameButErr(err) => {
                self.loading_text = None;
                TopLevelMessage::Error(Box::new(err.into()));
            }
            GeneralMessage::NameButSuccess() => {
                self.loading_text = Some("Name updated".to_string());
            }
            GeneralMessage::UploadGuildImage() => {
                let inner = client.inner().clone();
                let content_store = client.content_store_arc();
                return Command::perform(
                    async move {
                        let id = select_upload_files(&inner, content_store, true)
                            .await?
                            .remove(0)
                            .id;
                        let update_info = UpdateGuildInformation::new(guild_id);
                        Ok(update_guild_information(
                            &inner,
                            UpdateGuildInformation::new_guild_picture(update_info, Some(id)),
                        )
                        .await?)
                    },
                    |result| {
                        result.map_or_else(
                            |err| TopLevelMessage::Error(Box::new(err)),
                            |_| {
                                TopLevelMessage::Nothing
                            },
                        )
                    },
                );
            },
            GeneralMessage::GoBack() => {
                return TopLevelScreen::push_screen_cmd(TopLevelScreen::Main(
                    Box::new( super::super::MainScreen::default()),
                ));
            }
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
        TabLabel::IconText(Icon::Gear.into(), self.title())
    }

    fn content(
        &mut self,
        client: &Client,
        _: &mut GuildMetaData,
        theme: Theme,
        thumbnail_cache: &ThumbnailCache,
    ) -> Element<'_, ParentMessage> {
        let name_edit_but_state = &mut self.name_edit_but_state;
        let guild = client.guilds.get(&self.guild_id).unwrap();
        let mut back = label_button!(&mut self.back_but_state, "Back").style(theme);
        back = back.on_press(ParentMessage::General(GeneralMessage::GoBack()));
        let ui_text_input_row = row(vec![
            TextInput::new(
                &mut self.name_edit_state,
                guild.name.as_str(),
                self.name_edit_field.as_str(),
                |text| ParentMessage::General(GeneralMessage::NameChanged(text)),
            )
            .style(theme)
            .padding(PADDING / 2)
            .width(length!(= 300))
            .into(),
            Button::new(name_edit_but_state, label!["Update"])
                .on_press(ParentMessage::General(GeneralMessage::NameButPressed()))
                .style(theme)
                .into(),
        ])
        .into();

        let ui_update_guild_icon = fill_container(
            guild
                .picture
                .as_ref()
                .map(|guild_picture| thumbnail_cache.thumbnails.get(guild_picture))
                .flatten()
                .map_or_else(
                    || {
                        Element::from(
                            label!(guild
                                .name
                                .chars()
                                .next()
                                .unwrap_or('u')
                                .to_ascii_uppercase())
                            .size(30),
                        )
                    },
                    |handle| Element::from(Image::new(handle.clone())),
                ),
        );

        let ui_image_but = Button::new(&mut self.icon_edit_but_state, ui_update_guild_icon)
            .on_press(ParentMessage::General(GeneralMessage::UploadGuildImage()))
            .height((length!(= 50)))
            .width((length!(= 50)))
            .style(theme)
            .into();

        if let Some(ldg_text) = &self.loading_text {
            let content = Container::new(column(vec![
                label!("Icon").into(),
                ui_image_but,
                label!("Name").into(),
                label!(ldg_text).into(),
                ui_text_input_row,
                back.into()
            ]));
            content.into()
        } else {
            let content = Container::new(column(vec![
                label!("Icon").into(),
                ui_image_but,
                label!("Name").into(),
                ui_text_input_row,
                back.into()
            ]));
            content.into()
        }
    }
}
