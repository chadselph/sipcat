# sipcat

Work in progress command line SIP client in Rust.

I'm working on this to learn Rust and to learn SIP.
You probably don't want to use any code from here,
as I don't really know Rust or SIP (or RTP/SDP/related technologies)

# What works

* Listening on UDP and doing very primitive parsing of SIP responses / requests.

# TODO

* Fix parsing of multi-line headers
* Routing. Currently it always sends to the UDP socket it got the message
  from, this isn't how SIP routing should work.
* SDP. A sub-module that generates new SDPs and negotiates between two SDPs.
* Media / RTP
* Will probably need some type of state machine to represent a call status.