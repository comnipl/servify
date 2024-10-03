use super::SimpleCounter;

#[servify::export]
impl SimpleCounter {
    fn set(&mut self, value: u32) {
        self.counter = value;
    }
}
