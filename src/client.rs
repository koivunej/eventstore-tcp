use std::io;
use std::net::SocketAddr;

use futures::Future;

use tokio_core::reactor::Handle;
use tokio_core::io::{Framed, Io};
use tokio_core::net::TcpStream;
use tokio_core::io::{Codec, EasyBuf};
use tokio_proto::TcpClient;
use tokio_proto::multiplex::{ClientProto, ClientService, RequestIdSource};
use tokio_service::Service;

use package::Package;
use codec::PackageCodec;

use uuid::Uuid;

pub struct EventStoreClient {
    inner: ClientService<TcpStream, PackageProto>,
}

impl EventStoreClient {
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
        Box::new(self.inner.call(req))
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

struct Separator;

impl Codec for Separator {
    type In = (Uuid, Package);
    type Out = (Uuid, Package);

    fn decode(&mut self, buf: &mut EasyBuf) -> io::Result<Option<Self::In>> {
        PackageCodec.decode(buf).map(|x| x.map(|x| (x.correlation_id, x)))
    }

    fn encode(&mut self, msg: (Uuid, Package), buf: &mut Vec<u8>) -> io::Result<()> {
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

struct PackageProto;

impl<T: Io + 'static> ClientProto<T> for PackageProto {
    type Request = Package;
    type Response = Package;
    type RequestId = Uuid;

    type Transport = Framed<T, Separator>;
    type BindTransport = Result<Self::Transport, io::Error>;
    type RequestIdSource = Separator;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(Separator))
    }

    fn requestid_source(&self) -> Self::RequestIdSource {
        Separator
    }
}
