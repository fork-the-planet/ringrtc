//
// Copyright 2019-2021 Signal Messenger, LLC
// SPDX-License-Identifier: AGPL-3.0-only
//

//! Simulation CallPlatform Interface.

use std::{
    collections::{HashMap, HashSet},
    fmt,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use crate::{
    common::{
        ApplicationEvent, CallConfig, CallDirection, CallId, CallMediaType, DeviceId, Result,
    },
    core::{
        call::Call,
        call_manager::CallManager,
        connection::{Connection, ConnectionType},
        group_call,
        platform::{Platform, PlatformItem},
        signaling,
    },
    lite::{
        sfu,
        sfu::{DemuxId, PeekInfo, PeekResult, UserId},
    },
    sim::error::SimError,
    webrtc::{
        media::{MediaStream, VideoTrack},
        peer_connection::{AudioLevel, PeerConnection, ReceivedAudioLevel},
        peer_connection_observer::NetworkRoute,
        sim::peer_connection::RffiPeerConnection,
    },
};

/// Simulation implementation for platform::Platform::{AppIncomingMedia,
/// AppRemotePeer, AppCallContext}
type SimPlatformItem = String;
impl PlatformItem for SimPlatformItem {}

#[derive(Default)]
struct SimStats {
    /// Number of offers sent to the client
    offers_sent: AtomicUsize,
    /// Number of answers sent to the client
    answers_sent: AtomicUsize,
    /// Number of ICE candidates sent to the client
    ice_candidates_sent: AtomicUsize,
    /// Number of normal hangups sent to the client
    normal_hangups_sent: AtomicUsize,
    /// Number of accepted hangups sent to the client
    accepted_hangups_sent: AtomicUsize,
    /// Number of declined hangups sent to the client
    declined_hangups_sent: AtomicUsize,
    /// Number of busy hangups sent to the client
    busy_hangups_sent: AtomicUsize,
    /// Number of need permission hangups sent to the client
    need_permission_hangups_sent: AtomicUsize,
    /// Number of busy messages sent to the client
    busys_sent: AtomicUsize,
    /// Number of start outgoing call events
    start_outgoing: AtomicUsize,
    /// Number of start incoming call events
    start_incoming: AtomicUsize,
    /// Number of offer expired events
    offer_expired: AtomicUsize,
    /// Number of call concluded events
    call_concluded: AtomicUsize,
    /// Track stream counts
    stream_count: AtomicUsize,
}

#[derive(Debug, PartialEq, Eq)]
pub struct GroupCallRingUpdate {
    pub group_id: group_call::GroupId,
    pub ring_id: group_call::RingId,
    pub sender_id: UserId,
    pub update: group_call::RingUpdate,
}

#[derive(Debug, PartialEq, Eq)]
pub struct OutgoingCallMessage {
    pub recipient_id: UserId,
    pub message: Vec<u8>,
    pub urgency: group_call::SignalingMessageUrgency,
}

/// Simulation implementation of platform::Platform.
#[derive(Clone, Default)]
pub struct SimPlatform {
    /// Platform API statistics
    stats: Arc<SimStats>,
    /// True if the CallPlatform functions should simulate an internal failure.
    force_internal_fault: Arc<AtomicBool>,
    /// True if the signaling functions should indicate a signaling
    /// failure to the call manager.
    force_signaling_failure: Arc<AtomicBool>,
    /// Track event frequencies
    event_map: Arc<Mutex<HashMap<ApplicationEvent, usize>>>,
    /// Track whether disconnecting of incoming media happened
    incoming_media_disconnected: Arc<AtomicBool>,
    /// Track group call ring updates
    group_call_ring_updates: Arc<Mutex<Vec<GroupCallRingUpdate>>>,
    /// Track outgoing opaque messages
    outgoing_call_messages: Arc<Mutex<Vec<OutgoingCallMessage>>>,
    /// Call Manager
    call_manager: Arc<Mutex<Option<CallManager<Self>>>>,
    /// True to manually require message_sent() to be invoked for Ice messages.
    no_auto_message_sent_for_ice: Arc<AtomicBool>,
    /// Last sent message from on_send_ice
    last_ice_sent: Arc<Mutex<Option<signaling::SendIce>>>,
}

impl fmt::Display for SimPlatform {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SimPlatform")
    }
}

