pub trait Backup {
    fn backup(&mut self) -> Result<(), String>;
}