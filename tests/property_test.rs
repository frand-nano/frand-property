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

// into_untyped: 컴포넌트 타입 C를 지워도 송신 시 부작용(setter)과 watch 채널이 보존되어야 한다.
#[test]
fn test_into_untyped_preserves_side_effect() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicI32, Ordering};

    let recorded = Arc::new(AtomicI32::new(-1));

    // send 시 component(=recorded)에 값을 기록하는 타입형 Sender
    let property = Property::<i32, Arc<AtomicI32>>::new(
        recorded.clone(),
        0,
        |c, v| { c.store(v, Ordering::SeqCst); },
    );

    let untyped = property.sender().clone().into_untyped();
    untyped.send(42);

    assert_eq!(recorded.load(Ordering::SeqCst), 42);   // 부작용 유지
    assert_eq!(untyped.value(), 42);                   // 동일 채널 관찰
    assert_eq!(property.receiver().value(), 42);
}
