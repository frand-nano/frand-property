# frand-property-macro

이 크레이트는 `frand-property` 생태계를 위한 프로시저 매크로(Procedural Macros)를 제공하며, 특히 Rust 데이터 모델과 Slint UI 간의 통합을 돕습니다.

## 개요

제공되는 주요 매크로는 `slint_model!`입니다. 이 매크로는 Rust 구조체를 Slint 속성 및 콜백에 바인딩하는 데 필요한 보일러플레이트 코드를 자동으로 생성해 줍니다. 배열 타입의 프로퍼티도 지원하여 반복되는 UI 요소를 쉽게 제어할 수 있습니다.

## 사용법

이 크레이트는 일반적으로 `frand-property`에서 `slint` 기능이 활성화되었을 때 내부적으로 사용됩니다.

```rust
use frand_property::slint_model;
// Slint에서 생성된 구조체 (예: AdderData)
use crate::AdderData;

const LEN: usize = 2;

slint_model! {
    pub AdderModel: AdderData {
        in x: i32,           // Slint -> Rust (단일 입력)
        out sum: i32,        // Rust -> Slint (단일 출력)

        // 배열 지원
        in inputs: i32[LEN], // 길이 2의 배열 입력
        out outputs: i32[2], // 길이 2의 배열 출력
    }
}
```

### 키워드 설명

- **`in`**: Slint UI에서 값이 변경되면 Rust 모델에 알림(`changed()`)을 보냅니다.
    - Rust 모델에는 `Receiver<T>` 또는 `Vec<Receiver<T>>` 타입 필드가 생성됩니다.
    - 배열의 경우, 인덱스와 함께 변경 사항을 감지할 수 있습니다.
- **`out`**: Rust 모델에서 값을 변경하여 Slint UI로 전송(`send()`)합니다.
    - Rust 모델에는 `Sender<C, T>` 또는 `Vec<Sender<C, T>>` 타입 필드가 생성됩니다.
    - **제약 사항**: `()` (유닛 타입)은 값을 가질 수 없으므로 `out` 키워드와 함께 사용할 수 없습니다.

`pub` 키워드를 사용하여 생성된 모델 구조체의 가시성을 제어할 수 있습니다.

## 배열 프로퍼티

배열 프로퍼티는 `name: type[len]` 문법으로 정의합니다. `len`은 `usize` 타입의 상수이거나 리터럴이어야 합니다.
배열을 사용하면 Rust에서는 `Vec<Receiver<...>>` 또는 `Vec<Sender<...>>` 형태로 관리되며, Slint 컴포넌트와 자동으로 바인딩됩니다.

## 라이선스

이 프로젝트는 MIT 라이선스 하에 배포됩니다. 자세한 내용은 [LICENSE](LICENSE) 파일을 참조하세요.