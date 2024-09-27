use super::SimpleCounter;

#[servify_macro::export]
impl SimpleCounter {
    fn get(&self) -> u32 {
        self.counter
    }
}
