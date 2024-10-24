use servify::processor::ServifyProcessor;

mod counter {
    use servify::processor::ServifyProcessor;
    
    pub(super) struct Processor {
        pub(super) count: u32,
    }

    impl ServifyProcessor for Processor {
        type Context = Context;
    }

    pub(super) enum Context {
        MessagePassing,
    }

}

impl counter::Processor {
    pub fn increment_and_get(&mut self, ctx: <Self as ServifyProcessor>::Context, amount: u32) -> u32 {
        self.count += amount;
        self.count
    }
}