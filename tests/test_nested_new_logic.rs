use frand_property::model;

// 내부 모델은 new()를 지원하기 위해 반드시 public이어야 합니다.
model! {
    pub InnerBasic {
        val: i32
    }
}

model! {
    pub OuterBasic {
        model inner: InnerBasic
    }
}

#[test]
fn test_nested_independence() {
    let outer1 = OuterBasic::clone_singleton();
    let outer2 = OuterBasic::clone_singleton();

    // 초기 상태 확인
    assert_eq!(outer1.inner.val.receiver().value(), 0);
    assert_eq!(outer2.inner.val.receiver().value(), 0);

    // outer1의 내부 모델 수정
    outer1.inner.val.sender().send(100);

    // outer1이 변경되었는지 확인
    assert_eq!(outer1.inner.val.receiver().value(), 100);

    // outer2는 이제 싱글톤이므로 같이 변경되어야 함
    assert_eq!(outer2.inner.val.receiver().value(), 100);
}

// Slice/Array 테스트
// 참고: 명시적 배열 필드 문법 `[Inner; N]`은 모델에 대해 macro에서 금지되어 있습니다.
// 하지만 배열 그 자체인 모델을 정의할 수는 있습니다.
model! {
    pub InnerList[2] {
        val: i32
    }
}

model! {
    pub OuterList {
        // 암시적 길이 []를 사용하며, 이는 generate_init_fields에서 Slice/Array 로직에 매핑됩니다.
        model list: InnerList[]
    }
}

#[test]
fn test_nested_list_independence() {
    let outer1 = OuterList::clone_singleton();
    let outer2 = OuterList::clone_singleton();
    
    // 초기값 확인
    assert_eq!(outer1.list[0].val.receiver().value(), 0);
    assert_eq!(outer2.list[0].val.receiver().value(), 0);
    
    // outer1 수정
    outer1.list[0].val.sender().send(50);
    
    // outer1 확인
    assert_eq!(outer1.list[0].val.receiver().value(), 50);
    
    // outer2도 변경되어야 함 (싱글톤)
    assert_eq!(outer2.list[0].val.receiver().value(), 50);
}
