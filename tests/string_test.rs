use frand_property::{model, Property};

model! {
    StringModel {
        pub name: String,
    }
}

#[test]
fn test_string_property_manual() {
    let prop = Property::<String>::from("Initial".to_string());
    assert_eq!(prop.receiver().value(), "Initial");

    prop.sender().send("Changed".to_string());
    assert_eq!(prop.receiver().value(), "Changed");
    
    // Test borrow
    {
        let borrowed = prop.receiver().borrow();
        assert_eq!(*borrowed, "Changed");
    }
}

#[test]
fn test_string_model() {
    let model = StringModel::clone_singleton();
    
    model.name.sender().send("Alice".to_string());
    assert_eq!(model.name.receiver().value(), "Alice");
    
    let model2 = StringModel::clone_singleton();
    assert_eq!(model2.name.receiver().value(), "Alice");
    
    model2.name.sender().send("Bob".to_string());
    assert_eq!(model.name.receiver().value(), "Bob");
}
