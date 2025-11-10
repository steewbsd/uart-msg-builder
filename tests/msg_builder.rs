extern crate uart_msg_builder;
use uart_msg_builder::message::Message;

fn crcfn(acc: u8, i: &u8) -> u8 {
    acc ^ i
}

#[test]
fn test_build_checksum() {
    let mut crc = crcfn;
    let msg = Message::build(&[1, 1, 1, 1], None, &mut crc).unwrap();
    assert_eq!(msg.get_checksum(), Ok(0));
}
