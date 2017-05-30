#![allow(dead_code)]
extern crate nom;
use std::str;
use nom::{line_ending, not_line_ending, rest};
use std::io::Write;


use protocol::{SipMethod, SipHeader, SipMessage};
use protocol::SipMethod::*;

named!(parse_method<&[u8], SipMethod>, alt!(
    map!(tag!("INVITE"), { |_| Invite }) |
    map!(tag!("ACK"), { |_| Ack }) |
    map!(tag!("BYE"), { |_| Bye }) |
    map!(tag!("CANCEL"), { |_| Cancel }) |
    map!(tag!("REGISTER"), { |_| Register }) |
    map!(tag!("OPTIONS"), { |_| Options }) |
    map!(tag!("PRACK"), { |_| Prack }) |
    map!(tag!("SUBSCRIBE"), { |_| Subscribe }) |
    map!(tag!("NOTIFY"), { |_| Notify }) |
    map!(tag!("PUBLISH"), { |_| Publish }) |
    map!(tag!("INFO"), { |_| Info }) |
    map!(tag!("REFER"), { |_| Refer }) |
    map!(tag!("MESSAGE"), { |_| Message }) |
    map!(tag!("UPDATE"), { |_| Update })
    )
);

// TODO: write a real sip uri parser
named!(parse_uri<&[u8], &str>, map_res!(take_until!(" "), str::from_utf8));

fn status_reason_char(c: u8) -> bool {
    let chars = vec!['-', '.', '!', '%', '*', '_', '+', '`', '\'', '~', ' '];
    nom::is_alphanumeric(c) || chars.contains(&(c as char))
}

named!(parse_reason_phrase<&str>, map_res!(take_while1!(status_reason_char), str::from_utf8));

// TODO?: versions
named!(parse_version, tag!("SIP/2.0"));

named!(parse_keep_alives<SipMessage<String> >, alt_complete!(
    map!(tag!("\r\n\r\n"), { |_| SipMessage::ClientKeepAlive}) |
    map!(tag!("\r\n"), { |_| SipMessage::ServerKeepAlive })
));

named!(pub parse_message<SipMessage<String> >, alt!(
    parse_request | parse_response | parse_keep_alives
));

named!(parse_request<&[u8], SipMessage<String> >,
       do_parse!(
           method: parse_method >>
           tag!(" ") >>
           uri: parse_uri >>
           tag!(" ") >>
           version: parse_version >>
           line_ending >>
           headers: separated_nonempty_list!(line_ending, parse_header) >>
           opt!(line_ending) >>
           opt!(line_ending) >>
           body: map_res!(rest, str::from_utf8) >>
           ( SipMessage::SipRequest {
               method: method,
               request_uri: uri.into(),
               headers: headers,
               body: body.into(), 
           })
));

fn parse_response_code(bytes: &[u8]) -> nom::IResult<&[u8], u16> {
    nom::digit(bytes).map({
        |_| (bytes[0] as u16 - 48) * 100 + (bytes[1] as u16 - 48) * 10 + (bytes[2] as u16 - 48)
    })
}

named!(parse_response<&[u8], SipMessage<String> >,
    do_parse!(
        version: parse_version >>
        tag!(" ") >>
        code: parse_response_code >>
        tag!(" ") >>
        reason: parse_reason_phrase >>
        line_ending >>
        headers: separated_nonempty_list!(line_ending, parse_header) >>
        (
            SipMessage::SipResponse {
                status_code: code,
                reason_phrase: reason.into(),
                headers: headers
            }
        )
    )
);

fn is_sip_token_char(c: u8) -> bool {
    let chars = vec!['-', '.', '!', '%', '*', '_', '+', '`', '\'', '~'];
    nom::is_alphanumeric(c) || chars.contains(&(c as char))
}

named!(token, take_while1!(is_sip_token_char));

