use std::any::Any;

pub struct ServifyMiddle<Kind, Context> {
    pub ctx: Context,
    pub request: ServifyRequest<Kind>,
}

pub struct ServifyRequest<Kind> {
    pub payload: Box<dyn Any>,
    pub kind: Kind,
}
