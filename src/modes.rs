pub mod multiple_mode;
pub mod cloud_mode;

pub enum Mode {
    Multiple,
    Cloud
}

pub struct Modes {
    pub mode: Mode,
}

impl Modes {
    pub fn new(mode: Mode) -> Modes {
        Modes {mode}
    }
}