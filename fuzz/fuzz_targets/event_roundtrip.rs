#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        if let Ok(ev) = serde_json::from_str::<kimi_wire::protocol::Event>(s) {
            let round = serde_json::to_string(&ev).unwrap();
            let back: kimi_wire::protocol::Event = serde_json::from_str(&round).unwrap();
            assert_eq!(ev, back);
        }
    }
});
