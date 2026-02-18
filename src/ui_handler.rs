use std::rc::Rc;

use ratatui::{
    Frame,
    layout::{self, Constraint, Layout, Rect, Rows},
    symbols,
    text::{Line, Text},
    widgets::{self, Block, Clear, Paragraph, Row, Table, TableState, Widget},
};

use crate::{
    conversation_handler::ConversationHandler,
    error_handler::ErrorHandler,
    network_handler::{self, NetworkHandler},
    prompt_handler::PromptHandler,
};

#[derive(Default, Debug)]
struct Areas {
    prompt: Option<Rect>,
    conversation: Option<Rect>,
    error: Option<Rect>,
    popup: Option<Rect>,
}

#[derive(Default, Debug)]
struct Elements<'b> {
    prompt: Option<Paragraph<'b>>,
    conversation: Option<Paragraph<'b>>,
    error: Option<Paragraph<'b>>,
    popup: Option<Table<'b>>,
}

#[derive(Default, Debug)]
pub enum LayoutPreset {
    #[default]
    DefaultLayout,
    AnotherLayout,
}

impl LayoutPreset {
    fn compute_layout(&self, area: Rect) -> Areas {
        match self {
            LayoutPreset::DefaultLayout => {
                let [conversation, prompt] = Layout::default()
                    .direction(layout::Direction::Vertical)
                    .constraints([layout::Constraint::Min(1), layout::Constraint::Length(3)])
                    .areas(area);
                let (prompt, conversation, error) =
                    (Some(prompt), Some(conversation), Some(prompt));
                let vertical_popup_area = Layout::default()
                    .direction(layout::Direction::Vertical)
                    .constraints([
                        layout::Constraint::Fill(1),
                        layout::Constraint::Min(1),
                        layout::Constraint::Fill(1),
                    ])
                    .areas::<3>(area)[1];

                let popup = Layout::default()
                    .direction(layout::Direction::Horizontal)
                    .constraints([
                        layout::Constraint::Fill(1),
                        layout::Constraint::Min(1),
                        layout::Constraint::Fill(1),
                    ])
                    .areas::<3>(vertical_popup_area)[1];
                let popup = Some(popup);

                Areas {
                    prompt,
                    conversation,
                    error,
                    popup,
                }
            }
            LayoutPreset::AnotherLayout => Areas {
                prompt: None,
                conversation: None,
                error: None,
                popup: None,
            },
        }
    }
}

#[derive(Default, Debug)]
pub struct UiHandler {
    areas: Areas,
    elements: Elements<'static>,
    layout_preset: LayoutPreset,
    pub show_popup: bool,
    popup_state: TableState,
}
// uses prompt_handler and conversation handler classes
impl UiHandler {
    fn form_elements(
        &mut self,
        prompt_handler: &PromptHandler,
        conversation_handler: &ConversationHandler,
        network_handler: &NetworkHandler,
        error_handler: &ErrorHandler,
    ) {
        // TODO: remove lifetimes, they should have their own strings don't use the ones in the
        // other structs lots of problems
        let new_elements = Elements {
            prompt: Some(self.form_prompt(prompt_handler)),
            conversation: Some(self.form_conversation(conversation_handler, network_handler)),
            error: if error_handler.error_exists() {
                Some(self.form_error(error_handler))
            } else {
                None
            },
            popup: if self.show_popup {
                Some(self.form_popup())
            } else {
                None
            },
        };
        self.elements = new_elements;
    }

    pub fn toggle_popup(&mut self) {
        self.popup_state.select_first();
        self.show_popup = !self.show_popup;
    }

    pub fn select_next_popup(&mut self) {
        self.popup_state.select_next();
    }

    pub fn select_prev_popup(&mut self) {
        self.popup_state.select_previous();
    }

    pub fn get_popup_id(&self) -> usize {
        self.popup_state.selected().unwrap()
    }

