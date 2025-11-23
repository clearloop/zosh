//! Tests for the initialize instruction

use crate::Test;
use zyphers::api;

#[test]
fn test_initialize_success() -> anyhow::Result<()> {
    let test = Test::new();
    let vals = crate::pubkeys(3);
    let instruction = api::initialize(test.payer, vals, 2)?;
    let result = test
        .mollusk
        .process_instruction(&instruction, &[(test.payer, Default::default())]);
    println!("result: {:?}", result);
    Ok(())
}
