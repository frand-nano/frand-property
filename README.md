# frand-property

**frand-property**는 Rust와 Slint UI 간의 상태 동기화 및 비동기 로직 처리를 단순화하기 위한 라이브러리입니다.

`slint_model!` 매크로를 통해 단일 값뿐만 아니라 배열 형태의 프로퍼티 바인딩을 자동화하고, `System` 트레이트를 통해 비동기 업데이트 로직을 체계적으로 관리할 수 있습니다.

## 주요 기능

- **`slint_model!` 매크로**: Rust 구조체와 Slint 컴포넌트 간의 프로퍼티를 직관적으로 정의합니다.
    - `in`: Slint -> Rust (값 변경 감지 또는 이벤트 수신)
    - `out`: Rust -> Slint (값 전송)
    - **배열 지원**: `type[len]` 구문으로 여러 개의 프로퍼티를 한 번에 선언하고 관리할 수 있습니다.
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

`slint_model!` 매크로를 사용하여 Rust 측 모델을 정의합니다. 배열 문법을 사용할 수 있습니다.

```rust
use frand_property::slint_model;
// Slint에서 생성된 모듈 경로 (build.rs 설정 필요)
use crate::AdderData; 

const LEN: usize = 2;

slint_model! {
    pub AdderModel: AdderData {
        // [in] Slint -> Rust
        // 배열로 정의하면 Rust에서는 Vec<Receiver<...>>가 됩니다.
        in x: i32[LEN],
        in y: i32[LEN],

        // [out] Rust -> Slint
        // 배열로 정의하면 Rust에서는 Vec<Sender<...>>가 됩니다.
        out sum: i32[LEN],
    }
}
```

### 2. Slint 파일 정의

Slint 파일에서는 전역 데이터(Global)와 이를 사용하는 로직을 정의합니다. 배열을 사용하는 경우 반복문(`for`)을 활용하여 UI를 구성할 수 있습니다.

```slint
export global AdderData {
    // Slint 모델 정의
    // in: 변경 시 Rust로 알림이 전달됩니다.
    in property <[int]> x: [0, 0];
    in property <[int]> y: [0, 0];
    
    // out: Rust에서 값을 변경하면 UI에 반영됩니다.
    in property <[int]> sum: [0, 0];
    
    // 배열 요소 변경 알림을 위한 콜백 (in 프로퍼티용)
    callback changed-x(int, int); // index, value
    callback changed-y(int, int); // index, value
}

// ... UI 컴포넌트 구현 ...
```
*(참고: 실제 Slint 코드는 `slint_model!` 매크로가 기대하는 인터페이스(콜백 이름 등)에 맞춰 작성해야 합니다.)*

### 3. 시스템 로직 구현 (`System` trait)

`System` 트레이트를 구현하여 비즈니스 로직을 작성합니다. 배열의 각 요소를 독립적으로 제어할 수 있습니다.

```rust
use frand_property::System;

impl<C: slint::ComponentHandle + 'static> System for AdderModel<C> {
    fn start_system(&self) {
        // 배열의 각 요소에 대한 채널에 접근
        // slint_model!로 생성된 필드는 Vec<Receiver<T>> (in) 또는 Vec<Sender<C, T>> (out) 타입입니다.
        let mut x_0 = self.x[0].clone();
        let mut y_0 = self.y[0].clone();
        let sum_0 = self.sum[0].clone();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // 0번 인덱스의 x 값이 변경되었을 때
                    new_x = x_0.changed() => {
                        sum_0.send(new_x + y_0.value());
                    }
                    // 0번 인덱스의 y 값이 변경되었을 때
                    new_y = y_0.changed() => {
                        sum_0.send(x_0.value() + new_y);
                    }
                }
            }
        });
        // 반복문 등을 사용하여 모든 인덱스에 대한 로직을 설정할 수도 있습니다.
    }
}
```

### 4. 메인 함수에서 실행

`main.rs`에서 모델을 초기화하고 시스템을 시작합니다.

```rust
#[tokio::main]
async fn main() -> Result<(), slint::PlatformError> {
    let window = MainWindow::new()?;

    // 모델 생성 및 시스템 시작
    let adder_model = AdderModel::<MainWindow>::new(&window);
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