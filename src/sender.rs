use crate::message::{Message, Synchronization};
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
            message: msg,
        };
        us
    }

    // Sends the message payload through the writer in chunks of header + payload + chsum
    pub fn send(mut self) -> Result<(), &'static str> {
        while let Some(data) = self.message.data() {
        // send the synchronization header first
        let res;
        match self.message.synch() {
            Synchronization::Short(d) => {
                res = self.uart.write_all(&d[..]);
            },
            Synchronization::Medium(d) => {
                res = self.uart.write_all(&d[..]);
            },
            Synchronization::Long(d) => {
                res = self.uart.write_all(&d[..]);
            },
        }
        if let Err(_) = res {
            return Err("Could not send the header to the writer");
        }
        // send the message payload
        let res = self.uart.write_all(data);
        // send the checksum last
        };
        Ok(())
    }
}
