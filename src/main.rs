use std::{fs::File, io, path::Path};

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::DefaultTerminal;

mod conversation_handler;
mod error_handler;
mod event_handler;
mod network_handler;
mod prompt_handler;
mod ui_handler;

use conversation_handler::ConversationHandler;
use error_handler::ErrorHandler;
use event_handler::EventHandler;
use network_handler::NetworkHandler;
use prompt_handler::PromptHandler;
use ui_handler::UiHandler;

use crate::event_handler::AppEvent;

#[tokio::main]
async fn main() -> Result<()> {
    let mut terminal = ratatui::init();
    App::new(&mut terminal).run()?;
    ratatui::restore();
    Ok(())
}

pub struct App<'b> {
    ui_handler: UiHandler,
    network_handler: NetworkHandler,
    event_handler: EventHandler,
    conversation_handler: ConversationHandler,
    prompt_handler: PromptHandler,
    error_handler: ErrorHandler,
    terminal: &'b mut DefaultTerminal,
    exit: bool,
}

impl<'b> App<'b> {
    pub fn run(&mut self) -> Result<()> {
        self.network_handler
            .provider(network_handler::Provider::HuggingFace);
        //This is for initialization
        self.ui_handler.update(
            &mut self.terminal.get_frame(),
            &self.prompt_handler,
            &mut self.conversation_handler,
            &self.network_handler,
            &self.error_handler,
        );
        while !self.exit {
            self.terminal.draw(|frame| {
                self.ui_handler
                    .draw(frame, &mut self.prompt_handler, &mut self.error_handler)
            })?;
            self.handle_event(event::read()?)?;
            self.update()?;
        }

        Ok(())
    }

    fn new(terminal: &'b mut DefaultTerminal) -> Self {
        App {
            network_handler: NetworkHandler::default(),
            event_handler: EventHandler::default(),
            conversation_handler: ConversationHandler::default(),
            prompt_handler: PromptHandler::default(),
            error_handler: ErrorHandler::default(),
            ui_handler: UiHandler::default(),
            terminal,
            exit: false,
        }
    }

    fn process_prompt(&mut self) -> Result<()> {
        self.ui_handler.clear_prompt(&mut self.terminal.get_frame());
        let response_result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(
                self.network_handler
                    .send_prompt(self.prompt_handler.current_prompt()),
            )
        });
        match response_result {
            Ok(response) => {
                self.conversation_handler
                    .save_pair(&mut self.prompt_handler, &response);
            }
            Err(err) => self
                .error_handler
                .set_error(error_handler::ErrorState::NetworkError, &err.to_string()),
        }
        Ok(())
    }

    fn update(&mut self) -> Result<()> {
        while let Some(event) = self.event_handler.next_event() {
            match event {
                AppEvent::PromptSent => {
                    self.process_prompt()?;
                    self.event_handler.push_event(AppEvent::UpdateUi);
                    self.event_handler.push_event(AppEvent::PromptProcessed);
                }
                AppEvent::Exit => {
                    self.exit = true;
                }
                AppEvent::CharPressed(c) => {
                    if self.error_handler.error_exists() {
                        self.error_handler.clear_error();
                        self.ui_handler.clear_error(&mut self.terminal.get_frame());
                    }
                    self.prompt_handler.add_char_prompt(c);
                    self.event_handler.push_event(AppEvent::UpdateUi);
                }
                AppEvent::ScrollUp => {
                    self.conversation_handler.scroll_up();
                    self.event_handler.push_event(AppEvent::UpdateUi);
                }
                AppEvent::ScrollDown => {
                    self.conversation_handler.scroll_down();
                    self.event_handler.push_event(AppEvent::UpdateUi);
                }
                AppEvent::CharDelete => {
                    self.prompt_handler.remove_char_prompt();
                    self.event_handler.push_event(AppEvent::UpdateUi);
                }
                AppEvent::PromptProcessed => {
                    self.conversation_handler.scroll_to_end();
                    self.event_handler.push_event(AppEvent::UpdateUi);
                }
                AppEvent::UpdateUi => {
                    self.ui_handler.update(
                        &mut self.terminal.get_frame(),
                        &self.prompt_handler,
                        &mut self.conversation_handler,
                        &self.network_handler,
                        &self.error_handler,
                    );
                }
                AppEvent::TogglePopup => {
                    self.ui_handler.toggle_popup();
                    self.event_handler.push_event(AppEvent::UpdateUi);
                }
                AppEvent::SelectedModel => {
                    let model_index = self.ui_handler.get_popup_id();
                    self.network_handler.set_selected_model(model_index);
                    self.event_handler.push_event(AppEvent::TogglePopup);
                }
            }
        }
        Ok(())
    }
    fn handle_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Key(key_event) => match key_event.code {
                KeyCode::Enter => {
                    if self.ui_handler.show_popup {
                        self.event_handler.push_event(AppEvent::SelectedModel);
                    } else {
                        self.event_handler.push_event(AppEvent::PromptSent);
                    }
                }
                KeyCode::Char(c) => {
                    if self.ui_handler.show_popup {
                        match c {
                            'j' => {
                                self.ui_handler.select_next_popup();
                            }
                            'k' => {
                                self.ui_handler.select_prev_popup();
                            }
                            _ => {}
                        }
                    } else {
                        self.event_handler.push_event(AppEvent::CharPressed(c));
                    }
                }
                KeyCode::Up => {
                    self.event_handler.push_event(AppEvent::ScrollUp);
                }
                KeyCode::Down => {
                    self.event_handler.push_event(AppEvent::ScrollDown);
                }
                KeyCode::Backspace => {
                    self.event_handler.push_event(AppEvent::CharDelete);
                }
                KeyCode::Esc => {
                    self.event_handler.push_event(AppEvent::Exit);
                }
                KeyCode::Tab => {
                    self.event_handler.push_event(AppEvent::TogglePopup);
                }
                _ => {}
            },
            Event::FocusLost => {}
            _ => {}
        }
        Ok(())
    }
}
