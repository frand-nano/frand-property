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
fn test_model_array() {
    let models = ItemModel::new_array::<MODEL_COUNT>(); // Vec<ItemModel> 반환
    
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
