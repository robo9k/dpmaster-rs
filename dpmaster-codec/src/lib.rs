use bytes::BytesMut;
use dpmaster_proto::messages::{GetServersMessage, GetServersResponseMessage};
use tokio_util::codec::{Decoder, Encoder};

struct GetServersMessageEncoder {}

impl Encoder<GetServersMessage> for GetServersMessageEncoder {
    type Error = std::io::Error;

    fn encode(&mut self, _item: GetServersMessage, _dst: &mut BytesMut) -> Result<(), Self::Error> {
        todo!()
    }
}

struct GetServersResponseMessageDecoder {}

impl Decoder for GetServersResponseMessageDecoder {
    type Item = GetServersResponseMessage;
    type Error = std::io::Error;

    fn decode(&mut self, _src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
