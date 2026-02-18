use crate::prompt_handler::PromptHandler;
use ratatui::{layout::Rect, widgets::Paragraph};

#[derive(Default, Debug)]
pub struct ConversationHandler {
    pub messages: Vec<(String, String)>,
    pub scroll_offset: u16,
    pub conversation_width: u16,
    pub conversation_height: u16,
    pub conversation_line_count: usize,
}
//TODO: don't use ratatui wrap lines. Do it yourself
impl ConversationHandler {
    pub fn save_pair(&mut self, prompt: &mut PromptHandler, response: &str) {
        self.save_prompt(prompt);
        self.save_respond(response);
    }

    fn save_prompt(&mut self, prompt_handler: &mut PromptHandler) {
        self.messages.push((
            String::from("user"),
            String::from(prompt_handler.current_prompt()),
        ));
        prompt_handler.clear_prompt();
    }
    fn save_respond(&mut self, respond: &str) {
        self.messages
            .push((String::from("assistant"), String::from(respond)));
    }
    fn max_scroll(&self) -> u16 {
        (self.conversation_line_count as u16).saturating_sub(self.conversation_height)
    }

    fn clamp_scroll(&mut self) {
        let max_scroll = self.max_scroll();
        if self.scroll_offset > max_scroll {
            self.scroll_offset = max_scroll;
        }
    }
    // when new prompt generates, triggered
    pub fn scroll_to_end(&mut self) {
        self.scroll_offset = self.max_scroll();
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }
    pub fn scroll_down(&mut self) {
        let max_scroll = self.max_scroll();
        if self.scroll_offset < max_scroll {
            self.scroll_offset += 1;
        }
    }

    pub fn update_conversation_metrics(&mut self, area: Rect, paragraph: &Paragraph) {
        // this way of updating could be wrong
        let inner_width = area.width.saturating_sub(2);
        let inner_height = area.height.saturating_sub(2);

        self.conversation_width = inner_width;
        self.conversation_height = inner_height;
        self.conversation_line_count = paragraph
            .line_count(self.conversation_width)
            .saturating_sub(2);

        self.clamp_scroll();
    }
}

pub struct ScrollState {}
