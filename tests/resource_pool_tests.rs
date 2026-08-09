use proptest::prelude::any;
use proptest::prelude::prop;
use proptest::prelude::prop_assert_eq;
use proptest::prelude::proptest;
use qubit_budget::ResourceLimit;
use qubit_budget::ResourcePool;
use qubit_budget::ResourcePoolError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestResource {
    OpenFiles,
}

fn acquire_then_release(
    pool: &mut ResourcePool<TestResource>,
    amount: u64,
) -> Result<(), ResourcePoolError<TestResource>> {
    pool.try_acquire(amount)?;
    pool.release(amount)?;
    Ok(())
}

#[test]
fn test_release_makes_capacity_reusable() {
    let mut pool =
        ResourcePool::new(TestResource::OpenFiles, ResourceLimit::new(2));
    pool.try_acquire(2).expect("capacity should be acquirable");
    pool.release(1).expect("one held unit should be releasable");
    pool.try_acquire(1)
        .expect("released capacity should be reusable");
    assert_eq!(pool.available(), 0);
    assert_eq!(pool.in_use(), 2);
}

#[test]
fn test_one_error_type_supports_question_mark_for_both_operations() {
    let mut pool =
        ResourcePool::new(TestResource::OpenFiles, ResourceLimit::new(2));
    acquire_then_release(&mut pool, 1)
        .expect("both operations should share one error type");
}

#[test]
fn test_acquire_reports_exhaustion_without_mutation() {
    let mut pool =
        ResourcePool::new(TestResource::OpenFiles, ResourceLimit::new(2));
    pool.try_acquire(2)
        .expect("the maximum should be acquirable");
    let error = pool
        .try_acquire(1)
        .expect_err("one more unit should be exhausted");
    assert!(matches!(
        error,
        ResourcePoolError::Exhausted {
            available: 0,
            requested: 1,
            ..
        }
    ));
    assert_eq!(pool.available(), 0);
}

#[test]
fn test_invalid_release_is_atomic() {
    let mut pool =
        ResourcePool::new(TestResource::OpenFiles, ResourceLimit::new(2));
    pool.try_acquire(1).expect("one unit should fit");
    let error = pool
        .release(2)
        .expect_err("cannot release more than is held");
    assert!(matches!(
        error,
        ResourcePoolError::InvalidRelease {
            in_use: 1,
            requested: 2,
            ..
        }
    ));
    assert_eq!(pool.available(), 1);
}

#[test]
fn test_pool_accessors_report_the_finite_capacity() {
    let pool =
        ResourcePool::new(TestResource::OpenFiles, ResourceLimit::new(2));
    assert_eq!(pool.resource(), &TestResource::OpenFiles);
    assert_eq!(pool.limit(), ResourceLimit::new(2));
    assert_eq!(pool.capacity(), 2);
    assert_eq!(pool.available(), 2);
    assert_eq!(pool.in_use(), 0);
}

proptest! {
    #[test]
    fn test_legal_acquire_release_sequences_preserve_pool_invariants(
        maximum in 0_u64..=128,
        operations in prop::collection::vec((any::<bool>(), 0_u16..=256), 0..64),
    ) {
        let mut pool = ResourcePool::new(TestResource::OpenFiles, ResourceLimit::new(maximum));
        let mut model_available = maximum;

        for (acquire, raw_amount) in operations {
            if acquire {
                let requested = u64::from(raw_amount) % (model_available + 1);
                pool.try_acquire(requested).expect("model-generated acquire fits");
                model_available -= requested;
            } else {
                let in_use = maximum - model_available;
                let requested = u64::from(raw_amount) % (in_use + 1);
                pool.release(requested).expect("model-generated release is valid");
                model_available += requested;
            }
            prop_assert_eq!(pool.available(), model_available);
            prop_assert_eq!(pool.available() + pool.in_use(), maximum);
        }
    }
}
