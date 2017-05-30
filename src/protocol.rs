use std::fmt;

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
    // there's too many of these...
    Header(String, String),
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