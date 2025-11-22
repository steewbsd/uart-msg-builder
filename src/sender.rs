use crate::message::{Message, MAX_DATA_CHUNK};
// Holds an UART instance (a writer) and allows performing
// serialized sends on it.
pub struct Sender<'a, T>
where
    T: embedded_io::Write,
{
    uart: T,
    message: Message<'a>,
}

impl<'a, T> Sender<'a, T>
where
    T: embedded_io::Write,
{
    pub fn new(uart: T, msg: Message<'a>) -> Sender<'a, T> {
        let us = Sender {
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
            res = self.uart.write_all(&self.message.synch()[..]);
            if let Err(_) = res {
                return Err("Could not send the header to the writer");
            };
            // send the message payload
            if data.len() >= MAX_DATA_CHUNK
            
            let res = self.uart.write_all(data);
            // send the checksum last
        };
        Ok(())
    }
}
