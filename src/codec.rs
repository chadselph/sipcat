extern crate tokio_core;
extern crate tokio_proto;

use std::io;
use std::net::SocketAddr;
use std::str;
use std::vec::Vec;

use tokio_core::net::UdpCodec;
use protocol::SipMessage;
use parser::{parse_message, write_sip_message};
use nom::IResult;

// tokio "codecs" using the sip parser and serializer
pub struct UdpSip;

impl UdpCodec for UdpSip {
    type In = (SocketAddr, SipMessage<String>);
    type Out = (SocketAddr, SipMessage<String>);

    fn decode(&mut self, src: &SocketAddr, buf: &[u8]) -> io::Result<Self::In> {
        // TODO: this won't work with fragmented UDP packets
        match parse_message(buf) {
            IResult::Done(_, req) => Ok((*src, req)),
            IResult::Error(_) => {
                Err(io::Error::new(io::ErrorKind::InvalidInput,
                                   format!("bad message: {:?}", buf)))
            }
            IResult::Incomplete(_) => {
                Err(io::Error::new(io::ErrorKind::BrokenPipe,
                                   format!("incomplete message: {:?}", buf)))
            }
        }
    }

    fn encode(&mut self, (addr, msg): Self::Out, into: &mut Vec<u8>) -> SocketAddr {
        let bytes = write_sip_message(&msg);
        into.extend(bytes);
        addr
    }
}