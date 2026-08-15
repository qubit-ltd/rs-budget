#![no_main]

use libfuzzer_sys::fuzz_target;
use qubit_budget::ResourceLimit;
use qubit_budget::json::JsonMeasurement;
use qubit_budget::json::JsonResource;
use qubit_budget::json::JsonValueLimits;

const MAX_INPUT_LEN: usize = 4 * 1024;

fuzz_target!(|data: &[u8]| {
    let input = &data[..data.len().min(MAX_INPUT_LEN)];
    let nodes = u64::from(input.first().copied().unwrap_or_default());
    let payload = u64::from(input.get(1).copied().unwrap_or_default());
    let limits = JsonValueLimits::<JsonResource, u64>::unconfigured()
        .with_structure_limits(
            qubit_budget::StructureLimits::empty().with_nodes_limit(
                ResourceLimit::new(JsonResource::Nodes, nodes),
            ),
        )
        .with_payload_bytes_limit(ResourceLimit::new(
            JsonResource::PayloadBytes,
            payload,
        ));
    let mut budget = limits.budget();
    for chunk in input.get(2..).unwrap_or_default().chunks(3) {
        let measurement = match chunk.first().copied().unwrap_or_default() % 4 {
            0 => JsonMeasurement::Null { depth: 0 },
            1 => JsonMeasurement::Boolean { depth: 0 },
            2 => JsonMeasurement::String {
                depth: 0,
                bytes: usize::from(chunk.get(1).copied().unwrap_or_default()),
            },
            _ => JsonMeasurement::Key {
                bytes: usize::from(chunk.get(1).copied().unwrap_or_default()),
            },
        };
        let before_nodes = budget.used_nodes();
        let before_payload = budget.used_payload_bytes();
        let commit = chunk.get(2).copied().unwrap_or_default() & 1 == 1;
        let mut transaction = budget.transaction();
        let result = transaction.try_admit(measurement);
        if result.is_ok() && commit {
            transaction.commit();
        }
        if result.is_err() || !commit {
            assert_eq!(budget.used_nodes(), before_nodes);
            assert_eq!(budget.used_payload_bytes(), before_payload);
        }
        assert!(budget.used_nodes() <= Some(nodes));
        assert!(budget.used_payload_bytes() <= Some(payload));
    }
});
