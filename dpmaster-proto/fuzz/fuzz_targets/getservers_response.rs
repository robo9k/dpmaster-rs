#![no_main]
use libfuzzer_sys::fuzz_target;

// ```
// dpmaster-proto$ cargo fuzz run getservers_response -- -dict=fuzz/dictionaries/getservers_response -max_len=1500 -timeout=1
// ```
fuzz_target!(|data: &[u8]| {
    let _ = dpmaster_proto::deserializer::getserversresponse_message(data);
});
