pub trait BackupOutput {
    fn create_output_map(&mut self) -> Result<(), String>;
}