pub trait Command {
    fn execute(&mut self) {}
    fn undo(&mut self) {}
}