named!(parse_header<&[u8], SipHeader>,
       do_parse!(
    name: map_res!(token, str::from_utf8) >>
    ws!(char!(':')) >>
    value: map_res!(not_line_ending, str::from_utf8) >>
    (SipHeader::Header(name.into(), value.into()))
));

/// more of a serializer than a parser, shouldn't be in this module probably
/// but this is just where it lives for now.
/// hoping to learn how to do this more efficiently.
/// Once my SIP objects are just wrapping &[u8]s it should be
/// a lot nicer!
#[allow(unused_must_use)]
pub fn write_sip_message(m: SipMessage<String>) -> Vec<u8> {
    // TODO: how do I write an object to bytes the proper way? Also, would like less duplication.
    match m {
        SipMessage::SipRequest { method, request_uri, headers, body } => {

            let mut vec: Vec<u8> = Vec::new();
            vec.write(&format!("{}", method).into_bytes());
            vec.push(0x20u8);
            vec.write(&request_uri.into_bytes());
            vec.write(b"\r\n");
            for h in headers {
                match h {
                    SipHeader::Header(name, value) => {
                        vec.write(&name.into_bytes());
                        vec.write(b": ");
                        vec.write(&value.into_bytes());
                        vec.write(b"\r\n");
                        ()
                    }
                    _ => (),
                }
            }
            vec.write(b"\r\n");
            vec.write(&body.into_bytes());
            vec
        }

        SipMessage::SipResponse { status_code, reason_phrase, headers } => {
            let mut vec: Vec<u8> = Vec::new();
            vec.write(b"SIP/2.0 ");
            vec.write(&format!("{}", status_code).into_bytes());
            vec.write(&reason_phrase.into_bytes());
            vec.write(b"\r\n");
            for h in headers {
                match h {
                    SipHeader::Header(name, value) => {
                        vec.write(&name.into_bytes());
                        vec.write(b": ");
                        vec.write(&value.into_bytes());
                        vec.write(b"\r\n");
                        ()
                    }
                    _ => (),
                }
            }
            vec.write(b"\r\n");
            // TODO: body
            vec
        }
        SipMessage::ClientKeepAlive => vec![13, 10, 13, 10],
        SipMessage::ServerKeepAlive => vec![13, 10],
    }
}

#[cfg(test)]
mod tests {

    use parser::{parse_response_code, token, parse_response, parse_request, parse_header};
    use nom;
    use nom::line_ending;
    use std::fmt::Debug;

    fn assert_parsed_equal<T: Debug + PartialEq>(parse_result: nom::IResult<&[u8], T>,
                                                 expected: T)
                                                 -> () {
        assert_eq!(parse_result.unwrap().1, expected);
    }

    use protocol::*;

    #[test]
    fn test_parse_headers() {
        println!("{:?}", parse_header(b"Max-Forwards: 70\r\n").unwrap());

        named!(headers<&[u8], Vec<SipHeader> >, separated_nonempty_list!(line_ending, parse_header));
        let twoheaders = b"Header1: Value1\r\nHeader2: Value2\r\nHeader3: Value3\n\n";
        println!("{:?}", headers(twoheaders))
    }

    #[test]
    fn parse_sip_string() {
        let sip_example1 = b"INVITE sip:user2@server2.com SIP/2.0
Via: SIP/2.0/UDP pc33.server1.com;branch=z9hG4bK776asdhds
Max-Forwards: 70
To: user2 <sip:user2@server2.com>
From: user1 <sip:user1@server1.com>;tag=1928301774
Call-ID: a84b4c76e66710@pc33.server1.com
CSeq: 314159 INVITE
Contact: <sip:user1@pc33.server1.com>
Content-Type: application/sdp
Content-Length: 142

v=0
o=bob 2808844564 2808844564 IN IP4 host.biloxi.example.com
s=
c=IN IP4 host.biloxi.example.com
t=0 0
m=audio 49172 RTP/AVP 0 8
a=rtpmap:0 PCMU/8000
a=rtpmap:8 PCMA/8000
m=video 0 RTP/AVP 31
a=rtpmap:31 H261/90000";

        println!("{:?}", parse_request(sip_example1));
        parse_request(sip_example1).unwrap();
        ()

    }