impl fmt::Debug for SimPlatform {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Drop for SimPlatform {
    fn drop(&mut self) {
        info!("Dropping SimPlatform");
    }
}

impl Platform for SimPlatform {
    type AppIncomingMedia = SimPlatformItem;
    type AppRemotePeer = SimPlatformItem;
    type AppConnection = RffiPeerConnection;
    type AppCallContext = SimPlatformItem;

    fn create_connection(
        &mut self,
        call: &Call<Self>,
        remote_device_id: DeviceId,
        connection_type: ConnectionType,
        signaling_version: signaling::Version,
        call_config: CallConfig,
        audio_levels_interval: Option<Duration>,
    ) -> Result<Connection<Self>> {
        info!(
            "create_connection(): call_id: {} remote_device_id: {}, signaling_version: {:?}",
            call.call_id(),
            remote_device_id,
            signaling_version,
        );

        let fake_pc = RffiPeerConnection::new();

        let connection = Connection::new(
            call.clone(),
            remote_device_id,
            connection_type,
            call_config,
            audio_levels_interval,
            None,
        )
        .unwrap();
        connection.set_app_connection(fake_pc).unwrap();

        let peer_connection = PeerConnection::new(connection.peer_connection_rffi(), None, None);

        connection.set_peer_connection(peer_connection).unwrap();

        Ok(connection)
    }

    fn on_start_call(
        &self,
        remote_peer: &Self::AppRemotePeer,
        call_id: CallId,
        direction: CallDirection,
        call_media_type: CallMediaType,
    ) -> Result<()> {
        info!(
            "on_start_call(): remote_peer: {}, call_id: {}, direction: {}, call_media_type {}",
            remote_peer, call_id, direction, call_media_type
        );

        if self.force_internal_fault.load(Ordering::Acquire) {
            Err(SimError::StartCallError.into())
        } else {
            let _ = match direction {
                CallDirection::Outgoing => self.stats.start_outgoing.fetch_add(1, Ordering::AcqRel),
                CallDirection::Incoming => self.stats.start_incoming.fetch_add(1, Ordering::AcqRel),
            };
            Ok(())
        }
    }

    fn on_event(
        &self,
        remote_peer: &Self::AppRemotePeer,
        _call_id: CallId,
        event: ApplicationEvent,
    ) -> Result<()> {
        info!("on_event(): {}, remote_peer: {}", event, remote_peer);

        let mut map = self.event_map.lock().unwrap();
        map.entry(event).and_modify(|e| *e += 1).or_insert(1);

        Ok(())
    }

    fn on_network_route_changed(
        &self,
        _remote_peer: &Self::AppRemotePeer,
        network_route: NetworkRoute,
    ) -> Result<()> {
        info!("on_network_route_changed(): {:?}", network_route);
        Ok(())
    }

    fn on_audio_levels(
        &self,
        _remote_peer: &Self::AppRemotePeer,
        captured_level: AudioLevel,
        received_level: AudioLevel,
    ) -> Result<()> {
        trace!("on_audio_levels(): {}, {}", captured_level, received_level);
        Ok(())
    }

    fn on_low_bandwidth_for_video(
        &self,
        _remote_peer: &Self::AppRemotePeer,
        recovered: bool,
    ) -> Result<()> {
        info!("on_low_bandwidth_for_video(): {}", recovered);
        Ok(())
    }

    fn on_send_offer(
        &self,
        remote_peer: &Self::AppRemotePeer,
        call_id: CallId,
        offer: signaling::Offer,
    ) -> Result<()> {
        info!(
            "on_send_offer(): remote_peer: {}, call_id: {}, offer: {}",
            remote_peer,
            call_id,
            offer.to_info_string()
        );

        if self.force_internal_fault.load(Ordering::Acquire) {
            Err(SimError::SendOfferError.into())
        } else {
            let _ = self.stats.offers_sent.fetch_add(1, Ordering::AcqRel);
            if self.force_signaling_failure.load(Ordering::Acquire) {
                self.message_send_failure(call_id).unwrap();
            } else {
                self.message_sent(call_id).unwrap();
            }
            Ok(())
        }
    }

