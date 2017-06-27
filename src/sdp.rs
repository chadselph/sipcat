
pub struct MediaDescription {
    media_name: String,
    media_title: Option<String>,
    connection_information: Option<String>,
    bandwidth_information: Vec<String>,
    encryption_key: Option<String>,
    attributes: Vec<String>,
}

pub struct Sdp {
    version: u8,
    originator: String, // todo better types!
    session_name: String,
    session_title: Option<String>,
    description_uri: Option<String>,
    email_addresses: Vec<String>,
    phone_numbers: Vec<String>,
    connection_information: Option<String>,
    bandwidth_information: Vec<String>,
    time_active: (u64, u64),
    repeat_times: Vec<String>,
    zone_adjustment: Option<String>,
    encryption_key: Option<String>,
    attributes: Vec<String>,
    media_descriptions: Vec<MediaDescription>,
}

impl Sdp {
    pub fn new(rtp_port: u16) -> String {

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
            sdp
    }
}