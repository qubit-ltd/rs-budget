use qubit_budget::LimitExceeded;
use qubit_budget::ResourceLimit;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestResource {
    Items,
}

#[test]
fn test_error_accessors_return_structured_facts() {
    let error =
        LimitExceeded::new(TestResource::Items, ResourceLimit::new(4), 5);
    assert_eq!(error.resource(), &TestResource::Items);
    assert_eq!(error.limit().maximum(), 4);
    assert_eq!(error.observed(), 5);
    assert_eq!(error.into_resource(), TestResource::Items);
}
