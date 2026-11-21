#[cfg(feature = "tokio")]
use std::task::{Context, Poll};
#[cfg(any(feature = "tokio", feature = "blocking"))]
use std::time::Instant;
use std::{fmt::Debug, io, net::SocketAddr};

/// Selects socket types and readiness waiting; does not create an executor.
pub trait Runtime: Debug + Send + Sync + Sized + 'static {
    /// TCP socket belonging to this runtime.
    type Tcp: StreamSocket<Runtime = Self>;
    /// UDP socket belonging to this runtime.
    type Udp: DatagramSocket<Runtime = Self>;
}

/// Readiness a wait observes for one socket.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Interest {
    /// Wake when the socket has input.
    pub read: bool,
    /// Wake when the socket can accept output.
    pub write: bool,
}
impl Interest {
    /// Input only.
    pub const READ: Self = Self {
        read: true,
        write: false,
    };
}

/// Nonblocking socket operations used by the shared transport.
pub trait Socket: Debug + Send + Sync + 'static {
    /// Runtime to which this socket belongs.
    type Runtime: Runtime;
    /// Whether reads and writes preserve datagram boundaries.
    fn is_datagram(&self) -> bool;
    /// Prepare a caller-owned socket for nonblocking operations.
    fn prepare(&self) -> io::Result<()>;
    /// Attempt a read without waiting.
    fn try_recv(&self, buf: &mut [u8]) -> io::Result<usize>;
    /// Attempt a write without waiting.
    fn try_send(&self, buf: &[u8]) -> io::Result<usize>;
    /// Local socket address.
    fn local_addr(&self) -> io::Result<SocketAddr>;
    /// End the TCP write direction; a no-op for UDP.
    fn shutdown(&self) -> io::Result<()>;
    /// Poll Tokio readiness. Only used for sockets whose runtime is Tokio.
    #[cfg(feature = "tokio")]
    #[doc(hidden)]
    fn poll_ready(&self, _cx: &mut Context<'_>, _interest: Interest) -> Poll<io::Result<()>> {
        unreachable!("not a Tokio socket")
    }
    /// Borrow the operating-system source for blocking readiness registration.
    #[cfg(all(feature = "blocking", unix))]
    #[doc(hidden)]
    fn source(&self) -> std::os::fd::BorrowedFd<'_> {
        unreachable!("not a blocking socket")
    }
    /// Borrow the operating-system source for blocking readiness registration.
    #[cfg(all(feature = "blocking", windows))]
    #[doc(hidden)]
    fn source(&self) -> std::os::windows::io::BorrowedSocket<'_> {
        unreachable!("not a blocking socket")
    }
}

/// A TCP socket that determines its runtime at construction.
pub trait StreamSocket: Socket {
    /// Convert to the runtime's TCP socket type.
    fn into_tcp(self) -> <Self::Runtime as Runtime>::Tcp;
}
/// A UDP socket that determines its runtime at construction.
pub trait DatagramSocket: Socket {
    /// Convert to the runtime's UDP socket type.
    fn into_udp(self) -> <Self::Runtime as Runtime>::Udp;
}

/// Adapter for standard-library sockets and blocking readiness waiting.
#[cfg(feature = "blocking")]
#[derive(Debug)]
pub struct Blocking;
#[cfg(feature = "blocking")]
impl Runtime for Blocking {
    type Tcp = std::net::TcpStream;
    type Udp = std::net::UdpSocket;
}
/// Adapter for Tokio sockets. The application owns the Tokio executor.
#[cfg(feature = "tokio")]
#[derive(Debug)]
pub struct Tokio;
#[cfg(feature = "tokio")]
impl Runtime for Tokio {
    type Tcp = tokio::net::TcpStream;
    type Udp = tokio::net::UdpSocket;
}

#[cfg(feature = "blocking")]
struct CachedPoller {
    poller: polling::Poller,
    events: polling::Events,
}
#[cfg(feature = "blocking")]
thread_local! {
    // Each wait removes its registrations, so the thread can reuse this poller.
    static POLLER: std::cell::RefCell<Option<CachedPoller>> = const { std::cell::RefCell::new(None) };
}

