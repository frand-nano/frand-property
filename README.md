# frand-property

**frand-property**는 Rust와 Slint UI 간의 상태 동기화 및 비동기 로직 처리를 단순화하기 위한 라이브러리입니다.

`slint_model!` 매크로를 통해 프로퍼티 바인딩을 자동화하고, `System` 트레이트를 통해 비동기 업데이트 로직을 체계적으로 관리할 수 있습니다.

## 주요 기능

- **`slint_model!` 매크로**: Rust 구조체와 Slint 컴포넌트 간의 프로퍼티를 직관적으로 정의합니다.
    - `in`: Slint -> Rust (값 변경 감지 또는 이벤트 수신)
    - `out`: Rust -> Slint (값 전송)
- **`System` 트레이트**: 비동기 작업 루프(`tokio::spawn`, `tokio::select!`)를 표준화된 방식으로 관리합니다.
- **반응형 데이터 흐름**: Slint의 프로퍼티 변경을 Rust에서 `tokio` 채널 기반으로 감지하고 처리합니다.

## 설치

`Cargo.toml`에 의존성을 추가합니다.

```toml
[dependencies]
frand-property = "0.1.2" # 최신 버전 확인
slint = "1.8"
tokio = { version = "1", features = ["full"] }
```

## 사용 방법

### 1. Rust 모델 정의 (`slint_model!`)

`slint_model!` 매크로를 사용하여 Rust 측 모델을 정의합니다.

```rust
use frand_property::slint_model;
// Slint에서 생성된 모듈 경로 (build.rs 설정 필요)
use crate::AdderData; 

slint_model! {
    pub AdderModel: AdderData {
        // [in] Slint -> Rust
        // Slint의 값이 바뀌면 Rust에서 감지합니다.
        in x: i32,
        in y: i32,

        // [out] Rust -> Slint
        // Rust에서 값을 계산하여 Slint로 전송합니다.
        out sum: i32,
    }
}
```

### 2. Slint 파일 정의

`slint_model!`이 정의한 인터페이스에 맞춰 Slint 파일을 작성합니다.
AdderModel 의 문서화 주석에 생성된 slint 코드를 사용하면
Slint 측과 데이터를 연동할 구조체와 컴포넌트를 편리하게 정의할 수 있습니다.

```slint
export global AdderData {
    // [in] x: i32
    // in-out property로 정의하고 변경 콜백을 만들어야 합니다.
    // "changed-" 접두사 필수
    in-out property <int> x;
    callback changed-x(x: int);
    in-out property <int> y;
    callback changed-y(y: int);

    // [out] sum: i32
    // out 연산은 Rust가 값을 덮어쓰므로 in property로 정의합니다.
    in property <int> sum;
}

// 이 컴포넌트는 Rust 모델과 신호를 주고받는 연결 고리 역할을 합니다.
export component AdderDataSystem {
    // 값 변경 감지 연결
    property <int> system-x: AdderData.x;
    changed system-x => {
        AdderData.changed-x(system-x)
    }
    property <int> system-y: AdderData.y;
    changed system-y => {
        AdderData.changed-y(system-y)
    }
}
```

### 3. 시스템 로직 구현 (`System` trait)

`System` 트레이트를 구현하여 비즈니스 로직을 작성합니다.

```rust
use frand_property::System;

impl<C: slint::ComponentHandle + 'static> System for AdderModel<C> {
    fn start_system(&self) {
        // 채널 생성 (Clone하여 async 블록으로 이동)
        let mut x = self.x.receiver().clone();
        let mut y = self.y.receiver().clone();
        let sum = self.sum.sender().clone();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // x 값이 변경되었을 때
                    new_x = x.changed() => {
                        sum.send(new_x + y.value());
                    }
                    // y 값이 변경되었을 때
                    new_y = y.changed() => {
                        sum.send(x.value() + new_y);
                    }
                }
            }
        });
    }
}
```

### 4. 메인 함수에서 실행

```rust
#[tokio::main]
async fn main() -> Result<(), slint::PlatformError> {
    let window = MainWindow::new()?;

    // 모델 생성
    let adder_model = AdderModel::<MainWindow>::new(&window);
    
    // 시스템 시작 (비동기 루프 실행)
    adder_model.start_system();

    window.run()?;
    Ok(())
}
```

## 구조

- **`frand-property-macro`**: `slint_model!` 프로시저럴 매크로
- **`frand-property`**: `System` 트레이트 및 런타임 헬퍼
- **`frand-property-slint`**: 예제 프로젝트

## 라이선스

이 프로젝트는 MIT 라이선스 하에 배포됩니다. 자세한 내용은 [LICENSE](LICENSE) 파일을 참조하세요.