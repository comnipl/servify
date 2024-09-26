use super::SimpleCounter;

#[servify_macro::export]
impl SimpleCounter {
    fn increment_and_get(&mut self) -> u32 {
        self.counter += 1;
        self.counter
    }
}
