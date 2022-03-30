/// A serialized one these goes in the "opaque" field of the CallingMessage::Offer in SignalService.proto
/// For future compatibility, we can add new slots (v5, v6, ...)
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Offer {
    #[prost(message, optional, tag="4")]
    pub v4: ::core::option::Option<ConnectionParametersV4>,
}
/// A serialized one these goes in the "opaque" field of the CallingMessage::Offer in SignalService.proto
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Answer {
    #[prost(message, optional, tag="4")]
    pub v4: ::core::option::Option<ConnectionParametersV4>,
}
/// A serialized one these goes in the "opaque" field of the CallingMessage::Ice in SignalService.proto
/// Unlike other message types, the ICE message contains many of these, not just one.
/// We should perhaps rename this to "IceUpdate" since it can either be a candidate
/// or a removal of a candidate.  But it would require a lot of FFI code to be renamed
/// which doesn't seem worth it at the moment.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IceCandidate {
    /// Use a field value of 2 for compatibility since both V2 and V3 have the same format.
    #[prost(message, optional, tag="2")]
    pub added_v3: ::core::option::Option<IceCandidateV3>,
    /// ICE candidate removal identifies the removed candidate
    /// by (transport_name, component, ip, port, udp/tcp).
    /// But we assume transport_name = "audio", component = 1, and udp
    /// So we just need (ip, port)
    #[prost(message, optional, tag="3")]
    pub removed: ::core::option::Option<SocketAddr>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IceCandidateV3 {
    #[prost(string, optional, tag="1")]
    pub sdp: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SocketAddr {
    /// IPv4: 4 bytes; IPv6: 16 bytes
    #[prost(bytes="vec", optional, tag="1")]
    pub ip: ::core::option::Option<::prost::alloc::vec::Vec<u8>>,
    #[prost(uint32, optional, tag="2")]
    pub port: ::core::option::Option<u32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VideoCodec {
    #[prost(enumeration="VideoCodecType", optional, tag="1")]
    pub r#type: ::core::option::Option<i32>,
    /// Used for H264; Not used for VP8 or VP9
    #[prost(uint32, optional, tag="2")]
    pub level: ::core::option::Option<u32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConnectionParametersV4 {
    #[prost(bytes="vec", optional, tag="1")]
    pub public_key: ::core::option::Option<::prost::alloc::vec::Vec<u8>>,
    #[prost(string, optional, tag="2")]
    pub ice_ufrag: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag="3")]
    pub ice_pwd: ::core::option::Option<::prost::alloc::string::String>,
    /// In other words, the video codecs the sender can receive.
    #[prost(message, repeated, tag="4")]
    pub receive_video_codecs: ::prost::alloc::vec::Vec<VideoCodec>,
    /// Used at call establishment to convey the bitrate that should be used for sending.
    #[prost(uint64, optional, tag="5")]
    pub max_bitrate_bps: ::core::option::Option<u64>,
}
/// A generic calling message that is opaque to the application but interpreted by RingRTC.
/// A serialized one of these goes into the "Opaque" field in the CallingMessage variant
/// in Signal protocol messages.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CallMessage {
    #[prost(message, optional, tag="1")]
    pub group_call_message: ::core::option::Option<super::group_call::DeviceToDevice>,
    #[prost(message, optional, tag="2")]
    pub ring_intention: ::core::option::Option<call_message::RingIntention>,
    #[prost(message, optional, tag="3")]
    pub ring_response: ::core::option::Option<call_message::RingResponse>,
}
/// Nested message and enum types in `CallMessage`.
pub mod call_message {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct RingIntention {
        #[prost(bytes="vec", optional, tag="1")]
        pub group_id: ::core::option::Option<::prost::alloc::vec::Vec<u8>>,
        #[prost(enumeration="ring_intention::Type", optional, tag="2")]
        pub r#type: ::core::option::Option<i32>,
        /// This is signed so it fits in a SQLite integer column.
        #[prost(sfixed64, optional, tag="3")]
        pub ring_id: ::core::option::Option<i64>,
    }
    /// Nested message and enum types in `RingIntention`.
    pub mod ring_intention {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
        #[repr(i32)]
        pub enum Type {
            Ring = 0,
            Cancelled = 1,
        }
    }
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct RingResponse {
        #[prost(bytes="vec", optional, tag="1")]
        pub group_id: ::core::option::Option<::prost::alloc::vec::Vec<u8>>,
        #[prost(enumeration="ring_response::Type", optional, tag="2")]
        pub r#type: ::core::option::Option<i32>,
        /// This is signed so it fits in a SQLite integer column.
        #[prost(sfixed64, optional, tag="3")]
        pub ring_id: ::core::option::Option<i64>,
    }
    /// Nested message and enum types in `RingResponse`.
    pub mod ring_response {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
        #[repr(i32)]
        pub enum Type {
            Ringing = 0,
            Accepted = 1,
            Declined = 2,
            Busy = 3,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum VideoCodecType {
    Vp8 = 8,
    Vp9 = 9,
    H264ConstrainedBaseline = 40,
    H264ConstrainedHigh = 46,
}
