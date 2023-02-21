#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = dpmaster_proto::deserializer::inforesponse(data);
});
