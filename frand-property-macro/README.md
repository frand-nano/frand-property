# frand-property-macro

이 크레이트는 `frand-property` 생태계를 위한 프로시저 매크로(Procedural Macros)를 제공하며, 특히 Rust 데이터 모델과 Slint UI 간의 통합을 돕습니다.

## 개요

제공되는 주요 매크로는 `slint_model!`입니다. 이 매크로는 Rust 구조체를 Slint 속성 및 콜백에 바인딩하는 데 필요한 보일러플레이트 코드를 자동으로 생성해 줍니다.

## 사용법

이 크레이트는 일반적으로 `frand-property`에서 `slint` 기능이 활성화되었을 때 내부적으로 사용됩니다.

```rust
use frand_property::slint_model;

slint_model! {
    MyModel: MyData {
        out status_message: String,
        in on_click: (),
    }
}
```

- `out`: Rust에서 Slint로 흐르는 속성(상태)을 정의합니다.
- `in`: Slint에서 Rust로 흐르는 콜백이나 신호(이벤트)를 정의합니다.

## 라이선스

이 프로젝트는 MIT 라이선스 하에 배포됩니다. 자세한 내용은 [LICENSE](LICENSE) 파일을 참조하세요.