# frand-property-macro

이 크레이트는 `frand-property` 생태계를 위한 프로시저 매크로(Procedural Macros)를 제공하며, 특히 Rust 데이터 모델과 Slint UI 간의 통합을 돕습니다.

## 개요

제공되는 주요 매크로는 `slint_model!`입니다. 이 매크로는 Rust 구조체를 Slint 속성 및 콜백에 바인딩하는 데 필요한 보일러플레이트 코드를 자동으로 생성해 줍니다.

## 사용법

이 크레이트는 일반적으로 `frand-property`에서 `slint` 기능이 활성화되었을 때 내부적으로 사용됩니다.

```rust
use frand_property::slint_model;
// Slint에서 생성된 구조체 (예: AdderData)
use crate::AdderData;

slint_model! {
    pub AdderModel: AdderData {
        in x: i32,         // Slint -> Rust (입력)
        out sum: i32,      // Rust -> Slint (출력)
        in-out data: i32,  // 양방향 동기화
    }
}
```

- `in`: Slint UI에서 값이 변경되면 Rust 모델에 알림(`changed()`)을 보냅니다.
- `out`: Rust 모델에서 값을 변경하여 Slint UI로 전송(`send()`)합니다.
- `in-out`: 양방향 동기화를 지원합니다.

`pub` 키워드를 사용하여 생성된 모델 구조체의 가시성을 제어할 수 있습니다.

## 라이선스

이 프로젝트는 MIT 라이선스 하에 배포됩니다. 자세한 내용은 [LICENSE](LICENSE) 파일을 참조하세요.