use super::SimpleCounter;

#[servify_macro::export]
impl SimpleCounter {
    fn reset(&mut self) {
        self.counter = 0;
    }
}
