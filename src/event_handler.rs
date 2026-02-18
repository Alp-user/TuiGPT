use std::collections::VecDeque;

#[derive(Copy, Clone, Debug)]
pub enum AppEvent {
    PromptSent,
    Exit,
    CharPressed(char),
    ScrollUp,
    ScrollDown,
    CharDelete,
    PromptProcessed,
    UpdateUi,
    TogglePopup,
    SelectedModel,
}

#[derive(Default, Debug)]
pub struct EventHandler {
    queue: VecDeque<AppEvent>,
}
impl EventHandler {
    pub fn next_event(&mut self) -> Option<AppEvent> {
        self.queue.pop_front()
    }
    pub fn push_event(&mut self, event: AppEvent) {
        self.queue.push_back(event);
    }
}

impl Iterator for EventHandler {
    type Item = AppEvent;
    fn next(&mut self) -> Option<Self::Item> {
        self.next_event()
    }
}
