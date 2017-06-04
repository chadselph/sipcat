extern crate futures;
extern crate tokio_core;
extern crate tokio_proto;
extern crate nom;
extern crate sipcat;

use std::str;

use futures::{Stream, Sink, stream};
use tokio_core::net::UdpSocket;
use tokio_core::reactor::Core;
use sipcat::protocol::{SipMessage, SipMethod, SipHeader};
use sipcat::codec::UdpSip;
use std::net::SocketAddr;


fn main() {
    println!("Hello, world!");
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let addr = "0.0.0.0:5060".parse().unwrap();
    let sock = UdpSocket::bind(&addr, &handle).unwrap();

    let (sink, stream) = sock.framed(UdpSip).split();
    let server = stream.filter_map(|(addr, msg)| {
        println!("{} says \t{:?}\n", addr, msg);
        match msg {
            SipMessage::SipRequest { headers, .. } => {
                let resp = SipMessage::SipResponse {
                    status_code: 200,
                    reason_phrase: "OK".into(),
                    headers: headers,
                };
                Option::Some((addr, resp))
            }
            other => Option::None,
        }
    });
    let sdp = "v=0
o=- 3696217174 3696217174 IN IP4 184.23.0.6
s=pjmedia
b=AS:117
t=0 0
a=X-nat:0
m=audio 4004 RTP/AVP 103 102 104 125 109 3 0 8 9 101
c=IN IP4 184.23.0.6
t=0 0
m=audio 49172 RTP/AVP 0 8
a=rtpmap:0 PCMU/8000
a=rtpmap:8 PCMA/8000
m=video 0 RTP/AVP 31
a=rtpmap:31 H261/90000
"
        .replace("\n", "\r\n");
    let number = "<sip:phone@chad.pstn.twilio.com>";
    let m: SipMessage<String> = SipMessage::invite("sip:chad@chad.sip.us1.twilio.com".to_owned(),
                                                   number.to_owned() as String,
                                                   //"\"Ya Boy\" <sip:chad@twilio.com>".to_owned() as String,
                                                   "\"some sexy dude\" \
                                                    <sip:chad@chad.sip.us1.twilio.com>;\
                                                    tag=eInv-ek28m4NFATrKyHQwKw2D7HaNSXQ"
                                                       .to_owned(),
                                                   "<sip:chad@184.23.0.7:15060>".to_owned() as
                                                   String,
                                                   sdp.to_owned() as String);

    // let sending = sink.send(
    println!("Sending {}",
             str::from_utf8(&sipcat::parser::write_sip_message(&m)).unwrap());
    let twilio_ip: SocketAddr = "54.172.60.2:5060".parse().unwrap();
    let server2 = stream::once(Ok((twilio_ip, m)));
    let sending = sink.send_all(server2.chain(server));
    core.run(sending).unwrap();
    ()

}
