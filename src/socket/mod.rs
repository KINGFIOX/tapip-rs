use crate::time::Instant;

pub mod icmp;
pub mod tcp;
pub mod udp;

/// A network socket.
///
/// This enumeration abstracts the various types of sockets based on the IP protocol.
/// To downcast a `Socket` value to a concrete socket, use the [AnySocket] trait,
/// e.g. to get `udp::Socket`, call `udp::Socket::downcast(socket)`.
///
/// It is usually more convenient to use [SocketSet::get] instead.
///
/// [AnySocket]: trait.AnySocket.html
/// [SocketSet::get]: struct.SocketSet.html#method.get
#[derive(Debug)]
pub enum Socket<'a> {
    Icmp(icmp::Socket<'a>),
    // Raw(raw::Socket<'a>),
    // Udp(udp::Socket<'a>),
    // Tcp(tcp::Socket<'a>),
}

/// A conversion trait for network sockets.
pub trait AnySocket<'a> {
    fn upcast(self) -> Socket<'a>;
    fn downcast<'c>(socket: &'c Socket<'a>) -> Option<&'c Self>
    where
        Self: Sized;
    fn downcast_mut<'c>(socket: &'c mut Socket<'a>) -> Option<&'c mut Self>
    where
        Self: Sized;
}

/// Gives an indication on the next time the socket should be polled.
#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Clone, Copy)]
pub(crate) enum PollAt {
    /// The socket needs to be polled immediately.
    Now,
    /// The socket needs to be polled at given [Instant][struct.Instant].
    Time(Instant),
    /// The socket does not need to be polled unless there are external changes.
    Ingress,
}

macro_rules! from_socket {
    ($socket:ty, $variant:ident) => {
        impl<'a> AnySocket<'a> for $socket {
            fn upcast(self) -> Socket<'a> {
                Socket::$variant(self)
            }

            fn downcast<'c>(socket: &'c Socket<'a>) -> Option<&'c Self> {
                #[allow(unreachable_patterns)]
                match socket {
                    Socket::$variant(socket) => Some(socket),
                    _ => None,
                }
            }

            fn downcast_mut<'c>(socket: &'c mut Socket<'a>) -> Option<&'c mut Self> {
                #[allow(unreachable_patterns)]
                match socket {
                    Socket::$variant(socket) => Some(socket),
                    _ => None,
                }
            }
        }
    };
}

from_socket!(icmp::Socket<'a>, Icmp);
