use frand_property::{Property, PropertyList};

#[derive(Clone)]
struct Component {
    #[allow(dead_code)]
    id: usize,
}

#[test]
fn test_property_list_into_senders_receivers() {
    let p1 = Property::new(Component { id: 1 }, 10, |_, _| {});
    let p2 = Property::new(Component { id: 2 }, 20, |_, _| {});
    
    let props = vec![p1, p2];
    
    let senders = props.iter().into_senders();
    assert_eq!(senders.len(), 2);
    
    let receivers = props.iter().into_receivers();
    assert_eq!(receivers.len(), 2);
    
    assert_eq!(receivers[0].value(), 10);
    assert_eq!(receivers[1].value(), 20);
}

#[test]
fn test_property_list_from_slice() {
    let p1 = Property::from(10);
    let props = [p1];
    
    let senders = props.into_senders();
    assert_eq!(senders.len(), 1);
}
