// ```
// dpmaster-proto$ cargo bolero test --engine kani --max-input-length 1500 --timeout 1s getinfo
// ```
#[cfg_attr(kani, kani::proof)]
fn main() {
    bolero::check!().for_each(|v| {
        let _ = dpmaster_proto::deserializer::getinfo(v);
    });
}
