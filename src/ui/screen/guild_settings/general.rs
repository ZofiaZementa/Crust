use iced::{Column, Container, Element, Radio, Text};
use iced_aw::{TabLabel};
use crate::ui::screen::guild_settings::{Tab, Message, Icon};
use crate::client::Client;
use crate::ui::component::*;
use tracing::Instrument;

#[derive(Debug, Clone)]
pub enum GeneralMessage {
}

#[derive(Debug)]
pub struct GeneralTab {
    name_edit_state: text_input::State,
    guild_id: u64
}

impl GeneralTab {
    pub fn new(guild_id: u64) -> Self {
        GeneralTab {
            name_edit_state: Default::default(),
            guild_id
        }
    }

    pub fn update(&mut self, message: GeneralMessage) {
        match message {

        }
    }
}

impl Tab for GeneralTab {

    fn title(&self) -> String {
        String::from("Settings")
    }

    fn tab_label(&self) -> TabLabel {
        //TabLabel::Text(self.title())
        TabLabel::IconText(Icon::CogAlt.into(), self.title())
    }

    fn content(&mut self, client: &Client) -> Element<'_, Message> {
        let content= Container::new(
            Column::new().push(
                TextInput::new(&mut self.name_edit_state, "Name",
                               client.guilds.get(&self.guild_id).unwrap().name.as_str(), |_| ())
            )
        ).into();
        content.map(Message::General)
    }
}