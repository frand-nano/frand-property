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
    let m = InitModel::init(|m| {
        m.val.sender().send(42);
    });
    
    // Check if initialization worked
    assert_eq!(m.val.receiver().value(), 42);
    
    // Check if it's a valid independent instance
    let m2 = InitModel::init(|m| { m.val.sender().send(100); });
    assert_eq!(m2.val.receiver().value(), 100);
    assert_ne!(m.val.receiver().value(), m2.val.receiver().value());
}

#[test]
fn test_init_array_model() {
    let models = InitArrayModel::init(|ms| {
        assert_eq!(ms.len(), 5);
        for (i, m) in ms.iter().enumerate() {
            m.val.sender().send(i as i32 * 10);
        }
    });

    assert_eq!(models.len(), 5);
    for (i, m) in models.iter().enumerate() {
         assert_eq!(m.val.receiver().value(), i as i32 * 10);
    }
}
