#![no_main]
use libfuzzer_sys::fuzz_target;

// ```
// dpmaster-proto$ cargo fuzz run heartbeat -- -dict=fuzz/dictionaries/heartbeat -max_len=1500 -timeout=1
// ```
fuzz_target!(|data: &[u8]| {
    let _ = dpmaster_proto::deserializer::heartbeat(data);
});
