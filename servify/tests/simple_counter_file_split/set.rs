use super::SimpleCounter;

#[servify_macro::export]
impl SimpleCounter {
    fn set(&mut self, value: u32) {
        self.counter = value;
    }
}
