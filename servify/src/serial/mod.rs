use std::any::Any;

pub struct ServifyMiddle<Kind, Context> {
    pub ctx: Context,
    pub request: ServifyRequest<Kind>,
}

pub struct ServifyRequest<Kind> {
    pub payload: Box<dyn Any + Send>,
    pub kind: Kind,
}

impl<Kind> ServifyRequest<Kind> {
    pub fn with_ctx<Context>(self, ctx: Context) -> ServifyMiddle<Kind, Context> {
        ServifyMiddle { ctx, request: self }
    }
}