    #[test]
    fn telephone_register() {
        let sip = b"REGISTER sip:localhost SIP/2.0
Via: SIP/2.0/UDP 192.168.42.81:64095;rport;branch=z9hG4bKPjQc.VtxSKdY69oZCAxCA9B-vuS2EkAbk.
Max-Forwards: 70
From: \"Chad S\" <sip:chad@localhost>;tag=lVtowISouUTBVz9HkEaORARHOh3qZZd1
To: \"Chad S\" <sip:chad@localhost>
Call-ID: 6wn2XXLNyU-xtFKKIJVwKf7vp3vBTu5m
CSeq: 40619 REGISTER
User-Agent: Telephone 1.2.6
Contact: \"Chad S\" <sip:chad@192.168.42.81:64095;ob>
Expires: 300
Allow: PRACK, INVITE, ACK, BYE, CANCEL, UPDATE, INFO, SUBSCRIBE, NOTIFY, REFER, MESSAGE, OPTIONS
Content-Length:  0

";
        println!("{:?}", parse_request(sip));
        println!("{:?}", parse_header(b"Via: SIP/2.0/UDP 192.168.42.81:64095;rport;branch=z9hG4bKPjQc.VtxSKdY69oZCAxCA9B-vuS2EkAbk."));
        parse_request(sip).unwrap();
        ()

    }

    #[test]
    fn test_separated_nonempty_list() {
        use nom::IResult::Done;
        named!(test_parser<&[u8], Vec<&[u8]> >, separated_nonempty_list!(line_ending, token));
        let data = b"token1\nhello\ntoken3\n\n";
        println!("{:?}", test_parser(data));
        assert_eq!(test_parser(data), Done(&b"\n\n"[..], vec![&b"token1"[..], &b"hello"[..], &b"token3"[..]]))
    }

    #[test]
    fn test_response() {
        let resp = b"SIP/2.0 200 OK
Via: SIP/2.0/UDP site4.server2.com;branch=z9hG4bKnashds8;received=192.0.2.3
To: user2 <sip:user2@server2.com>;tag=a6c85cf
From: user1 <sip:user1@server1.com>;tag=1928301774
Call-ID: a84b4c76e66710@pc33.server1.com
CSeq: 314159 INVITE
Contact: <sip:user2@192.0.2.4>
Content-Type: application/sdp

";

        fn h(name: &str, value: &str) -> SipHeader {
            SipHeader::Header(name.into(), value.into())
        }
        assert_parsed_equal(parse_response(resp),
                            SipMessage::SipResponse {
                                status_code: 200,
                                reason_phrase: "OK".into(),
                                headers: vec![
                h("Via", "SIP/2.0/UDP site4.server2.com;branch=z9hG4bKnashds8;received=192.0.2.3"),
                h("To", "user2 <sip:user2@server2.com>;tag=a6c85cf"),
                h("From", "user1 <sip:user1@server1.com>;tag=1928301774"),
                h("Call-ID", "a84b4c76e66710@pc33.server1.com"),
                h("CSeq","314159 INVITE"),
                h("Contact", "<sip:user2@192.0.2.4>"),
                h("Content-Type", "application/sdp")
                ],
                            })

    }

    #[test]
    fn test_parse_resp_code() {
        assert_parsed_equal(parse_response_code(b"200"), 200);
        assert_parsed_equal(parse_response_code(b"999"), 999);
        assert_parsed_equal(parse_response_code(b"491"), 491);
        assert_parsed_equal(parse_response_code(b"101"), 101);
        assert_parsed_equal(parse_response_code(b"667"), 667);
        assert_eq!(parse_response_code(b"bad").is_err(), true)
    }
}