    fn on_send_answer(
        &self,
        remote_peer: &Self::AppRemotePeer,
        call_id: CallId,
        send: signaling::SendAnswer,
    ) -> Result<()> {
        info!(
            "on_send_answer(): remote_peer: {}, call_id: {}, receiver_device_id: {}, answer: {}",
            remote_peer,
            call_id,
            send.receiver_device_id,
            send.answer.to_info_string()
        );

        if self.force_internal_fault.load(Ordering::Acquire) {
            Err(SimError::SendAnswerError.into())
        } else {
            let _ = self.stats.answers_sent.fetch_add(1, Ordering::AcqRel);
            if self.force_signaling_failure.load(Ordering::Acquire) {
                self.message_send_failure(call_id).unwrap();
            } else {
                self.message_sent(call_id).unwrap();
            }
            Ok(())
        }
    }

    fn on_send_ice(
        &self,
        remote_peer: &Self::AppRemotePeer,
        call_id: CallId,
        send: signaling::SendIce,
    ) -> Result<()> {
        let (_broadcast, receiver_device_id) = match send.receiver_device_id {
            // The DeviceId doesn't matter if we're broadcasting
            None => (true, 0),
            Some(receiver_device_id) => (false, receiver_device_id),
        };

        info!(
            "on_send_ice_candidates(): remote_peer: {}, call_id: {}, receiver_device_id: {}",
            remote_peer, call_id, receiver_device_id
        );

        *self.last_ice_sent.lock().unwrap() = Some(send.clone());

        if self.force_internal_fault.load(Ordering::Acquire) {
            Err(SimError::SendIceCandidateError.into())
        } else {
            let _ = self
                .stats
                .ice_candidates_sent
                .fetch_add(send.ice.candidates.len(), Ordering::AcqRel);
            if self.force_signaling_failure.load(Ordering::Acquire) {
                if !self.no_auto_message_sent_for_ice.load(Ordering::Acquire) {
                    self.message_send_failure(call_id).unwrap();
                }
            } else if !self.no_auto_message_sent_for_ice.load(Ordering::Acquire) {
                self.message_sent(call_id).unwrap();
            }
            Ok(())
        }
    }

    fn on_send_hangup(
        &self,
        remote_peer: &Self::AppRemotePeer,
        call_id: CallId,
        send: signaling::SendHangup,
    ) -> Result<()> {
        info!(
            "on_send_hangup(): remote_peer: {}, call_id: {}",
            remote_peer, call_id
        );

        if self.force_internal_fault.load(Ordering::Acquire) {
            Err(SimError::SendHangupError.into())
        } else {
            match send.hangup {
                signaling::Hangup::Normal => {
                    let _ = self
                        .stats
                        .normal_hangups_sent
                        .fetch_add(1, Ordering::AcqRel);
                }
                signaling::Hangup::AcceptedOnAnotherDevice(_) => {
                    let _ = self
                        .stats
                        .accepted_hangups_sent
                        .fetch_add(1, Ordering::AcqRel);
                }
                signaling::Hangup::DeclinedOnAnotherDevice(_) => {
                    let _ = self
                        .stats
                        .declined_hangups_sent
                        .fetch_add(1, Ordering::AcqRel);
                }
                signaling::Hangup::BusyOnAnotherDevice(_) => {
                    let _ = self.stats.busy_hangups_sent.fetch_add(1, Ordering::AcqRel);
                }
                signaling::Hangup::NeedPermission(_) => {
                    let _ = self
                        .stats
                        .need_permission_hangups_sent
                        .fetch_add(1, Ordering::AcqRel);
                }
            }
            if self.force_signaling_failure.load(Ordering::Acquire) {
                self.message_send_failure(call_id).unwrap();
            } else {
                self.message_sent(call_id).unwrap();
            }
            Ok(())
        }
    }

