use frand_property::model;

model! {
    pub InitModel {
        pub val: i32,
    }
}


model! {
    pub InitArrayModel [5] {
        pub val: i32,
    }
}


#[test]
fn test_init_model() {
    // init_singleton() now returns Self (the inner struct), not Arc<Self>
    // And it shares state because it's a singleton.
    let m = InitModel::init_singleton(|m| {
        m.val.sender().send(42);
    });
    
    // Check if initialization worked
    assert_eq!(m.val.receiver().value(), 42);
    
    // Check if it's a SINGLETON (shared state)
    let m2 = InitModel::init_singleton(|m| { m.val.sender().send(100); });
    assert_eq!(m2.val.receiver().value(), 100);
    
    // Since it's a singleton, m1 should also see the change
    assert_eq!(m.val.receiver().value(), 100);
}

#[test]
fn test_init_array_model() {
    // Array Model: init takes an index
    let m0 = InitArrayModel::init_singleton(0, |m| {
        m.val.sender().send(10);
    });

    let m1 = InitArrayModel::init_singleton(1, |m| {
        m.val.sender().send(20);
    });

    assert_eq!(m0.val.receiver().value(), 10);
    assert_eq!(m1.val.receiver().value(), 20);
    
    // Verify singleton behavior across calls
    let m0_again = InitArrayModel::clone_singleton(); // Returns Arc<[Self]>
    assert_eq!(m0_again[0].val.receiver().value(), 10);
    assert_eq!(m0_again[1].val.receiver().value(), 20);
}
