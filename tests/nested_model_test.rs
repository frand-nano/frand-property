use frand_property_macro::model;

// Test 1: Instantiation
model! {
    InnerModel1 {
        pub x: i32,
    }
}

model! {
    InnerModel1Arr[2] {
        pub x: i32,
    }
}

model! {
    OuterModel1 {
        pub model inner: InnerModel1,
        pub model inner_arr: InnerModel1Arr[],
    }
}

#[test]
fn test_nested_model_instantiation() {
    let outer = OuterModel1::clone_singleton();
    
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

// Test 2: Clone Sender
model! {
    InnerModel2 {
        pub x: i32,
    }
}

model! {
    InnerModel2Arr[2] {
        pub x: i32,
    }
}

model! {
    OuterModel2 {
        pub model inner: InnerModel2,
        pub model inner_arr: InnerModel2Arr[],
    }
}

#[test]
fn test_nested_model_clone_sender() {
    let outer = OuterModel2::clone_singleton();
    let outer_sender = outer.clone_sender();
    
    // 복제된 외부 센더를 통해 중첩된 센더에 접근
    outer_sender.inner.x.send(42);
    
    assert_eq!(outer.inner.x.receiver().value(), 42);
    
    outer_sender.inner_arr[0].x.send(100);
    assert_eq!(outer.inner_arr[0].x.receiver().value(), 100);
}

// Test 3: Clone Receiver
model! {
    InnerModel3 {
        pub x: i32,
    }
}

model! {
    InnerModel3Arr[2] {
        pub x: i32,
    }
}

model! {
    OuterModel3 {
        pub model inner: InnerModel3,
        pub model inner_arr: InnerModel3Arr[],
    }
}

#[test]
fn test_nested_model_clone_receiver() {
    let outer = OuterModel3::clone_singleton();
    outer.inner.x.sender().send(123);
    
    let outer_receiver = outer.clone_receiver();
    
    assert_eq!(outer_receiver.inner.x.value(), 123);
    
    outer.inner_arr[1].x.sender().send(456);
    assert_eq!(outer_receiver.inner_arr[1].x.value(), 456);
}

// Test 4: Implicit Array Length Syntax
model! {
    InnerModel4[5] {
        pub x: i32,
    }
}

model! {
    OuterModel4 {
        pub model inner_arr: InnerModel4[],
    }
}

#[test]
fn test_implicit_array_syntax() {
    let outer = OuterModel4::clone_singleton();
    
    // Check length
    assert_eq!(outer.inner_arr.len(), 5);
    
    // Check functionality
    outer.inner_arr[0].x.sender().send(99);
    assert_eq!(outer.inner_arr[0].x.receiver().value(), 99);
}
