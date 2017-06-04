use std::fmt;
use uuid::Uuid;

#[derive(Debug, PartialEq)]
pub enum SipMethod {
    Invite,
    Ack,
    Bye,
    Cancel,
    Register,
    Options,
    Prack,
    Subscribe,
    Notify,
    Publish,
    Info,
    Refer,
    Message,
    Update,
}

impl fmt::Display for SipMethod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let capitalized = format!("{:?}", *self).to_uppercase();
        write!(f, "{}", capitalized)
    }
}

#[derive(Debug, PartialEq)]
pub enum SipHeader {
    /*
    Allow(String),
    CallID(String),
    Concact(String),
    CSeq(String),
    From(String),
    MaxForwards(u8),
    Route(String),
    To(String),
    UserAgent(String),
    Via(String),
    */
    // there's too many of these...
    Header(String, String),
    StaticHeader(&'static str, String),
}

pub trait SipUri {
    fn value(self) -> String;
}

impl SipUri for String {
    fn value(self) -> String {
        self
    }
}

pub trait SipMessageBody {
    fn length(&self) -> u32;
    fn content_type() -> &'static str;
    fn headers(&self) -> Vec<SipHeader> {
        vec![
            SipHeader::ContentType(Self::content_type().into()),
            SipHeader::ContentLength(self.length())
        ]
    }
}

impl SipMessageBody for String {
    fn length(&self) -> u32 {
        self.len() as u32
    }

    fn content_type() -> &'static str {
        "application/sdp"
    }
}

use self::SipHeader::StaticHeader;

#[allow(non_snake_case, dead_code)]
impl SipHeader {
    pub fn Allow(value: String) -> SipHeader {
        StaticHeader("Allow", value)
    }
    pub fn CallID(value: String) -> SipHeader {
        StaticHeader("Call-ID", value)
    }
    pub fn Contact<T: SipUri>(value: T) -> SipHeader {
        StaticHeader("Contact", value.value())
    }
    pub fn ContentLength(value: u32) -> SipHeader {
        StaticHeader("Content-Length", format!("{}", value))
    }
    pub fn ContentType(value: String) -> SipHeader {
        StaticHeader("Content-Type", value)
    }
    pub fn CSeq(value: String) -> SipHeader {
        StaticHeader("CSeq", value)
    }
    pub fn From<T: SipUri>(value: T) -> SipHeader {
        StaticHeader("From", value.value())
    }
    pub fn MaxForwards(value: u8) -> SipHeader {
        StaticHeader("Max-Forwards", format!("{}", value))
    }
    pub fn Route(value: String) -> SipHeader {
        StaticHeader("Route", value)
    }
    pub fn To<T: SipUri>(value: T) -> SipHeader {
        StaticHeader("To", value.value())
    }
    pub fn UserAgent(value: String) -> SipHeader {
        StaticHeader("User-Agent", value)
    }
    pub fn Via(value: String) -> SipHeader {
        StaticHeader("Via", value)
    }
}

#[derive(Debug, PartialEq)]
pub enum SipMessage<T> {
    SipRequest {
        method: SipMethod,
        // TODO: make this a &str after I understand lifetimes
        request_uri: String,
        headers: Vec<SipHeader>,
        body: T,
    },
    SipResponse {
        status_code: u16,
        reason_phrase: String,
        headers: Vec<SipHeader>,
    },
    // Not sure about the names of these
    ClientKeepAlive,
    ServerKeepAlive,
}

impl<A: SipMessageBody> SipMessage<A> {
    #[allow(dead_code)]
    pub fn invite<UTo: SipUri, UFrom: SipUri>(request_uri: UTo,
                                              to: UTo,
                                              from: UFrom,
                                              contact: UFrom,
                                              body: A)
                                              -> SipMessage<A> {
        let call_id = format!("{}", Uuid::new_v4());
        let mut headers = vec![
             SipHeader::Via("SIP/2.0/UDP 184.23.0.7:15060;rport;branch=1234".into()),
             SipHeader::MaxForwards(70),
             SipHeader::From(from),
            SipHeader::To(to),
             SipHeader::Contact(contact),
             SipHeader::CallID(call_id),
             SipHeader::CSeq("12345 INVITE".into()),
             SipHeader::Allow("ACK INVITE REGISTER BYE OPTIONS".into()),
             SipHeader::UserAgent("sipcat 0.0".into()),
             ];
        let body_headers = body.headers();
        headers.extend(body_headers);
             SipMessage::SipRequest {
                 method: SipMethod::Invite,
                 body: body,
                 headers: headers,
                 request_uri: request_uri.value()
             }
    }
}