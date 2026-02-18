#[derive(Default, Debug)]
pub enum ErrorState {
    #[default]
    Ok,
    AppError,
    NetworkError,
}

#[derive(Default, Debug)]
pub struct ErrorHandler {
    error_type: ErrorState,
    error_msg: Option<String>,
}

impl ErrorHandler {
    pub fn get_error_msg(&self) -> Option<&str> {
        self.error_msg.as_deref()
    }

    pub fn error_exists(&self) -> bool {
        !matches!(self.error_type, ErrorState::Ok)
    }

    pub fn set_error(&mut self, error_type: ErrorState, error_msg: &str) {
        self.error_type = error_type;
        self.error_msg = Some(String::from(error_msg));
    }

    pub fn clear_error(&mut self) {
        self.error_type = ErrorState::Ok;
        self.error_msg = None;
    }
}
