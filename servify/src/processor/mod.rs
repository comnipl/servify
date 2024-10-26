use crate::serial::ServifyMiddle;
use std::any::Any;

pub trait ServifyProcessor {
    type Kind;
    type Context;

    fn serve(
        &mut self,
        middle: ServifyMiddle<Self::Kind, Self::Context>,
    ) -> impl std::future::Future<Output = Box<dyn Any>>;
}
