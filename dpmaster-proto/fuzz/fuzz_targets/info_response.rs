#![no_main]
use libfuzzer_sys::fuzz_target;

// ```
// dpmaster-proto$ cargo fuzz run info_response -- -dict=fuzz/dictionaries/info_response -max_len=1500 -timeout=1
// ```
fuzz_target!(|data: &[u8]| {
    let _ = dpmaster_proto::deserializer::inforesponse(data);
});