    fn form_popup(&mut self) -> Table<'static> {
        let titles = network_handler::get_all_models();
        let rows = titles.iter().map(|&x| Row::new([x]));
        let constraints = [Constraint::Fill(1)];
        Table::new(rows, constraints)
            .block(Block::bordered())
            .highlight_symbol(">>")
    }

    fn form_prompt(&self, prompt_handler: &PromptHandler) -> Paragraph<'static> {
        Paragraph::new(Text::from(String::from(prompt_handler.current_prompt())))
            .wrap(widgets::Wrap { trim: false })
    }

    fn form_conversation(
        &self,
        conversation_handler: &ConversationHandler,
        network_handler: &NetworkHandler,
    ) -> Paragraph<'static> {
        let text = Text::from(
            conversation_handler
                .messages
                .iter()
                .flat_map(|x| [Line::from(format!("{}:{}", x.0, x.1)), Line::from("")])
                .collect::<Vec<Line>>(),
        );
        Paragraph::new(text)
            .block(
                Block::bordered()
                    .border_set(symbols::border::DOUBLE)
                    .title(network_handler.get_selected_model()),
            )
            .wrap(widgets::Wrap { trim: false })
            .scroll((conversation_handler.scroll_offset, 0))
    }

    fn form_error(&self, error_handler: &ErrorHandler) -> Paragraph<'static> {
        // this function should not be called when an error event is not triggered so unwrap
        Paragraph::new(Text::from(String::from(
            error_handler.get_error_msg().unwrap(),
        )))
        .wrap(widgets::Wrap { trim: false })
    }

    pub fn clear_prompt(&mut self, frame: &mut Frame) {
        frame.render_widget(Clear, self.areas.prompt.unwrap());
    }

    pub fn clear_conversation(&mut self, frame: &mut Frame) {
        frame.render_widget(Clear, self.areas.conversation.unwrap());
    }

    pub fn clear_error(&mut self, frame: &mut Frame) {
        frame.render_widget(Clear, self.areas.error.unwrap());
    }

    fn update_ui_default_layout(
        &mut self,
        frame: &mut Frame,
        prompt_handler: &PromptHandler,
        conversation_handler: &mut ConversationHandler,
        network_handler: &NetworkHandler,
        error_handler: &ErrorHandler,
    ) {
        self.areas = self.layout_preset.compute_layout(frame.area());
        self.form_elements(
            prompt_handler,
            conversation_handler,
            network_handler,
            error_handler,
        );
        conversation_handler.update_conversation_metrics(
            self.areas.conversation.unwrap(),
            self.elements.conversation.as_mut().unwrap(),
        );
    }

    pub fn update(
        &mut self,
        frame: &mut Frame,
        prompt_handler: &PromptHandler,
        conversation_handler: &mut ConversationHandler,
        network_handler: &NetworkHandler,
        error_handler: &ErrorHandler,
    ) {
        match self.layout_preset {
            LayoutPreset::DefaultLayout => {
                self.update_ui_default_layout(
                    frame,
                    prompt_handler,
                    conversation_handler,
                    network_handler,
                    error_handler,
                );
            }
            LayoutPreset::AnotherLayout => {}
        }
    }

    fn render_prompt(&self, frame: &mut Frame, prompt_handler: &PromptHandler) {
        // let prompt = self.form_prompt(prompt_handler);
        frame.render_widget(
            self.elements.prompt.as_ref().unwrap(),
            self.areas.prompt.unwrap(),
        );
    }

    fn render_conversation(&self, frame: &mut Frame) {
        frame.render_widget(
            self.elements.conversation.as_ref().unwrap(),
            self.areas.conversation.unwrap(),
        );
    }

    fn render_popup(&mut self, frame: &mut Frame) {
        if self.show_popup {
            frame.render_stateful_widget(
                self.elements.popup.as_ref().unwrap(),
                self.areas.popup.unwrap(),
                &mut self.popup_state,
            );
        }
    }

    fn render_error(
        &mut self,
        frame: &mut Frame,
        error_handler: &ErrorHandler,
        prompt_handler: &mut PromptHandler,
    ) {
        if error_handler.error_exists() {
            prompt_handler.clear_prompt();
            self.clear_error(frame);
            // let error = self.form_error(error_handler);
            frame.render_widget(
                self.elements.error.as_ref().unwrap(),
                self.areas.error.unwrap(),
            );
        }
    }

    fn draw_default_layout(
        &mut self,
        frame: &mut Frame,
        prompt_handler: &mut PromptHandler,
        error_handler: &mut ErrorHandler,
    ) {
        self.render_prompt(frame, prompt_handler);
        self.render_conversation(frame);
        self.render_error(frame, error_handler, prompt_handler);
        self.render_popup(frame);
    }

    pub fn draw(
        &mut self,
        frame: &mut Frame,
        prompt_handler: &mut PromptHandler,
        error_handler: &mut ErrorHandler,
    ) {
        match self.layout_preset {
            LayoutPreset::DefaultLayout => {
                self.draw_default_layout(frame, prompt_handler, error_handler)
            }
            LayoutPreset::AnotherLayout => {
                self.draw_default_layout(frame, prompt_handler, error_handler)
            }
        }
    }
}
