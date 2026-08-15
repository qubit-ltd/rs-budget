#![no_main]

use std::io::Write as _;

use libfuzzer_sys::fuzz_target;
use qubit_budget::ResourceBudget;

const MAX_INPUT_LEN: usize = 4 * 1024;

fuzz_target!(|data: &[u8]| {
    let input = &data[..data.len().min(MAX_INPUT_LEN)];
    let maximum = u64::from(input.first().copied().unwrap_or_default());
    let mut budget = ResourceBudget::new((), maximum);
    let payload = input.get(1..).unwrap_or_default();
    let result = budget.try_write_string(|writer| {
        let mut output = writer.as_io();
        for chunk in payload.chunks(8) {
            output.write_all(chunk)?;
        }
        Ok::<(), std::io::Error>(())
    });
    match result {
        Ok(output) => {
            assert!(std::str::from_utf8(output.as_bytes()).is_ok());
            assert_eq!(budget.used(), output.len() as u64);
            assert!(budget.used() <= maximum);
        }
        Err(_) => {
            assert_eq!(budget.used(), 0);
            assert_eq!(budget.remaining(), maximum);
        }
    }
});
