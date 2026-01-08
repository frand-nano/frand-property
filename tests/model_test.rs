use frand_property::model;
use frand_property::arraystring::{ArrayString, typenum::U20};
use std::time::Duration;
use tokio::time::sleep;

// 1. 기본 모델 정의 테스트
model! {
    pub BasicModel {
        pub count: i32,
        pub name: ArrayString<U20>,
    }
}

#[tokio::test]
async fn test_basic_model() {
    let model = BasicModel::new();
    
    // 초기값 확인 (Default)
    assert_eq!(model.count.receiver().value(), 0);
    assert_eq!(model.name.receiver().value().as_str(), "");

    // 값 변경 및 확인
    model.count.sender().send(42);
    assert_eq!(model.count.receiver().value(), 42);

    let hello = ArrayString::<U20>::try_from_str("Hello").unwrap();
    model.name.sender().send(hello);
    assert_eq!(model.name.receiver().value().as_str(), "Hello");
}

#[tokio::test]
async fn test_async_notification() {
    let model = BasicModel::new();
    let mut receiver = model.count.receiver().clone();

    // 별도 태스크에서 값 변경
    tokio::spawn(async move {
        sleep(Duration::from_millis(50)).await;
        model.count.sender().send(100);
    });

    // 변경 감지 대기
    let new_val = receiver.changed().await;
    assert_eq!(new_val, 100);
}

// 2. 배열 필드 테스트
model! {
    pub ArrayFieldModel {
        pub scores: i32[5],
    }
}

#[test]
fn test_array_field() {
    let model = ArrayFieldModel::new();
    
    assert_eq!(model.scores.len(), 5);
    
    // 개별 요소 접근 및 수정
    model.scores[0].sender().send(10);
    model.scores[4].sender().send(99);

    assert_eq!(model.scores[0].receiver().value(), 10);
    assert_eq!(model.scores[1].receiver().value(), 0); // 초기값
    assert_eq!(model.scores[4].receiver().value(), 99);
}

// 3. 모델 배열 테스트
const MODEL_COUNT: usize = 3;
model! {
    pub ItemModel {
        pub id: i32,
    }
}

#[test]
fn test_model_vec() {
    let models = ItemModel::new_vec::<MODEL_COUNT>(); // Vec<ItemModel> 반환
    
    assert_eq!(models.len(), 3);
    
    models[0].id.sender().send(1);
    models[1].id.sender().send(2);
    
    assert_eq!(models[0].id.receiver().value(), 1);
    assert_eq!(models[1].id.receiver().value(), 2);
    assert_eq!(models[2].id.receiver().value(), 0);
}

// 4. 가시성 테스트 (모듈 내부 테스트라 private 접근 가능, 컴파일 여부만 확인)
model! {
    pub VisibilityTestModel {
        pub public_val: i32,
        private_val: i32,
    }
}

#[test]
fn test_visibility_compile() {
    let model = VisibilityTestModel::new();
    model.public_val.sender().send(1);
    model.private_val.sender().send(2); // 같은 모듈 내라서 접근 가능
}

// 5. ModelSender/ModelReceiver 테스트
#[tokio::test]
async fn test_model_sender_receiver() {
    let model = BasicModel::new();
    
    let sender = model.clone_sender();
    let receiver = model.clone_receiver();
    
    sender.count.send(777);
    assert_eq!(receiver.count.value(), 777);
    assert_eq!(model.count.receiver().value(), 777);
    
    let world = ArrayString::<U20>::try_from_str("World").unwrap();
    sender.name.send(world);
    assert_eq!(receiver.name.value().as_str(), "World");
    assert_eq!(model.name.receiver().value().as_str(), "World");
}

// 6. Array ModelSender/ModelReceiver 테스트
#[tokio::test]
async fn test_vec_model_sender_receiver() {
    let model = ArrayFieldModel::new();
    
    let sender = model.clone_sender();
    let receiver = model.clone_receiver();
    
    sender.scores[2].send(12345);
    assert_eq!(receiver.scores[2].value(), 12345);
    assert_eq!(model.scores[2].receiver().value(), 12345);
}

// 7. Blanket Extension Trait Method 테스트
#[tokio::test]
async fn test_model_extension_methods() {
    use frand_property::ModelList; // Trait import
    
    let models = BasicModel::new_vec::<5>();
    
    // 메소드 문법 호출
    let senders = models.clone_senders();
    let receivers = models.clone_receivers();
    
    assert_eq!(senders.len(), 5);
    assert_eq!(receivers.len(), 5);
    
    senders[0].count.send(100);
    assert_eq!(receivers[0].count.value(), 100);
    assert_eq!(models[0].count.receiver().value(), 100);
}
