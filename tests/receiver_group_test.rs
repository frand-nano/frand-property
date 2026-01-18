use frand_property::{Property, ReceiverGroup};

#[tokio::test]
async fn test_tuple_literal_usage() {
    let p1 = Property::from(1);
    let p2 = Property::from(2);
    let p3 = Property::from(3);

    // 튜플 리터럴 사용
    let tuple_literal = (p1.receiver().clone(), p2.receiver().clone(), p3.receiver().clone());
    assert_eq!(tuple_literal.value(), (1, 2, 3));
}

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
    // 비동기 전파 대기
    target.receiver_mut().changed().await;
    assert_eq!(target.receiver().value(), (10, 2));
    
    p2.sender().send(20);
    target.receiver_mut().changed().await;
    assert_eq!(target.receiver().value(), (10, 20));
}