#[cfg(feature = "blocking")]
impl Blocking {
    /// Wait for any requested socket readiness, or a deadline.
    pub fn wait(
        sockets: &[(&dyn Socket<Runtime = Self>, Interest)],
        deadline: Option<Instant>,
    ) -> io::Result<()> {
        POLLER.with(|cell| {
            let mut slot = cell.borrow_mut();
            let cached = match &mut *slot {
                Some(cached) => cached,
                None => slot.insert(CachedPoller {
                    poller: polling::Poller::new()?,
                    events: polling::Events::new(),
                }),
            };
            let mut clean = true;
            let result = Self::wait_on(cached, sockets, deadline, &mut clean);
            if !clean {
                // A registration could not be removed. Discard the poller rather than
                // let a stale entry affect a later wait.
                *slot = None;
            }
            result
        })
    }

    #[allow(unsafe_code)]
    fn wait_on(
        cached: &mut CachedPoller,
        sockets: &[(&dyn Socket<Runtime = Self>, Interest)],
        deadline: Option<Instant>,
        clean: &mut bool,
    ) -> io::Result<()> {
        // Registration borrows sockets only for the duration of this call.
        struct Guard<'a> {
            poller: &'a polling::Poller,
            sockets: Vec<&'a dyn Socket<Runtime = Blocking>>,
            clean: &'a mut bool,
        }
        impl Drop for Guard<'_> {
            fn drop(&mut self) {
                for socket in &self.sockets {
                    if self.poller.delete(socket.source()).is_err() {
                        *self.clean = false;
                    }
                }
            }
        }
        let poller = &cached.poller;
        let mut guard = Guard {
            poller,
            sockets: Vec::with_capacity(sockets.len()),
            clean,
        };
        for (key, (socket, interest)) in sockets.iter().enumerate() {
            if !interest.read && !interest.write {
                continue;
            }
            // SAFETY: the borrowed sockets outlive the guard. The guard unregisters
            // every successful addition before a socket can be dropped, on all paths.
            unsafe {
                poller.add(
                    &socket.source(),
                    polling::Event::new(key, interest.read, interest.write),
                )?;
            }
            guard.sockets.push(*socket);
        }
        loop {
            cached.events.clear();
            match poller.wait(
                &mut cached.events,
                deadline.map(|at| at.saturating_duration_since(Instant::now())),
            ) {
                Ok(_) => return Ok(()),
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            }
        }
    }
}

#[cfg(feature = "tokio")]
impl Tokio {
    /// Wait for any requested socket readiness, or a deadline.
    pub async fn wait(
        sockets: &[(&dyn Socket<Runtime = Self>, Interest)],
        deadline: Option<Instant>,
    ) -> io::Result<()> {
        let readiness = std::future::poll_fn(|cx| {
            for (socket, interest) in sockets {
                if let Poll::Ready(result) = socket.poll_ready(cx, *interest) {
                    return Poll::Ready(result);
                }
            }
            Poll::Pending
        });
        let timer = async {
            match deadline {
                Some(at) => tokio::time::sleep_until(at.into()).await,
                None => std::future::pending::<()>().await,
            }
        };
        tokio::select! {
            result = readiness => result,
            _ = timer => Ok(()),
        }
    }
}

#[cfg(feature = "blocking")]
macro_rules! blocking_registration {
    () => {
        #[cfg(unix)]
        fn source(&self) -> std::os::fd::BorrowedFd<'_> {
            std::os::fd::AsFd::as_fd(self)
        }
        #[cfg(windows)]
        fn source(&self) -> std::os::windows::io::BorrowedSocket<'_> {
            std::os::windows::io::AsSocket::as_socket(self)
        }
    };
}

