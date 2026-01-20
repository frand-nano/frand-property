# frand-property-macro

이 크레이트는 `frand-property` 생태계를 위한 프로시저 매크로(Procedural Macros)를 제공하며, Rust 데이터 모델과 Slint UI 간의 강력한 바인딩을 자동 생성합니다.

## `slint_model!` 매크로

`slint_model!`은 Rust 구조체 정의를 기반으로 다음 요소들을 자동으로 생성 및 연결합니다:
1. **Rust 모델 구조체**: `Receiver` 및 `Sender` 필드를 포함하여 데이터 흐름을 제어합니다.
2. **Slint 초기화 로직**: `SlintNotifyModel`을 사용하여 Slint의 Global 데이터와 Rust 모델을 동기화합니다.
3. **싱글톤 패턴 지원**: 모델 인스턴스를 전역 싱글톤으로 관리하여 어디서든 안전하게 접근할 수 있게 합니다.
4. **Slint 문서 (Doc Comments)**: 생성된 Slint 코드를 Rust 문서 주석으로 포함하여 IDE에서 확인할 수 있게 합니다.

## 사용법

```rust
use frand_property::slint_model;
use crate::MyData; // Slint에서 생성된 Struct

const ARRAY_LEN: usize = 4;   // 배열 필드 길이

slint_model! {
    // 1. 단일 모델 정의
    pub MyModel: MyData {
        in input_val: i32,
        out output_val: i32,
    }

    // 2. 배열 모델 정의 (매크로 단계에서는 단일 모델과 동일하게 정의)
    // 인스턴스 생성 시 MyVecModel::clone_singleton_vec::<N>()을 호출하여 Vec<MyVecModel>을 반환받습니다.
    pub MyVecModel: MyData {
        // 3. 배열 필드 정의 (Vec<Receiver<...>> 생성)
        in inputs: i32[ARRAY_LEN],
        out outputs: i32[ARRAY_LEN],
    }
}
```

### 키워드 및 타입 매핑

| 키워드 | 방향 | Slint 타입 | Rust 생성 타입 | 설명 |
|---|---|---|---|---|
| **`in`** | Slint → Rust | `in-out property` | `frand_property::Receiver<T>` | Slint에서 값이 변경되면 Rust에서 감지합니다. |
| **`out`** | Rust → Slint | `out property` | `frand_property::Sender<C, T>` | Rust에서 값을 보내면 Slint UI가 업데이트됩니다. |
| **`model`** | Internal (Rust) | (없음/무시됨) | `Struct` / `Vec<Struct>` | 다른 `Model`을 중첩하여 포함합니다. Slint 코드 생성에는 영향을 주지 않습니다. |
| **`[N]`** | - | `[type]` (배열) | `Vec<...>` | 필드 이름 뒤에 붙으면 해당 타입의 `Vec`을 생성하는 배열 필드가 됩니다. (모델 이름 뒤에는 붙이지 않습니다) |

### 고급 기능

#### 1. 배열 모델 생성 (`clone_singleton_vec`)
모델 정의 시에는 단일 모델처럼 정의하고, Rust 코드에서 인스턴스화할 때 `Model::clone_singleton_vec::<N>()` 메소드를 사용하여 `N`개의 모델이 담긴 `Vec<Model>`을 생성할 수 있습니다. 이는 리스트 뷰나 반복되는 UI 컴포넌트를 제어할 때 유용합니다.

#### 2. `ArrayString` 지원
`frand_property::arraystring::ArrayString` 타입을 사용하면 고정 길이 문자열을 효율적으로 처리할 수 있습니다. 이는 Slint의 `string` 타입과 자동으로 매핑되며, 변환 오버헤드를 줄여줍니다.

#### 3. Slint Global 네이밍 규칙
매크로는 Slint 측에 **`{ModelName}Global`**이라는 이름의 전역 객체가 존재한다고 가정합니다.
예를 들어 `pub AdderModel: AdderData { ... }`로 정의했다면, Slint 파일에는 다음과 같은 정의가 있어야 합니다:

```slint
export global AdderModelGlobal {
    in-out property <[AdderData]> data;
    // ...
}
```


#### 4. 중첩 모델 (Nested Models)
`model` 키워드를 사용하여 모델 내부에 다른 모델을 포함할 수 있습니다. 이는 Rust 코드 구조를 계층화하는 데 유용하지만, Slint 데이터 생성에서는 제외됩니다.

```rust
slint_model! {
    pub InnerModel: InnerData { in id: i32 }
}

slint_model! {
    pub OuterModel: OuterData {
        model inner: InnerModel,       // 단일 중첩
        model children: InnerModel[3], // 배열 중첩
        out name: String,
    }
}
```

## 에러 처리

- `out` 키워드는 유닛 타입 `()`과 함께 사용할 수 없습니다. (값을 전송해야 하므로)
- 배열 길이는 상수(`const`) 또는 정수 리터럴이어야 합니다.

## 라이선스

이 프로젝트는 MIT 라이선스 하에 배포됩니다. 자세한 내용은 [LICENSE](LICENSE) 파일을 참조하세요.