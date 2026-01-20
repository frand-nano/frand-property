use frand_property_macro::model;

model! {
    pub InnerModel {
        pub x: i32,
    }
}

model! {
    pub OuterModel {
        pub model inner: InnerModel,
        pub model inner_arr: InnerModel[2],
    }
}

#[test]
fn test_nested_model_instantiation() {
    let outer = OuterModel::new();
    
    // 내부 모델 존재 여부 확인
    let inner_sender = outer.inner.x.sender();
    inner_sender.send(10);
    
    // 속성 메서드를 사용하여 리시버 접근
    let inner_rx = outer.inner.x.receiver();
    assert_eq!(inner_rx.value(), 10);
    
    // 배열 내부 모델 확인
    let arr_0_sender = outer.inner_arr[0].x.sender();
    arr_0_sender.send(20);
    
    let arr_0_rx = outer.inner_arr[0].x.receiver();
    assert_eq!(arr_0_rx.value(), 20);
    
    let arr_1_sender = outer.inner_arr[1].x.sender();
    arr_1_sender.send(30);
    assert_eq!(outer.inner_arr[1].x.receiver().value(), 30);
}

#[test]
fn test_nested_model_clone_sender() {
    let outer = OuterModel::new();
    let outer_sender = outer.clone_sender();
    
    // 복제된 외부 센더를 통해 중첩된 센더에 접근
    outer_sender.inner.x.send(42);
    
    assert_eq!(outer.inner.x.receiver().value(), 42);
    
    outer_sender.inner_arr[0].x.send(100);
    assert_eq!(outer.inner_arr[0].x.receiver().value(), 100);
}

#[test]
fn test_nested_model_clone_receiver() {
    let outer = OuterModel::new();
    outer.inner.x.sender().send(123);
    
    let outer_receiver = outer.clone_receiver();
    
    assert_eq!(outer_receiver.inner.x.value(), 123);
    
    outer.inner_arr[1].x.sender().send(456);
    assert_eq!(outer_receiver.inner_arr[1].x.value(), 456);
}
