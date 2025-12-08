# frand-property

`frand-property`는 효율적인 속성 관리와 상태 동기화를 위해 설계된 Rust 라이브러리로, 특히 [Slint](https://slint.dev/) UI 툴킷과의 통합에 최적화되어 있습니다.

이 라이브러리는 애플리케이션 상태를 반응형(Reactive)으로 처리하는 방식을 제공하여, 비즈니스 로직과 UI 컴포넌트를 쉽게 연결할 수 있도록 돕습니다.

## 주요 기능

- **반응형 속성 (Reactive Properties)**: 상태 변경을 비동기적으로 관리합니다.
- **Slint 통합**: `slint` 기능(feature)과 헬퍼 매크로를 통해 Slint와 매끄럽게 바인딩됩니다.
- **비동기 지원**: 견고한 비동기 런타임 지원을 위해 `tokio` 기반으로 구축되었습니다.

## 설치

`Cargo.toml` 파일에 다음을 추가하세요:

```toml
[dependencies]
frand-property = { version = "0.1.0", features = ["slint"] }
```

## 사용법

데이터 모델과 바인딩을 정의할 때는 `slint_model!` 매크로를 사용합니다 (이 매크로는 `slint` 기능이 활성화되면 `frand-property-macro`를 통해 제공됩니다).

```rust
use frand_property::*;

// 사용 예시
slint_model! {
    AppModel: AppData {
        out counter: i32,
        in increment: (),
    }
}
```

## 라이선스

이 프로젝트는 MIT 라이선스 하에 배포됩니다. 자세한 내용은 [LICENSE](LICENSE) 파일을 참조하세요.