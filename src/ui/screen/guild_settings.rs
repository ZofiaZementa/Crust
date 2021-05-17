use iced::{Align, Column, Container, Element, Font, Length, Sandbox,
           Settings, Text, };
use iced_aw::{Tabs, TabLabel};
use crate::ui::screen::Message as TopLevelMessage;
mod general;
use crate::ui::screen::guild_settings::general::{GeneralMessage, GeneralTab};
use crate::client::error::ClientError;
use crate::ui::component::Command;
use crate::client::Client;
use crate::ui::style::Theme;



const HEADER_SIZE: u16 = 32;
const TAB_PADDING: u16 = 16;

const ICON_FONT: Font = iced::Font::External{
    name: "Icons",
    bytes: include_bytes!("../../fonts/icons.ttf"),
};

enum Icon {
    User,
    Heart,
    Calc,
    CogAlt,
}

impl From<Icon> for char {
    fn from(icon: Icon) -> Self {
        match icon {
            Icon::User => '\u{E800}',
            Icon::Heart => '\u{E801}',
            Icon::Calc => '\u{F1EC}',
            Icon::CogAlt => '\u{E802}',
        }
    }
}


#[derive(Debug)]
pub struct GuildSettings {
    active_tab: usize,
    general_tab: GeneralTab,
    current_error: String
}

#[derive(Clone, Debug)]
pub enum Message {
    TabSelected(usize),
    General(GeneralMessage),
}


impl GuildSettings {

    pub fn new(guild_id: u64) -> Self {
        GuildSettings {
            active_tab: 0,
            general_tab: GeneralTab::new(guild_id),
            current_error: String::from("")
        }
    }

    fn title(&self) -> String {
        String::from("Guild settings")
    }

    pub fn update(&mut self, message: Message, client: &Client) -> Command<TopLevelMessage> {
        match message {
            Message::TabSelected(selected) => {
                self.active_tab = selected
            },
            Message::General(message) => {
                self.general_tab.update(message)
            },
        }
        Command::none()
    }

    pub fn view(&mut self, theme: Theme, client: &Client) -> Element<'_, Message> {
        let position = iced_aw::TabBarPosition::Top;

        Tabs::new(self.active_tab, Message::TabSelected)
            .push(self.general_tab.tab_label(), self.general_tab.view(client))
            .tab_bar_style(theme)
            .icon_font(ICON_FONT)
            .tab_bar_position(position)
            .into()
    }


    pub fn on_error(&mut self, error: ClientError) -> Command<TopLevelMessage> {
        self.current_error = error.to_string();
        Command::none()
    }
}

trait Tab {

    fn title(&self) -> String;

    fn tab_label(&self) -> TabLabel;

    fn view(&mut self, client: &Client) -> Element<'_, Message> {
        let column = Column::new()
            .spacing(20)
            .push(self.content(client));

        Container::new(column)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Align::Center)
            .align_y(Align::Center)
            .padding(TAB_PADDING)
            .into()
    }

    fn content(&mut self, client: &Client) -> Element<'_, Message>;
}