#[cfg(feature = "blocking")]
impl Socket for std::net::TcpStream {
    type Runtime = Blocking;
    fn is_datagram(&self) -> bool {
        false
    }
    fn prepare(&self) -> io::Result<()> {
        self.set_nonblocking(true)
    }
    fn try_recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        std::io::Read::read(&mut &*self, buf)
    }
    fn try_send(&self, buf: &[u8]) -> io::Result<usize> {
        std::io::Write::write(&mut &*self, buf)
    }
    fn local_addr(&self) -> io::Result<SocketAddr> {
        self.local_addr()
    }
    fn shutdown(&self) -> io::Result<()> {
        self.shutdown(std::net::Shutdown::Write)
    }
    blocking_registration!();
}
#[cfg(feature = "blocking")]
impl Socket for std::net::UdpSocket {
    type Runtime = Blocking;
    fn is_datagram(&self) -> bool {
        true
    }
    fn prepare(&self) -> io::Result<()> {
        self.set_nonblocking(true)
    }
    fn try_recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.recv(buf)
    }
    fn try_send(&self, buf: &[u8]) -> io::Result<usize> {
        self.send(buf)
    }
    fn local_addr(&self) -> io::Result<SocketAddr> {
        self.local_addr()
    }
    fn shutdown(&self) -> io::Result<()> {
        Ok(())
    }
    blocking_registration!();
}
#[cfg(feature = "tokio")]
impl Socket for tokio::net::TcpStream {
    type Runtime = Tokio;
    fn is_datagram(&self) -> bool {
        false
    }
    fn prepare(&self) -> io::Result<()> {
        Ok(())
    }
    fn try_recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.try_read(buf)
    }
    fn try_send(&self, buf: &[u8]) -> io::Result<usize> {
        self.try_write(buf)
    }
    fn local_addr(&self) -> io::Result<SocketAddr> {
        self.local_addr()
    }
    fn shutdown(&self) -> io::Result<()> {
        // A borrowed standard socket shares the TCP connection without changing its options.
        let socket = socket2::SockRef::from(self);
        socket.shutdown(std::net::Shutdown::Write)
    }
    fn poll_ready(&self, cx: &mut Context<'_>, interest: Interest) -> Poll<io::Result<()>> {
        if interest.read {
            let read = self.poll_read_ready(cx).map_ok(|_| ());
            if read.is_ready() {
                return read;
            }
        }
        if interest.write {
            self.poll_write_ready(cx).map_ok(|_| ())
        } else {
            Poll::Pending
        }
    }
}
#[cfg(feature = "tokio")]
impl Socket for tokio::net::UdpSocket {
    type Runtime = Tokio;
    fn is_datagram(&self) -> bool {
        true
    }
    fn prepare(&self) -> io::Result<()> {
        Ok(())
    }
    fn try_recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.try_recv(buf)
    }
    fn try_send(&self, buf: &[u8]) -> io::Result<usize> {
        self.try_send(buf)
    }
    fn local_addr(&self) -> io::Result<SocketAddr> {
        self.local_addr()
    }
    fn shutdown(&self) -> io::Result<()> {
        Ok(())
    }
    fn poll_ready(&self, cx: &mut Context<'_>, interest: Interest) -> Poll<io::Result<()>> {
        if interest.read {
            let read = self.poll_recv_ready(cx);
            if read.is_ready() {
                return read;
            }
        }
        if interest.write {
            self.poll_send_ready(cx)
        } else {
            Poll::Pending
        }
    }
}
#[cfg(any(feature = "tokio", feature = "blocking"))]
macro_rules! socket_types {
    ($tcp:ty, $udp:ty) => {
        impl StreamSocket for $tcp {
            fn into_tcp(self) -> <Self::Runtime as Runtime>::Tcp {
                self
            }
        }
        impl DatagramSocket for $udp {
            fn into_udp(self) -> <Self::Runtime as Runtime>::Udp {
                self
            }
        }
    };
}
#[cfg(feature = "blocking")]
socket_types!(std::net::TcpStream, std::net::UdpSocket);
#[cfg(feature = "tokio")]
socket_types!(tokio::net::TcpStream, tokio::net::UdpSocket);

#[cfg(all(test, feature = "blocking"))]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn blocking_wait_reuses_the_thread_poller_across_sockets_and_calls() {
        let a = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        let b = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        for socket in [&a, &b] {
            socket.prepare().unwrap();
        }
        let sockets: [(&dyn Socket<Runtime = Blocking>, Interest); 2] =
            [(&a, Interest::READ), (&b, Interest::READ)];
        // Nothing readable: the deadline ends the wait.
        let start = Instant::now();
        Blocking::wait(&sockets, Some(start + Duration::from_millis(20))).unwrap();
        assert!(start.elapsed() >= Duration::from_millis(20));
        // Readiness on either socket ends a later wait on the same cached poller.
        for target in [&a, &b] {
            let _ = target.send_to(&[1], target.local_addr().unwrap()).unwrap();
            let start = Instant::now();
            Blocking::wait(&sockets, Some(start + Duration::from_secs(5))).unwrap();
            assert!(start.elapsed() < Duration::from_secs(1));
            let _ = target.try_recv(&mut [0; 4]).unwrap();
        }
        // A socket registered earlier can be dropped and its descriptor reused safely.
        drop(b);
        let c = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        c.prepare().unwrap();
        let _ = c.send_to(&[1], c.local_addr().unwrap()).unwrap();
        Blocking::wait(
            &[(&c, Interest::READ)],
            Some(Instant::now() + Duration::from_secs(5)),
        )
        .unwrap();
        assert!(POLLER.with(|cell| cell.borrow().is_some()));
    }
}