    fn on_send_busy(&self, remote_peer: &Self::AppRemotePeer, call_id: CallId) -> Result<()> {
        info!(
            "on_send_busy(): remote_peer: {}, call_id: {}",
            remote_peer, call_id
        );

        if self.force_internal_fault.load(Ordering::Acquire) {
            Err(SimError::SendBusyError.into())
        } else {
            let _ = self.stats.busys_sent.fetch_add(1, Ordering::AcqRel);
            if self.force_signaling_failure.load(Ordering::Acquire) {
                self.message_send_failure(call_id).unwrap();
            } else {
                self.message_sent(call_id).unwrap();
            }
            Ok(())
        }
    }

    fn send_call_message(
        &self,
        recipient_id: UserId,
        message: Vec<u8>,
        urgency: group_call::SignalingMessageUrgency,
    ) -> Result<()> {
        self.outgoing_call_messages
            .lock()
            .unwrap()
            .push(OutgoingCallMessage {
                recipient_id,
                message,
                urgency,
            });
        Ok(())
    }

    fn send_call_message_to_group(
        &self,
        _group_id: group_call::GroupId,
        message: Vec<u8>,
        urgency: group_call::SignalingMessageUrgency,
        recipients_override: HashSet<UserId>,
    ) -> Result<()> {
        for recipient_id in recipients_override {
            let _ = self.send_call_message(recipient_id, message.clone(), urgency);
        }
        Ok(())
    }

    fn create_incoming_media(
        &self,
        _connection: &Connection<Self>,
        _incoming_media: MediaStream,
    ) -> Result<Self::AppIncomingMedia> {
        Ok("MediaStream".to_owned())
    }

    fn connect_incoming_media(
        &self,
        remote_peer: &Self::AppRemotePeer,
        app_call_context: &Self::AppCallContext,
        _incoming_media: &Self::AppIncomingMedia,
    ) -> Result<()> {
        info!(
            "connect_incoming_media(): remote_peer: {}, call_context: {}",
            remote_peer, app_call_context
        );

        if self.force_internal_fault.load(Ordering::Acquire) {
            Err(SimError::MediaStreamError.into())
        } else {
            let _ = self.stats.stream_count.fetch_add(1, Ordering::AcqRel);
            Ok(())
        }
    }

    fn disconnect_incoming_media(&self, app_call_context: &Self::AppCallContext) -> Result<()> {
        info!(
            "disconnect_incoming_media(): call_context: {}",
            app_call_context
        );

        if self.force_internal_fault.load(Ordering::Acquire) {
            Err(SimError::CloseMediaError.into())
        } else {
            self.incoming_media_disconnected
                .store(true, Ordering::Release);
            Ok(())
        }
    }

    fn compare_remotes(
        &self,
        remote_peer1: &Self::AppRemotePeer,
        remote_peer2: &Self::AppRemotePeer,
    ) -> Result<bool> {
        info!(
            "compare_remotes(): remote1: {}, remote2: {}",
            remote_peer1, remote_peer2
        );

        Ok(remote_peer1 == remote_peer2)
    }

    fn on_offer_expired(
        &self,
        _remote_peer: &Self::AppRemotePeer,
        _call_id: CallId,
        _age: Duration,
    ) -> Result<()> {
        info!("on_offer_expired():");
        let _ = self.stats.offer_expired.fetch_add(1, Ordering::AcqRel);
        Ok(())
    }

    fn on_call_concluded(
        &self,
        _remote_peer: &Self::AppRemotePeer,
        _call_id: CallId,
    ) -> Result<()> {
        info!("on_call_concluded():");
        if self.force_internal_fault.load(Ordering::Acquire) {
            Err(SimError::CallConcludedError.into())
        } else {
            let _ = self.stats.call_concluded.fetch_add(1, Ordering::AcqRel);
            Ok(())
        }
    }

    fn request_membership_proof(&self, client_id: group_call::ClientId) {
        let mut cm = self.call_manager.lock().unwrap();
        cm.as_mut().unwrap().set_membership_proof(client_id, vec![]);
    }

    fn request_group_members(&self, client_id: group_call::ClientId) {
        let mut cm = self.call_manager.lock().unwrap();
        cm.as_mut().unwrap().set_group_members(client_id, vec![]);
    }

    fn handle_connection_state_changed(
        &self,
        _client_id: group_call::ClientId,
        _connection_state: group_call::ConnectionState,
    ) {
        unimplemented!()
    }

