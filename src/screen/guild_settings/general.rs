use iced::{Column, Container, Element, Radio, Text};
use iced_aw::{TabLabel};
use crate::screen::guild_settings::{Tab, Message, Icon};

use crate::{
    client::{error::ClientError, Client},
    component::*,
    style::*,
};

#[derive(Debug, Clone)]
pub enum GeneralMessage {
}

#[derive(Debug)]
pub struct GeneralTab {
}

impl GeneralTab {
    pub fn new() -> Self {
        GeneralTab {
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

    fn content(&mut self) -> Element<'_, Message> {
        let content: Element<'_, GeneralMessage> = Container::new(
            Column::new()
                .push(
                    crate::label!("HELP ME!")
                )
            )
            .into();

        content.map(Message::General)
    }
}
