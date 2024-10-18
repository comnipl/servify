//! Serial is the data structure used in a transport layer such as String, Binary, etc.

pub trait ServifyMiddle<T, U, Kind> {
    fn payload(&self) -> T;
    fn kind(&self) -> Kind;

    fn from_message(&self, message: ServifySerialMessage<T>) -> Self;
    fn into_message(self)-> ServifySerialMessage<T>;
}

pub struct ServifySerialMessage<T>(T);