    fn handle_network_route_changed(
        &self,
        _client_id: group_call::ClientId,
        network_route: NetworkRoute,
    ) {
        info!("handle_network_route_changed(): {:?}", network_route);
    }

    fn handle_speaking_notification(
        &self,
        _client_id: group_call::ClientId,
        event: group_call::SpeechEvent,
    ) {
        info!("handle_speaking_notification(): {:?}", event,);
    }

    fn handle_audio_levels(
        &self,
        _client_id: group_call::ClientId,
        captured_level: AudioLevel,
        received_levels: Vec<ReceivedAudioLevel>,
    ) {
        trace!(
            "handle_audio_levels(): {:?}, {:?}",
            captured_level,
            received_levels
        );
    }

    fn handle_low_bandwidth_for_video(&self, _client_id: group_call::ClientId, recovered: bool) {
        info!("handle_low_bandwidth_for_video(): {}", recovered);
    }

    fn handle_reactions(
        &self,
        _client_id: group_call::ClientId,
        reactions: Vec<group_call::Reaction>,
    ) {
        info!("handle_reactions(): {:?}", reactions);
    }

    fn handle_raised_hands(&self, _client_id: group_call::ClientId, raised_hands: Vec<DemuxId>) {
        info!("handle_raised_hands(): {:?}", raised_hands);
    }

    fn handle_join_state_changed(
        &self,
        _client_id: group_call::ClientId,
        _join_state: group_call::JoinState,
    ) {
    }

    fn handle_remote_devices_changed(
        &self,
        _client_id: group_call::ClientId,
        _remote_device_states: &[group_call::RemoteDeviceState],
        _reason: group_call::RemoteDevicesChangedReason,
    ) {
    }

    fn handle_incoming_video_track(
        &self,
        _client_id: group_call::ClientId,
        _remote_demux_id: DemuxId,
        _incoming_video_track: VideoTrack,
    ) {
        unimplemented!()
    }

    fn handle_peek_changed(
        &self,
        _client_id: group_call::ClientId,
        _peek_info: &PeekInfo,
        _joined_members: &HashSet<UserId>,
    ) {
        unimplemented!()
    }

    fn handle_ended(&self, _client_id: group_call::ClientId, _reason: group_call::EndReason) {
        unimplemented!()
    }

    fn group_call_ring_update(
        &self,
        group_id: group_call::GroupId,
        ring_id: group_call::RingId,
        sender_id: UserId,
        update: group_call::RingUpdate,
    ) {
        self.group_call_ring_updates
            .lock()
            .unwrap()
            .push(GroupCallRingUpdate {
                group_id,
                ring_id,
                sender_id,
                update,
            });
    }

    fn handle_remote_mute_request(&self, client_id: group_call::ClientId, mute_source: DemuxId) {
        info!("handle_remote_mute_request({}, {})", client_id, mute_source);
    }

    fn handle_observed_remote_mute(
        &self,
        client_id: group_call::ClientId,
        mute_source: DemuxId,
        mute_target: DemuxId,
    ) {
        info!(
            "handle_observed_remote_mute({}, {}, {})",
            client_id, mute_source, mute_target
        );
    }
}

impl sfu::Delegate for SimPlatform {
    fn handle_peek_result(&self, _request_id: u32, _peek_result: PeekResult) {
        unimplemented!()
    }
}

impl SimPlatform {
    /// Create a new SimPlatform object.
    pub fn new() -> Self {
        SimPlatform::default()
    }

    pub fn close(&mut self) {
        info!("close(): dropping Call Manager object");
        let mut cm = self.call_manager.lock().unwrap();
        let _ = cm.take();
    }

    pub fn set_call_manager(&mut self, call_manager: CallManager<Self>) {
        let mut cm = self.call_manager.lock().unwrap();
        *cm = Some(call_manager);
    }

    fn message_sent(&self, call_id: CallId) -> Result<()> {
        let mut cm = self.call_manager.lock().unwrap();
        cm.as_mut().unwrap().message_sent(call_id).unwrap();
        Ok(())
    }

    fn message_send_failure(&self, call_id: CallId) -> Result<()> {
        let mut cm = self.call_manager.lock().unwrap();
        cm.as_mut().unwrap().message_send_failure(call_id).unwrap();
        Ok(())
    }

