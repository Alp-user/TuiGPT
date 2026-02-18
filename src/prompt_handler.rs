#[derive(Default, Debug)]
pub struct PromptHandler {
    c_prompt: String,
}
impl PromptHandler {
    pub fn current_prompt(&self) -> &str {
        &self.c_prompt
    }
    pub fn add_char_prompt(&mut self, new_char: char) {
        self.c_prompt.push(new_char);
    }
    pub fn remove_char_prompt(&mut self) {
        self.c_prompt.pop();
    }
    pub fn clear_prompt(&mut self) {
        self.c_prompt.clear();
    }
}
