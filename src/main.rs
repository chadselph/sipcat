extern crate futures;
extern crate tokio_core;
extern crate tokio_proto;
#[macro_use]
extern crate nom;
mod parser;

use std::io;
use std::net::SocketAddr;
use std::str;
use std::vec::Vec;

use futures::{Stream, Sink};
use tokio_core::net::{UdpSocket, UdpCodec};
use tokio_core::reactor::Core;
use parser::{SipMessage, SipMethod, parse_message, write_sip_message};
use nom::IResult;

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
        let mut bytes = write_sip_message(msg);
        into.append(&mut bytes);
        addr
    }
}

fn main() {
    println!("Hello, world!");
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let addr = "127.0.0.1:5060".parse().unwrap();
    /*
    let invite: SipRequest<String> = SipRequest {
        method: SipMethod::Invite,
        request_uri: "+15094302095@pstn.twilio.com".to_string(),
        headers: headers,
        body: sdp.to_string(),
    };
    */

    let sock = UdpSocket::bind(&addr, &handle).unwrap();

    let (sink, stream) = sock.framed(UdpSip).split();
    let server = stream.filter_map(|(addr, msg)| {
        println!("{} says {:?}", addr, msg);
        let resp = match msg {
            SipMessage::SipRequest { method: SipMethod::Register, headers, .. } => {
                SipMessage::SipResponse {
                    status_code: 200,
                    reason_phrase: "OK".into(),
                    headers: headers,
                }
            }
            _ => {
                SipMessage::SipResponse {
                    status_code: 404,
                    reason_phrase: "IDK".into(),
                    headers: vec![],
                }
            }

        };
        Some((addr, resp))
    });
    let sending = sink.send_all(server);
    //server.into_future());//.unwrap();
    core.run(sending).unwrap();
    ()

}