    pub fn force_internal_fault(&mut self, enable: bool) {
        self.force_internal_fault.store(enable, Ordering::Release);
    }

    pub fn force_signaling_failure(&mut self, enable: bool) {
        self.force_signaling_failure
            .store(enable, Ordering::Release);
    }

    pub fn no_auto_message_sent_for_ice(&mut self, enable: bool) {
        self.no_auto_message_sent_for_ice
            .store(enable, Ordering::Release);
    }

    pub fn event_count(&self, event: ApplicationEvent) -> usize {
        let mut errors = 0;
        let map = self.event_map.lock().unwrap();

        if let Some(entry) = map.get(&event) {
            errors += entry;
        }

        errors
    }

    pub fn error_count(&self) -> usize {
        self.event_count(ApplicationEvent::EndedInternalFailure)
    }

    pub fn clear_error_count(&self) {
        let mut map = self.event_map.lock().unwrap();
        let _ = map.remove(&ApplicationEvent::EndedInternalFailure);
    }

    pub fn ended_count(&self) -> usize {
        let mut ends = 0;

        let ended_events = vec![
            ApplicationEvent::EndedLocalHangup,
            ApplicationEvent::EndedRemoteHangup,
            ApplicationEvent::EndedRemoteBusy,
            ApplicationEvent::EndedTimeout,
            ApplicationEvent::EndedInternalFailure,
            ApplicationEvent::EndedSignalingFailure,
            ApplicationEvent::EndedConnectionFailure,
            ApplicationEvent::EndedAppDroppedCall,
        ];
        for event in ended_events {
            ends += self.event_count(event);
        }

        ends
    }

    pub fn offers_sent(&self) -> usize {
        self.stats.offers_sent.load(Ordering::Acquire)
    }

    pub fn answers_sent(&self) -> usize {
        self.stats.answers_sent.load(Ordering::Acquire)
    }

    pub fn ice_candidates_sent(&self) -> usize {
        self.stats.ice_candidates_sent.load(Ordering::Acquire)
    }

    pub fn last_ice_sent(&self) -> Option<signaling::SendIce> {
        self.last_ice_sent.lock().unwrap().clone()
    }

    pub fn normal_hangups_sent(&self) -> usize {
        self.stats.normal_hangups_sent.load(Ordering::Acquire)
    }

    pub fn accepted_hangups_sent(&self) -> usize {
        self.stats.accepted_hangups_sent.load(Ordering::Acquire)
    }

    pub fn declined_hangups_sent(&self) -> usize {
        self.stats.declined_hangups_sent.load(Ordering::Acquire)
    }

    pub fn busy_hangups_sent(&self) -> usize {
        self.stats.busy_hangups_sent.load(Ordering::Acquire)
    }

    pub fn need_permission_hangups_sent(&self) -> usize {
        self.stats
            .need_permission_hangups_sent
            .load(Ordering::Acquire)
    }

    pub fn busys_sent(&self) -> usize {
        self.stats.busys_sent.load(Ordering::Acquire)
    }

    pub fn stream_count(&self) -> usize {
        self.stats.stream_count.load(Ordering::Acquire)
    }

    pub fn incoming_media_disconnected(&self) -> bool {
        self.incoming_media_disconnected.load(Ordering::Acquire)
    }

    pub fn start_outgoing_count(&self) -> usize {
        self.stats.start_outgoing.load(Ordering::Acquire)
    }

    pub fn start_incoming_count(&self) -> usize {
        self.stats.start_incoming.load(Ordering::Acquire)
    }

    pub fn offer_expired_count(&self) -> usize {
        self.stats.offer_expired.load(Ordering::Acquire)
    }

    pub fn call_concluded_count(&self) -> usize {
        self.stats.call_concluded.load(Ordering::Acquire)
    }

    pub fn take_group_call_ring_updates(&self) -> Vec<GroupCallRingUpdate> {
        std::mem::take(&mut *self.group_call_ring_updates.lock().unwrap())
    }

    pub fn take_outgoing_call_messages(&self) -> Vec<OutgoingCallMessage> {
        std::mem::take(&mut *self.outgoing_call_messages.lock().unwrap())
    }
}
