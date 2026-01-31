use frand_property::{model, Property, ReceiverGroup};
use arraystring::{ArrayString, typenum::U20};
use std::time::Duration;
use tokio::time::sleep;


// 1. 기본 모델 정의 테스트
model! {
    BasicModel1 {
        pub count: i32,
        pub name: ArrayString<U20>,
    }
}

#[tokio::test]
async fn test_basic_model() {
    let model = BasicModel1::clone_singleton();
    
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

model! {
    AsyncModel {
        pub count: i32,
    }
}

#[tokio::test]
async fn test_async_notification() {
    let model = AsyncModel::clone_singleton();
    let mut receiver = model.count.receiver().clone();

    // 별도 태스크에서 값 변경
    tokio::spawn(async move {
        sleep(Duration::from_millis(50)).await;
        model.count.sender().send(100);
    });

    // 변경 감지 대기
    let new_val = receiver.modified().await;
    assert_eq!(new_val, 100);
}

// 2. 배열 필드 테스트
model! {
    ArrayFieldModel {
        pub scores: i32[5],
    }
}

#[test]
fn test_array_field() {
    let model = ArrayFieldModel::clone_singleton();
    
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
    ItemModel[MODEL_COUNT] {
        pub id: i32,
    }
}

#[test]
fn test_model_vec() {
    let models = ItemModel::clone_singleton(); // Arc<[ItemModel]> 반환
    
    assert_eq!(models.len(), 3);
    
    models[0].id.sender().send(1);
    models[1].id.sender().send(2);
    
    assert_eq!(models[0].id.receiver().value(), 1);
    assert_eq!(models[1].id.receiver().value(), 2);
    assert_eq!(models[2].id.receiver().value(), 0);
}

// 4. 가시성 테스트 (모듈 내부 테스트라 private 접근 가능, 컴파일 여부만 확인)
model! {
    VisibilityTestModel {
        pub public_val: i32,
        private_val: i32,
    }
}

#[test]
fn test_visibility_compile() {
    let model = VisibilityTestModel::clone_singleton();
    model.public_val.sender().send(1);
    model.private_val.sender().send(2); // 같은 모듈 내라서 접근 가능
}

// 5. ModelSender/ModelReceiver 테스트
#[tokio::test]
async fn test_model_sender_receiver() {
    let model = BasicModel1::clone_singleton();
    
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
    let model = ArrayFieldModel::clone_singleton();
    
    let sender = model.clone_sender();
    let receiver = model.clone_receiver();
    
    sender.scores[2].send(12345);
    assert_eq!(receiver.scores[2].value(), 12345);
    assert_eq!(model.scores[2].receiver().value(), 12345);
}

// 7. Blanket Extension Trait Method 테스트
model! {
    ExtensionModel {
        pub count: i32,
    }
}

#[tokio::test]
async fn test_model_extension_methods() {
    use frand_property::ModelList; // Trait import
    
    let models = {
        let mut v = Vec::with_capacity(5);
        for _ in 0..5 {
            v.push(ExtensionModel::clone_singleton());
        }
        v
    };
    
    // 메소드 문법 호출
    let senders = models.clone_senders();
    let receivers = models.clone_receivers();
    
    assert_eq!(senders.len(), 5);
    assert_eq!(receivers.len(), 5);
    

    senders[0].count.send(100);
    assert_eq!(receivers[0].count.value(), 100);
    assert_eq!(models[0].count.receiver().value(), 100);
}

// 8. Iterator Bind 테스트
const BIND_COUNT: usize = 5;
model! {
    BindSourceArrayModel[BIND_COUNT] {
        pub count: i32,
    }
}

model! {
    BindTargetArrayModel[BIND_COUNT] {
        pub count: i32,
    }
}

#[tokio::test]
async fn test_iterator_bind() {
    use frand_property::PropertyIteratorExt;
    use frand_property::ModelList;
    
    // Source: 값을 변경할 모델들 (Array Model)
    let source_models = BindSourceArrayModel::clone_singleton();
    
    // Target: 값이 전달될 타겟 모델들 (Array Model)
    let target_models = BindTargetArrayModel::clone_singleton();
    let target_senders = target_models.clone_senders();
    
    // Source의 receiver들을 Target의 sender들에 바인딩
    // source.count -> target.count
    source_models.iter()
        .map(|m| m.count.receiver())
        .spawn_bind(target_senders.iter().map(|s| s.count.clone()));
        
    // Source 값 변경
    source_models[0].count.sender().send(12345);
    source_models[4].count.sender().send(98765);
    
    // 비동기 전달 대기
    tokio::time::sleep(Duration::from_millis(50)).await;
    
    // Target 값 확인
    assert_eq!(target_models[0].count.receiver().value(), 12345);
    assert_eq!(target_models[1].count.receiver().value(), 0); // 변경 안됨
    assert_eq!(target_models[4].count.receiver().value(), 98765);
}

// 9. Tuple spawn_bind 테스트
#[tokio::test]
async fn test_tuple_spawn_bind() {
    let p1 = Property::from(1);
    let p2 = Property::from(2);
    
    // 바인딩할 타겟 속성
    let mut target = Property::from((0, 0));

    let tuple = (p1.receiver().clone(), p2.receiver().clone());
    
    // 튜플의 변경사항을 타겟 송신자에 바인딩
    tuple.spawn_bind(target.sender().clone());

    p1.sender().send(10);
    target.receiver_mut().modified().await;
    assert_eq!(target.receiver().value(), (10, 2));

    p2.sender().send(20);
    target.receiver_mut().modified().await;
    assert_eq!(target.receiver().value(), (10, 20));
}

use std::sync::LazyLock;

static LAZY_LEN: LazyLock<usize> = LazyLock::new(|| 4);

model! {
    LazyModel[*LAZY_LEN] {
        pub value: i32,
    }
}

#[test]
fn test_lazy_lock_len() {
    let models = LazyModel::clone_singleton();
    assert_eq!(models.len(), *LAZY_LEN);
    assert_eq!(models.len(), 4);
}
