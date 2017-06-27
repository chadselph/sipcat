extern crate futures;
extern crate tokio_core;
extern crate tokio_proto;
extern crate nom;
extern crate sipcat;
extern crate clap;
extern crate ansi_term;

use std::str;

use futures::{Stream, Sink, stream};
use tokio_core::net::UdpSocket;
use tokio_core::reactor::Core;
use sipcat::protocol::{SipMessage, SipMethod};
use sipcat::codec::UdpSip;
use clap::{Arg, App};


fn main() {

    let matches = App::new("sipcat")
        .version("0.0")
        .about("Send a sip invite")
        .arg(Arg::with_name("bind address")
            .default_value("0.0.0.0:5060")
            .takes_value(true)
            .help("address and UDP port to bind on for SIP")
            .long("bind")
            .short("b"))
        .arg(Arg::with_name("rtp address")
            .default_value("0.0.0.0:4004")
            .takes_value(true)
            .help("Address and udp port to bind on for RTP")
            .long("rtp-addr"))
        .arg(Arg::with_name("send to address")
            .default_value("54.172.60.2:5060")
            .takes_value(true)
            .help("Force the SIP request to a specific server (should eventually use the URI to \
                   determine this)")
            .long("send-to"))
        .arg(Arg::with_name("request uri")
            .help("SIP Address the message will be directed to")
            .takes_value(true)
            .long("uri")
            .required(true))
        .arg(Arg::with_name("to")
            .help("SIP address in the To: header")
            .takes_value(true)
            .required(true)
            .long("to"))
        .arg(Arg::with_name("from")
            .help("SIP address in the To: header")
            .takes_value(true)
            .required(true)
            .long("from"))
        .arg(Arg::with_name("contact")
            .takes_value(true)
            .help("SIP address in the To: header")
            .required(true)
            .long("contact"))
        .arg(Arg::with_name("verbose")
            .help("Show all SIP messages")
            .long("verbose")
            .short("v"))
        .get_matches();

    fn debug_sip(msg: &SipMessage<String>) -> () {
        println!("{}",
                 str::from_utf8(&sipcat::parser::write_sip_message(msg)).unwrap());

    }

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let bind_addr = matches.value_of("bind address")
        .and_then(|b| b.parse().ok())
        .unwrap_or_else(|| {
            panic!("Invalid sip bind address, should look like 0.0.0.0:5060");
        });
    let rtp_addr = matches.value_of("rtp address")
        .and_then(|b| b.parse().ok())
        .unwrap_or_else(|| {
            panic!("Invalid rtp bind address, should look like 0.0.0.0:4004");
        });
    let sock = UdpSocket::bind(&bind_addr, &handle).unwrap();
    let rtp_sock = UdpSocket::bind(&rtp_addr, &handle).unwrap();

    let (sink, stream) = sock.framed(UdpSip).split();
    let server = stream.filter_map(|(addr, msg)| {
            println!("got from {}:", addr);
            debug_sip(&msg);
            match msg {
                SipMessage::SipRequest { headers, .. } => {
                    let resp = SipMessage::SipResponse {
                        status_code: 200.into(),
                        reason_phrase: "OK".into(),
                        headers: headers,
                        body: "".into(),
                    };
                    Option::Some((addr, resp))
                }
                SipMessage::SipResponse { ref status_code, .. } if status_code.is_final() => {
                    // TODO: fix this crap
                    let mut headers: Vec<sipcat::protocol::SipHeader> = ["Call-ID", "To", "From"]
                        .iter()
                        .filter_map(|name| {
                            // yeah there must be a better way
                            msg.header(name.to_owned().to_owned())
                        })
                        .map(|ref x| {
                            sipcat::protocol::SipHeader::Header(x.name().into(), x.value().into())
                        })
                        .collect();
                    headers.push(sipcat::protocol::SipHeader::StaticHeader("Max-Forwards",
                                                                           "70".into()));
                    headers.push(sipcat::protocol::SipHeader::StaticHeader("CSeq",
                                                                           "12345 ACK".into()));
                    headers.push(sipcat::protocol::SipHeader::StaticHeader("Content-Length",
                                                                           "0".into()));
                    Option::Some((addr,
                                  SipMessage::SipRequest {
                                      method: SipMethod::Ack,
                                      request_uri: msg.header("Contact".into())
                                          .unwrap()
                                          .value()
                                          .into(),
                                      headers: headers,
                                      body: "".into(),
                                  }))
                }
                _other => Option::None,
            }
        })
        .map(|addr_msg| {
            let (addr, msg) = addr_msg;
            println!("Sending to {}", addr);
            debug_sip(&msg);
            (addr, msg)
        });
    let sdp = "v=0
o=- 3704930061 3704930061 IN IP4 192.168.42.81
s=pjmedia
b=AS:117
t=0 0
a=X-nat:0
m=audio 4004 RTP/AVP 0 103 102 104 125 109 3 8 9 101
c=IN IP4 192.168.42.81
b=TIAS:96000
a=rtcp:4005 IN IP4 192.168.42.81
a=sendrecv
a=rtpmap:0 PCMU/8000
a=rtpmap:103 speex/16000
a=rtpmap:102 speex/8000
a=rtpmap:104 speex/32000
a=rtpmap:125 opus/48000/2
a=fmtp:125 useinbandfec=1
a=rtpmap:109 iLBC/8000
a=fmtp:109 mode=30
a=rtpmap:3 GSM/8000
a=rtpmap:8 PCMA/8000
a=rtpmap:9 G722/8000
a=rtpmap:101 telephone-event/8000
a=fmtp:101 0-16
"
        .replace("\n", "\r\n");
    let m: SipMessage<String> = SipMessage::invite(matches.value_of("request uri").unwrap(),
                                                   matches.value_of("to").unwrap(),
                                                   matches.value_of("from").unwrap(),
                                                   matches.value_of("contact").unwrap(),
                                                   sdp.to_owned() as String);

    let addr = matches.value_of("send to address")
        .and_then(|b| b.parse().ok())
        .unwrap_or_else(|| {
            panic!("Invalid bind address, should look like 0.0.0.0:5060");
        });
    debug_sip(&m);

    let server2 = stream::once(Ok((addr, m)));
    let sending = sink.send_all(server2.chain(server));
    core.run(sending).unwrap();
    ()

}
