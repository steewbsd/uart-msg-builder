#[derive(Debug)]
// Provides the option to send a short (2 bytes), medium (4 bytes) or long (8 bytes)
// synchronization header
pub enum Synchronization {
    Short([u8; 2]),  // 2 byte header
    Medium([u8; 4]), // 4 byte header
    Long([u8; 8]),   // 8 byte header
}

#[derive(Debug)]
// Structure for a serial message
pub struct Message<'a, F>
where
    F: FnMut(u8, &u8) -> u8 + Copy,
{
    // Synchronization header
    synch: Synchronization,
    // Message payload
    data: Option<&'a [u8]>,
    // Payload cheksum
    checksum: Option<u8>,
    // Optional checksum function provided by the implementor
    checksum_func: Option<F>,
}

impl<'a, F> Message<'a, F>
where
    F: FnMut(u8, &u8) -> u8 + Copy,
{
    // Builds a message with the provided data and calculates the CHECKSUM for it,
    // appending it to the message body.
    pub fn build(
        data: &'a [u8],
        synch: Option<Synchronization>,
        chfn: &'a F,
    ) -> Result<Message<'a, F>, &'static str> {
        let mut slf = Message {
            synch: Synchronization::Short([0; 2]),
            data: None,
            checksum: None,
            checksum_func: None,
        };
        if let Some(s) = synch {
            slf.synch = s;
        }
        slf.checksum_func = Some(*chfn);
        slf.data = Some(data);
        let checksum = slf.get_checksum()?;
        slf.checksum = Some(checksum);
        Ok(slf)
    }

    // Calculates the checksum of the data in the message body. If no checksum function
    // implementation has been provided, it defaults to a basic element-wise
    // XOR checksum.
    pub fn get_checksum(&self) -> Result<u8, &'static str> {
        // check if the user has provided a checksum checking implementation function
        match self.data {
            Some(d) => {
                match self.checksum_func {
                    Some(f) => Ok(d.iter().fold(0, f)),
                    None => {
                        // perform default XOR checksum if no function has been provided
                        Ok(d.iter().fold(0, |acc, &i| acc ^ i))
                    }
                }
            }
            None => Err("These was no data to perform a checksum!"),
        }
    }
}
