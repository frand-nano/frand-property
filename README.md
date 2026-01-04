# frand-property

**frand-property**는 Rust와 Slint UI 간의 상태 동기화 및 비동기 로직 처리를 단순화하기 위한 라이브러리입니다.

`slint_model!` 매크로를 통해 복잡한 보일러플레이트 없이 Rust 구조체와 Slint 컴포넌트를 바인딩하고, `SlintNotifyModel`을 통해 효율적인 반응형 데이터 흐름을 구현할 수 있습니다.

## 주요 기능

- **`slint_model!` 매크로**: Rust 모델 정의로부터 Slint 바인딩 코드를 자동 생성합니다.
    - **자동화된 동기화**: `in` / `out` 키워드로 데이터 흐름 방향을 직관적으로 정의합니다.
    - **배열 모델 지원**: `Model[N]` 문법으로 동일한 로직을 가진 다수의 모델 인스턴스를 쉽게 생성합니다.
    - **배열 필드 지원**: `field: type[N]` 문법으로 고정 길이 배열 프로퍼티를 처리할 수 있습니다.
- **반응형 상태 관리**: Rust의 `System` 트레이트와 `tokio` 비동기 런타임을 결합하여 UI 이벤트를 효율적으로 처리합니다.
- **최적화된 렌더링**: 내부적으로 `SlintNotifyModel`을 사용하여 변경된 데이터만 세밀하게 업데이트하므로 불필요한 리렌더링을 방지합니다.

## 설치

`Cargo.toml`에 의존성을 추가합니다.

```toml
[dependencies]
frand-property = "0.2.3"
slint = "1.8"
tokio = { version = "1", features = ["full"] }
```

## 사용 방법

### 1. Rust 모델 정의 (`slint_model!`)

`slint_model!` 매크로를 사용하여 Rust 측 모델을 정의합니다.

```rust
use frand_property::slint_model;
// Slint에서 생성된 구조체 (build.rs 설정 필요)
// Slint 파일의 'export struct AdderData'에 대응합니다.
use crate::AdderData; 

slint_model! {
    // AdderData 구조체와 바인딩되는 AdderModel 정의
    pub AdderModel: AdderData {
        // [in] Slint -> Rust (값 변경 감지)
        // Rust 타입: frand_property::Receiver<i32>
        in x: i32,
        in y: i32,

        // [out] Rust -> Slint (값 전송 via Channel)
        // Rust 타입: frand_property::Sender<C, i32>
        out sum: i32,
    }
}
```

만약 여러 개의 모델 인스턴스가 필요하거나 필드가 배열인 경우 다음과 같이 정의할 수 있습니다:

```rust
const MODEL_LEN: usize = 2; // 모델 인스턴스 개수
const PROP_LEN: usize = 5;  // 각 필드의 배열 길이

slint_model! {
    // AdderArrayModel 인스턴스를 MODEL_LEN 개 생성 (Vec<AdderArrayModel> 반환)
    pub AdderArrayModel[MODEL_LEN]: AdderArrayData {
        // [in] 배열 필드
        // Rust 타입: Vec<frand_property::Receiver<i32>>
        in values: i32[PROP_LEN],
        
        // [out] 단일 필드
        // Rust 타입: frand_property::Sender<C, i32>
        out sum: i32,
    }
}
```

### 2. Slint 파일 정의

Slint 파일에서는 데이터 구조체(`struct`)와 이를 담을 전역 싱글톤(`global`)을 정의해야 합니다. 매크로는 `{ModelName}Global`이라는 이름의 글로벌 객체와 상호작용합니다.

```slint
// 1. 데이터 구조체 정의 (Rust의 AdderData와 매핑)
export struct AdderData {
    x: int,
    y: int,
    sum: int,
}

// 2. 전역 싱글톤 정의 (이름은 반드시 {ModelName}Global 규칙을 따라야 함)
export global AdderModelGlobal {
    // Rust에서 이 배열 데이터를 관리합니다 (SlintNotifyModel 사용)
    in-out property <[AdderData]> data;
}

// 3. 컴포넌트 구현
component AdderComponent inherits Rectangle {
    in-out property <int> index; // 모델 인덱스

    // Global 데이터 바인딩
    // (매크로가 생성하는 코드는 내부적으로 Global.data[index]를 참조합니다)
    // 실제 UI 구현...
}
```

### 3. 시스템 로직 구현 (`System` trait)

`System` 트레이트를 구현하여 상태 변경에 따른 비즈니스 로직을 작성합니다.

```rust
use frand_property::System;

impl<C: slint::ComponentHandle + 'static> System for AdderModel<C> {
    fn start_system(&self) {
        // Receiver / Sender 복제 (비동기 클로저로 이동)
        let mut x = self.x.clone();
        let mut y = self.y.clone();
        let sum = self.sum.clone();

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

`main.rs`에서 모델을 초기화하고 시스템을 시작합니다.

```rust
#[tokio::main]
async fn main() -> Result<(), slint::PlatformError> {
    let window = MainWindow::new()?;

    // 모델 생성 (Slint Global과 바인딩됨)
    let adder_model = AdderModel::<MainWindow>::new(&window);
    
    // 시스템 로직 시작 (비동기 루프 실행)
    adder_model.start_system();

    window.run()?;
    Ok(())
}
```

## 구조

- **`frand-property-macro`**: `slint_model!` 프로시저럴 매크로 구현체
- **`frand-property`**: 런타임 라이브러리 (Property, SlintNotifyModel, System trait 등)
- **`frand-property-slint`**: 전체 기능을 보여주는 Slint 예제 프로젝트

## 라이선스

이 프로젝트는 MIT 라이선스 하에 배포됩니다. 자세한 내용은 [LICENSE](LICENSE) 파일을 참조하세요.