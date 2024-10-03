use super::SimpleCounter;

#[servify::export]
impl SimpleCounter {
    fn get(&self) -> u32 {
        self.counter
    }
}
