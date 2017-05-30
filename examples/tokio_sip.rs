extern crate futures;
extern crate tokio_core;
extern crate tokio_proto;
extern crate nom;
extern crate sipcat;

use std::io;
use std::net::SocketAddr;
use std::str;
use std::vec::Vec;

use futures::{Stream, Sink};
use tokio_core::net::{UdpSocket, UdpCodec};
use tokio_core::reactor::Core;
use sipcat::protocol::{SipMessage, SipMethod};
use sipcat::parser::{parse_message, write_sip_message};
use nom::IResult;
use sipcat::codec::UdpSip;


fn main() {
    println!("Hello, world!");
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let addr = "127.0.0.1:5060".parse().unwrap();
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
