//
// Copyright 2019-2021 Signal Messenger, LLC
// SPDX-License-Identifier: AGPL-3.0-only
//

use std::{ffi::CString, os::raw::c_char, ptr::copy_nonoverlapping};

use crate::{
    webrtc,
    webrtc::{
        peer_connection_factory::{
            RffiAudioConfig, RffiAudioJitterBufferConfig, RffiIceServers, RffiPeerConnectionKind,
        },
        sim::{
            media::{
                RffiAudioTrack, RffiVideoSource, RffiVideoTrack, FAKE_AUDIO_TRACK,
                FAKE_VIDEO_SOURCE, FAKE_VIDEO_TRACK,
            },
            peer_connection::RffiPeerConnection,
            peer_connection_observer::RffiPeerConnectionObserver,
        },
    },
};

pub type RffiPeerConnectionFactoryOwner = u32;

impl webrtc::RefCounted for RffiPeerConnectionFactoryOwner {}

pub static FAKE_PEER_CONNECTION_FACTORY: RffiPeerConnectionFactoryOwner = 10;

pub type RffiPeerConnectionFactoryInterface = u32;

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe fn Rust_createPeerConnectionFactory(
    _audio_config: webrtc::ptr::Borrowed<RffiAudioConfig>,
    _use_injectable_network: bool,
) -> webrtc::ptr::OwnedRc<RffiPeerConnectionFactoryOwner> {
    info!("Rust_createPeerConnectionFactory()");
    webrtc::ptr::OwnedRc::from_ptr(&FAKE_PEER_CONNECTION_FACTORY)
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe fn Rust_createPeerConnectionFactoryWrapper(
    _interface: webrtc::ptr::BorrowedRc<RffiPeerConnectionFactoryInterface>,
) -> webrtc::ptr::OwnedRc<RffiPeerConnectionFactoryOwner> {
    panic!("no interface to wrap in sim!")
}

#[allow(non_snake_case, clippy::missing_safety_doc, clippy::too_many_arguments)]
pub unsafe fn Rust_createPeerConnection(
    _factory: webrtc::ptr::BorrowedRc<RffiPeerConnectionFactoryOwner>,
    _observer: webrtc::ptr::Borrowed<RffiPeerConnectionObserver>,
    _kind: RffiPeerConnectionKind,
    _audio_jitter_buffer_config: webrtc::ptr::Borrowed<RffiAudioJitterBufferConfig>,
    _audio_rtcp_report_interval_ms: i32,
    _ice_servers: webrtc::ptr::Borrowed<RffiIceServers>,
    _outgoing_audio_track: webrtc::ptr::BorrowedRc<RffiAudioTrack>,
    _outgoing_video_track: webrtc::ptr::BorrowedRc<RffiVideoTrack>,
) -> webrtc::ptr::OwnedRc<RffiPeerConnection> {
    info!("Rust_createPeerConnection()");
    webrtc::ptr::OwnedRc::from_ptr(Box::leak(Box::new(RffiPeerConnection::new())))
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe fn Rust_createAudioTrack(
    _factory: webrtc::ptr::BorrowedRc<RffiPeerConnectionFactoryOwner>,
) -> webrtc::ptr::OwnedRc<RffiAudioTrack> {
    info!("Rust_createVideoSource()");
    webrtc::ptr::OwnedRc::from_ptr(&FAKE_AUDIO_TRACK)
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe fn Rust_createVideoSource() -> webrtc::ptr::OwnedRc<RffiVideoSource> {
    info!("Rust_createVideoSource()");
    webrtc::ptr::OwnedRc::from_ptr(&FAKE_VIDEO_SOURCE)
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe fn Rust_createVideoTrack(
    _factory: webrtc::ptr::BorrowedRc<RffiPeerConnectionFactoryOwner>,
    _source: webrtc::ptr::BorrowedRc<RffiVideoSource>,
) -> webrtc::ptr::OwnedRc<RffiVideoTrack> {
    info!("Rust_createVideoTrack()");
    webrtc::ptr::OwnedRc::from_ptr(&FAKE_VIDEO_TRACK)
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe fn Rust_getAudioPlayoutDevices(
    _factory: webrtc::ptr::BorrowedRc<RffiPeerConnectionFactoryOwner>,
) -> i16 {
    1
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe fn Rust_getAudioPlayoutDeviceName(
    _factory: webrtc::ptr::BorrowedRc<RffiPeerConnectionFactoryOwner>,
    index: u16,
    name_out: *mut c_char,
    uuid_out: *mut c_char,
) -> i32 {
    if index != 0 {
        return -1;
    }
    copy_to_c_buffer("FakeSpeaker", name_out);
    copy_to_c_buffer("FakeSpeakerUuid", uuid_out);
    0
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe fn Rust_setAudioPlayoutDevice(
    _factory: webrtc::ptr::BorrowedRc<RffiPeerConnectionFactoryOwner>,
    index: u16,
) -> bool {
    index == 0
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe fn Rust_getAudioRecordingDevices(
    _factory: webrtc::ptr::BorrowedRc<RffiPeerConnectionFactoryOwner>,
) -> i16 {
    1
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe fn Rust_getAudioRecordingDeviceName(
    _factory: webrtc::ptr::BorrowedRc<RffiPeerConnectionFactoryOwner>,
    index: u16,
    name_out: *mut c_char,
    uuid_out: *mut c_char,
) -> i32 {
    if index != 0 {
        return -1;
    }
    copy_to_c_buffer("FakeMicrophone", name_out);
    copy_to_c_buffer("FakeMicrophoneUuid", uuid_out);
    0
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe fn Rust_setAudioRecordingDevice(
    _factory: webrtc::ptr::BorrowedRc<RffiPeerConnectionFactoryOwner>,
    index: u16,
) -> bool {
    index == 0
}

unsafe fn copy_to_c_buffer(string: &str, dest: *mut c_char) {
    let bytes = CString::new(string).unwrap();
    copy_nonoverlapping(bytes.as_ptr(), dest, string.len() + 1)
}
