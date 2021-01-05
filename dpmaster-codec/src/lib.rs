use bytes::{BufMut, BytesMut};
use cookie_factory::gen;
use dpmaster_proto::deserializer::getserversresponse_message;
use dpmaster_proto::messages::{GetServersMessage, GetServersResponseMessage};
use dpmaster_proto::serializer::gen_getservers_message;
use tokio_util::codec::{Decoder, Encoder};

pub struct GameClientCodec(());

impl GameClientCodec {
    pub fn new() -> Self {
        Self(())
    }
}

impl Encoder<GetServersMessage> for GameClientCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: GetServersMessage, dst: &mut BytesMut) -> Result<(), Self::Error> {
        gen(gen_getservers_message(&item), dst.writer())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)) // TODO
            .map(|_| ())
    }
}

impl Decoder for GameClientCodec {
    type Item = GetServersResponseMessage;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        getserversresponse_message(&src[..])
            .map_err(|_e| std::io::Error::new(std::io::ErrorKind::Other, "uhoh")) // TODO
            .map(|(_, m)| Some(m))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
