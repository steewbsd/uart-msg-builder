use crate::message::Message;

// Holds an UART instance (a writer) and allows performing
// serialized sends on it.
pub struct UartSender<'a, T, F>
where
    T: embedded_io::Write,
    F: FnMut(u8, &u8) -> u8 + Copy,
{
    uart: T,
    message: Message<'a, F>,
}

impl<'a, T, F> UartSender<'a, T, F>
where
    T: embedded_io::Write,
    F: FnMut(u8, &u8) -> u8 + Copy,
{
    pub fn new(uart: T, msg: Message<'a, F>) -> UartSender<'a, T, F> {
        let us = UartSender {
            uart: uart,
            message: msg
        };
        us
    }

    pub fn send(self) {}
}
