use std::io;
use std::net::SocketAddr;

use futures::Future;

use tokio_core::reactor::Handle;
use tokio_io::{AsyncWrite, AsyncRead};
use tokio_io::codec::{Encoder, Decoder, Framed};
use tokio_core::net::TcpStream;
use tokio_proto::TcpClient;
use tokio_proto::multiplex::{ClientProto, ClientService, NewRequestIdSource, RequestIdSource};
use tokio_service::Service;
use bytes::BytesMut;

use package::Package;
use codec::PackageCodec;

use uuid::Uuid;

/// `tokio_service::Service` implementation of the client.
pub struct EventStoreClient {
    inner: ClientService<TcpStream, PackageProto>,
}

impl EventStoreClient {
    /// Connect to an EventStore database listening at given `addr` using the given
    /// `tokio::reactor::Core`s `handle`.
    /// Returns a future representing the client which can be used to send and receive `Package`
    /// values.
    pub fn connect(addr: &SocketAddr, handle: &Handle) -> Box<Future<Item = Self, Error = io::Error>> {
        let ret = TcpClient::new(PackageProto)
            .connect(addr, handle)
            .map(|client_service| {
                EventStoreClient { inner: client_service }
            });

        Box::new(ret)
    }
}

impl Service for EventStoreClient {
    type Request = Package;
    type Response = Package;
    type Error = io::Error;
    type Future = Box<Future<Item = Package, Error = io::Error>>;

    fn call(&self, req: Package) -> Self::Future {
        self.inner.call(req).boxed()
    }
}

/*
/// Simple middleware
struct Heartbeats<T> {
    inner: T,
}

impl<T> Stream for Heartbeats<T>
    where T: Service<Request = Package, Response = Package, Error = io::Error>,
          T::Future: 'static
{
    type Request = Package;
    type Response = Package;
    type Error = io::Error;
    type Future = Box<Future<Item = Package, Error = io::Error>>;

    fn call(&self, req: Package) -> Self::Future {
        if self.credentials.as_ref().is_some() && req.authentication.is_none() {
            req.authentication = self.credentials.clone();
        }

        self.inner.call(req)
    }
}
*/

pub struct Separator;

impl Decoder for Separator {
    type Item = (Uuid, Package);
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<Self::Item>> {
        PackageCodec.decode(buf).map(|x| x.map(|x| (x.correlation_id, x)))
    }
}

impl Encoder for Separator {
    type Item = (Uuid, Package);
    type Error = io::Error;

    fn encode(&mut self, msg: (Uuid, Package), buf: &mut BytesMut) -> io::Result<()> {
        let (id, pkg) = msg;
        assert_eq!(id, pkg.correlation_id);
        PackageCodec.encode(pkg, buf)
    }
}

impl RequestIdSource<Uuid, Package> for Separator {
    fn next(&mut self, pkg: &Package) -> Uuid {
        pkg.correlation_id
    }
}

impl NewRequestIdSource<Uuid, Package> for Uuid {
    type RequestIdSource = Separator;

    fn requestid_source() -> Self::RequestIdSource {
        Separator
    }
}

struct PackageProto;

impl<T: AsyncRead + AsyncWrite + 'static> ClientProto<T> for PackageProto {
    type Request = Package;
    type Response = Package;
    type RequestId = Uuid;

    type Transport = Framed<T, Separator>;
    type BindTransport = Result<Self::Transport, io::Error>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(Separator))
    }
}
