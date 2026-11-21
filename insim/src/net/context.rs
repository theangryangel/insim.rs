use std::time::{Duration, Instant};

use crate::{
    Error, Packet, Result, VERSION,
    identifiers::RequestId,
    insim::{Isi, Tiny, TinyType},
};

/// The InSim conversation, driven by supplied packets, send outcomes and time.
/// It never accesses a socket or reads the clock.
#[derive(Debug)]
pub struct Context {
    timeout_at: Instant,
    active: bool,
    handshake: Option<Handshake>,
    ready: bool,
    closed: bool,
}
#[derive(Debug)]
struct Handshake {
    isi: Isi,
    deadline: Instant,
    retry_at: Option<Instant>,
}
impl Context {
    /// Create an inactive conversation. Telemetry can flow without InSim deadlines
    /// or automatic replies until initialization or resume is explicitly selected.
    pub fn unstarted(now: Instant) -> Self {
        Self {
            active: false,
            ready: false,
            ..Self::resume(now)
        }
    }
    /// Resume an already initialized InSim conversation at the supplied time.
    pub fn resume(now: Instant) -> Self {
        Self {
            timeout_at: now + Duration::from_secs(super::DEFAULT_TIMEOUT_SECS),
            active: true,
            handshake: None,
            ready: true,
            closed: false,
        }
    }
    /// Begin initialization. ReqI must be nonzero so LFS sends a correlated Ver.
    pub fn handshake(isi: Isi, udp: bool, now: Instant) -> Result<Self> {
        if isi.reqi == RequestId(0) {
            return Err(Error::InvalidHandshake("Isi.reqi must be nonzero"));
        }
        Ok(Self {
            timeout_at: now + Duration::from_secs(super::DEFAULT_TIMEOUT_SECS),
            active: true,
            handshake: Some(Handshake {
                isi,
                deadline: now + Duration::from_secs(10),
                retry_at: udp.then_some(now + Duration::from_secs(1)),
            }),
            ready: false,
            closed: false,
        })
    }
    /// Earliest protocol deadline, including handshake retries; absent before initialization.
    pub fn deadline(&self) -> Option<Instant> {
        self.active.then(|| {
            self.handshake.as_ref().map_or(self.timeout_at, |h| {
                h.retry_at.unwrap_or(h.deadline).min(h.deadline)
            })
        })
    }
    /// Whether a matching, compatible version reply has completed initialization.
    pub fn is_ready(&self) -> bool {
        self.ready && !self.closed
    }
    /// Advance timers; any returned packet must be queued for transmission immediately.
    pub fn advance(&mut self, now: Instant) -> Result<Option<Packet>> {
        if self.closed {
            return Err(Error::Disconnected);
        }
        if !self.active {
            return Ok(None);
        }
        if let Some(h) = &mut self.handshake {
            if now >= h.deadline {
                self.closed = true;
                return Err(Error::Timeout("InSim handshake".into()));
            }
            if h.retry_at.is_some_and(|at| now >= at) {
                h.retry_at = Some(now + Duration::from_secs(1));
                return Ok(Some(h.isi.clone().into()));
            }
        } else if now >= self.timeout_at {
            self.closed = true;
            return Err(Error::Timeout("no InSim packet received".into()));
        }
        Ok(None)
    }
    /// Process a received packet. Telemetry does not extend InSim liveness.
    /// Queue the returned automatic response before publishing the input packet.
    pub fn receive(&mut self, packet: &Packet, now: Instant) -> Result<Option<Packet>> {
        if self.closed {
            return Err(Error::Disconnected);
        }
        if !self.active {
            return Ok(None);
        }
        if matches!(
            packet,
            Packet::Outgauge(_) | Packet::Outsim(_) | Packet::Outsim2(_)
        ) {
            return Ok(None);
        }
        if let Packet::Ver(ver) = packet {
            if ver.insimver != VERSION {
                self.closed = true;
                return Err(Error::IncompatibleVersion(ver.insimver));
            }
            if self
                .handshake
                .as_ref()
                .is_some_and(|h| h.isi.reqi == ver.reqi)
            {
                self.handshake = None;
                self.ready = true;
            }
        }
        self.timeout_at = now + Duration::from_secs(super::DEFAULT_TIMEOUT_SECS);
        if matches!(
            packet,
            Packet::Tiny(Tiny {
                subt: TinyType::None,
                reqi: RequestId(0)
            })
        ) {
            return Ok(Some(
                Tiny {
                    subt: TinyType::None,
                    reqi: RequestId(0),
                }
                .into(),
            ));
        }
        Ok(None)
    }
    /// Restart the inactivity deadline while the caller is deliberately not reading.
    /// A handshake in progress keeps its own deadline.
    pub fn hold(&mut self, now: Instant) {
        if self.handshake.is_none() {
            self.timeout_at = now + Duration::from_secs(super::DEFAULT_TIMEOUT_SECS);
        }
    }
    /// Mark a terminal transport failure or completed shutdown.
    pub fn close(&mut self) {
        self.closed = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::insim::Ver;
    #[test]
    fn hold_defers_inactivity_but_not_the_handshake_deadline() {
        let now = Instant::now();
        let later = now + Duration::from_secs(super::super::DEFAULT_TIMEOUT_SECS + 1);
        let mut context = Context::resume(now);
        context.hold(later - Duration::from_secs(1));
        assert!(context.advance(later).unwrap().is_none());
        let mut context = Context::handshake(
            Isi {
                reqi: RequestId(7),
                ..Isi::default()
            },
            false,
            now,
        )
        .unwrap();
        context.hold(now + Duration::from_secs(9));
        assert!(matches!(
            context.advance(now + Duration::from_secs(10)),
            Err(Error::Timeout(_))
        ));
    }
    #[test]
    fn unstarted_conversation_has_no_deadline_or_protocol_reactions() {
        let now = Instant::now();
        let later = now + Duration::from_secs(86400);
        let mut context = Context::unstarted(now);
        assert_eq!(context.deadline(), None);
        assert!(!context.is_ready());
        assert!(context.advance(later).unwrap().is_none());
        let keepalive = Tiny {
            reqi: RequestId(0),
            subt: TinyType::None,
        }
        .into();
        assert!(context.receive(&keepalive, later).unwrap().is_none());
        assert_eq!(context.deadline(), None);
    }
    #[test]
    fn handshake_requires_matching_version_reply() {
        let now = Instant::now();
        let mut context = Context::handshake(
            Isi {
                reqi: RequestId(7),
                ..Isi::default()
            },
            false,
            now,
        )
        .unwrap();
        let mut ver = Ver {
            reqi: RequestId(8),
            insimver: VERSION,
            ..Ver::default()
        };
        assert!(context.receive(&ver.clone().into(), now).unwrap().is_none());
        assert!(!context.is_ready());
        ver.reqi = RequestId(7);
        assert!(context.receive(&ver.into(), now).unwrap().is_none());
        assert!(context.is_ready());
    }
    #[test]
    fn udp_handshake_retries_and_has_an_absolute_deadline() {
        let now = Instant::now();
        let mut context = Context::handshake(
            Isi {
                reqi: RequestId(7),
                ..Isi::default()
            },
            true,
            now,
        )
        .unwrap();
        assert!(context.advance(now).unwrap().is_none());
        assert!(matches!(
            context.advance(now + Duration::from_secs(1)).unwrap(),
            Some(Packet::Isi(_))
        ));
        assert!(matches!(
            context.advance(now + Duration::from_secs(10)),
            Err(Error::Timeout(_))
        ));
    }
    #[test]
    fn telemetry_does_not_extend_liveness_but_insim_does() {
        let now = Instant::now();
        let mut context = Context::resume(now);
        let telemetry = Packet::Outsim2(Default::default());
        assert!(
            context
                .receive(&telemetry, now + Duration::from_secs(60))
                .unwrap()
                .is_none()
        );
        assert_eq!(
            context.deadline(),
            Some(now + Duration::from_secs(super::super::DEFAULT_TIMEOUT_SECS))
        );
        let keepalive = Tiny {
            reqi: RequestId(0),
            subt: TinyType::None,
        }
        .into();
        assert!(matches!(
            context
                .receive(&keepalive, now + Duration::from_secs(60))
                .unwrap(),
            Some(Packet::Tiny(_))
        ));
        assert!(context.advance(now + Duration::from_secs(71)).is_ok());
        assert!(matches!(
            context.advance(now + Duration::from_secs(130)),
            Err(Error::Timeout(_))
        ));
    }
    #[test]
    fn incompatible_version_stays_failed_and_zero_handshake_id_is_rejected() {
        let now = Instant::now();
        assert!(Context::handshake(Isi::default(), false, now).is_err());
        let mut context = Context::resume(now);
        let packet = Ver {
            insimver: VERSION + 1,
            ..Ver::default()
        }
        .into();
        assert!(matches!(
            context.receive(&packet, now),
            Err(Error::IncompatibleVersion(_))
        ));
        assert!(matches!(context.advance(now), Err(Error::Disconnected)));
    }
}
