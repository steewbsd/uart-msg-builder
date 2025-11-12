pub const MAX_DATA_CHUNK: usize = 8;

#[derive(Debug)]
// Structure for a serial message
pub struct Message<'a>
{
    // Synchronization header
    synch: u8,
    // Message payload
    data: Option<&'a [u8]>,
    // Payload cheksum
    checksum: u8,
    // Optional checksum function provided by the implementor
    checksum_poly: u8,
}


impl<'a> Message<'a>
where
{
    // Returns inner synchronization header bytes
    pub fn synch(&self) -> u8 {
        self.synch
    }
    // Returns inner data (if available)
    pub fn data(&self) -> Option<&'a [u8]> {
        self.data
    }
    // Returns inner checksum (if available)
    pub fn checksum(&self) -> u8 {
        self.checksum
    }
    // Returns the max packet size: Header (2, 4 or 8) + chunk size + checksum value
    pub fn packet_size(&self) -> usize {
        // header + data chunk + checksum
        MAX_DATA_CHUNK*2 + 1
    }
    // Builds a message with the provided data and calculates the CHECKSUM for it,
    // appending it to the message body.
    pub fn build(
        data: &'a [u8],
        synch: u8,
        chpoly: u8,
    ) -> Result<Message<'a>, &'static str> {
        let mut slf = Message {
            synch: 0,
            data: None,
            checksum: 0,
            checksum_poly: 0,
        };
        slf.synch = synch;
        slf.checksum_poly = chpoly;
        slf.data = Some(data);
        let checksum = slf.get_checksum()?;
        slf.checksum = checksum;
        Ok(slf)
    }

    // Calculates the CRC of data using the provided polynomial
    fn get_crc(data: &u8, poly: &u8) -> u8 {
        // the amount of zeroes to the left of the polynomial
        let padded_zeroes = poly.leading_zeros();
        // calculate the actual polynomial size not counting the
        // leftwise zero padding
        
        // the actual polynomial size, without padding zeroes
        let poly_len = 8 - padded_zeroes;
        
        // create a temporary value where to perform the crc division on, result of
        // concatenating the data and the remainder (initially 0)
        let mut dividend: u16 = u16::from(*data) << poly_len;
        // iterate over the dividend bits and perform the bitwise xor with the polynomial
        let mut rotated_poly: u16 = u16::from(*poly) << (padded_zeroes + 8);
        // single bit that serves to check if the current dividend leading bit is 0 and we
        // should skip it.
        let mut zero_bit_check: u16 = 1 << 15;
        // iterate over the polynomial until we reach the padding
        let target: u16 = 0b1111_1111 << poly_len;
        while dividend & target != 0 {
            // rotate both the crc polynomial and the zero bit check
            // one position to the right
            rotated_poly >>= 1;
            zero_bit_check >>= 1;
            // if the leading polynomial bit coincides with a 0 on the
            // dividend, skip this calculation.
            if dividend & zero_bit_check == 0 { continue; };
            // perform a bitwise xor of the data with the
            // shifted polynomial
            dividend ^= rotated_poly;
        }
        // finally, we get the remainder
        let rem_mask: u16 = 0xFF_FF >> (16 - poly_len);
        let rem = dividend | rem_mask;
        // we can ignore the unwrap as this type cast should be safe
        u8::try_from(rem).unwrap()
        
    }

    // Calculates the checksum of the data in the message body. If no checksum function
    // implementation has been provided, it defaults to a basic element-wise
    // XOR checksum.
    pub fn get_checksum(&self) -> Result<u8, &'static str> {
        // check if the user has provided a checksum checking implementation function
        match self.data {
            Some(d) => {
                Ok(Self::get_crc(d.first().unwrap(), &self.checksum_poly))
            }
            None => Err("These was no data to perform a checksum!"),
        }
    }
}
