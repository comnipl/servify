mod counter {
    use std::any::Any;

    use servify::processor::ServifyProcessor;

    pub(super) trait ImplIncrementAndGet {
        fn process(&mut self, ctx: Context, payload: impl Any) -> impl Any;
    }

    pub(super) struct Processor {
        pub(super) count: u32,
    }

    impl ServifyProcessor for Processor {
        type Context = Context;
    }

    pub(super) enum Context {
        Direct,
        MessagePassing,
    }

}

mod counter_increment_and_get {
    use super::counter;
    use servify::processor::ServifyProcessor;

    trait Process {
        fn process(&mut self, ctx: <counter::Processor as ServifyProcessor>::Context, amount: u32) -> u32;
    }

    impl Process for counter::Processor {
        fn process(&mut self, ctx: <Self as ServifyProcessor>::Context, amount: u32) -> u32 {
            self.count += amount;
            self.count
        }
    }

    impl counter::Processor {
        pub fn increment_and_get(&mut self, amount: u32) -> u32 {
            self.process(counter::Context::Direct, amount)
        }
    }
}