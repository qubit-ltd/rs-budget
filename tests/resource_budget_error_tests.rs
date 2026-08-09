use qubit_budget::ResourceBudget;
use qubit_budget::ResourceLimit;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestResource {
    Bytes,
}

#[test]
fn test_budget_error_exposes_resource_limit_remaining_and_request() {
    let mut budget =
        ResourceBudget::new(TestResource::Bytes, ResourceLimit::new(3));
    budget.try_consume(2).expect("two units should fit");
    let error = budget
        .try_consume(2)
        .expect_err("two more units should exceed the remainder");
    assert_eq!(error.resource(), &TestResource::Bytes);
    assert_eq!(error.limit(), ResourceLimit::new(3));
    assert_eq!(error.remaining(), 1);
    assert_eq!(error.requested(), 2);
    assert_eq!(error.into_resource(), TestResource::Bytes);
}

#[test]
fn test_budget_error_display_contains_the_failed_facts() {
    let mut budget =
        ResourceBudget::new(TestResource::Bytes, ResourceLimit::new(1));
    let error = budget
        .try_consume(2)
        .expect_err("two units should exceed one unit");
    let display = error.to_string();
    assert!(display.contains("requested 2"));
    assert!(display.contains("only 1"));
}
