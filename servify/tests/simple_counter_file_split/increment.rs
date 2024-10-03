use super::SimpleCounter;

#[servify::export]
impl SimpleCounter {
    fn increment_and_get_ex(&mut self) -> u32 {
        self.counter += 1;
        self.counter
    }
}
