use frand_property::model;

model! {
    pub SingletonModel {
        pub count: i32,
    }
}

#[test]
fn test_singleton() {
    let instance1 = SingletonModel::clone_singleton();
    let instance2 = SingletonModel::clone_singleton();

    // Verify they share state
    instance1.count.sender().send(123);
    assert_eq!(instance2.count.receiver().value(), 123);
    
    instance2.count.sender().send(456);
    assert_eq!(instance1.count.receiver().value(), 456);
}

model! {
    pub SingletonVecModel[3] {
        pub val: i32,
    }
}

#[test]
fn test_singleton_vec() {
    let instance1 = SingletonVecModel::clone_singleton();
    let instance2 = SingletonVecModel::clone_singleton();
    
    // instance1 is Arc<[SingletonVecModel]>
    
    instance1[0].val.sender().send(10);
    assert_eq!(instance2[0].val.receiver().value(), 10);
    
    instance2[2].val.sender().send(99);
    assert_eq!(instance1[2].val.receiver().value(), 99);
}
