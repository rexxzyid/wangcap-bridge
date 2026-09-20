//! Compile-shaped proof that a downstream crate can build message literals and
//! implement the async traits using only this crate's re-exports (no direct
//! buffa/bytes/anyhow/async-trait/chrono dependency of its own).
#![cfg(test)]
// Tests/benches exercise the raw buffa API.
#![allow(clippy::disallowed_methods)]
// The `wangcap_bridge::` prefixes ARE the assertion here — spelling a path the way
// a downstream consumer would is the only thing this file checks. Shortening them
// to the direct dependency would leave the test passing with the re-exports gone.
#![allow(unused_qualifications)]

use crate as wangcap_bridge;
use wangcap_bridge::waproto::whatsapp as wa;

#[test]
fn message_literals_build_from_reexports_only() {
    // Explicit MessageField path, as a consumer would write it.
    let explicit = wa::Message {
        extended_text_message: wangcap_bridge::buffa::MessageField::some(
            wa::message::ExtendedTextMessage {
                text: Some("hi".into()),
                ..Default::default()
            },
        ),
        ..Default::default()
    };
    // The From<T> route: no MessageField naming at all.
    let via_into = wa::Message {
        extended_text_message: wa::message::ExtendedTextMessage {
            text: Some("hi".into()),
            ..Default::default()
        }
        .into(),
        ..Default::default()
    };
    assert_eq!(explicit, via_into);

    // Encode/decode through the re-exported Message trait.
    use wangcap_bridge::buffa::Message as _;
    let bytes = explicit.encode_to_vec();
    let back = wa::Message::decode_from_slice(&bytes).unwrap();
    assert_eq!(back, via_into);
}

// An implementable trait built purely from re-exports, the veloz shape.
struct NoopHook;

#[wangcap_bridge::async_trait]
impl wangcap_bridge::InboundDurabilityHook for NoopHook {
    async fn on_messages(
        &self,
        _client: std::sync::Arc<wangcap_bridge::Client>,
        _batch: &[wangcap_bridge::types::events::InboundMessage],
    ) -> wangcap_bridge::anyhow::Result<()> {
        Ok(())
    }
}

#[test]
fn hook_impl_is_object_safe_and_constructible() {
    let hook: Box<dyn wangcap_bridge::InboundDurabilityHook> = Box::new(NoopHook);
    let _ = &hook;
}

// A RetryAdmission policy built purely from re-exports.
struct AdmitAll;

impl wangcap_bridge::RetryAdmission for AdmitAll {
    fn admit(
        &self,
        _chat: &wangcap_bridge::Jid,
        _requester: &wangcap_bridge::Jid,
        _retry_count: u8,
    ) -> bool {
        true
    }
}

#[test]
fn retry_admission_is_object_safe_and_constructible() {
    let policy: Box<dyn wangcap_bridge::RetryAdmission> = Box::new(AdmitAll);
    let _ = &policy;
}

#[test]
fn bytes_and_chrono_reexports_are_usable() {
    let b = wangcap_bridge::bytes::Bytes::from_static(b"frame");
    assert_eq!(b.len(), 5);
    let _ts: wangcap_bridge::chrono::DateTime<wangcap_bridge::chrono::Utc> =
        wangcap_bridge::wacore::time::now_utc();
}
