use super::SimpleCounter;

#[servify::export]
impl SimpleCounter {
    fn reset(&mut self) {
        self.counter = 0;
    }
}
