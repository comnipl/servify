pub use servify_macro::{export, service};

pub trait ServifyExport {
    type Request;
    type Response;
}