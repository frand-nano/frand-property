# frand-property-slint

이 프로젝트는 `frand-property` 라이브러리를 사용한 Slint UI 애플리케이션 예제입니다.

두 가지 주요 모델(`AdderModel`, `ScreenModel`)을 통해 상태 관리와 화면 전환 패턴을 보여줍니다.

## 프로젝트 구조

- **`slint/`**: Slint UI 정의 (`.slint`)
    - `components/adder.slint`: 덧셈 계산기 UI
    - `screen/`: 화면 전환 관리
- **`src/`**: Rust 로직
    - `adder.rs`: `AdderModel` 정의 및 로직 (`x + y = sum`)
    - `screen.rs`: `ScreenModel` 정의 및 화면 전환 로직
    - `main.rs`: 애플리케이션 진입점 및 시스템 시작

## 주요 기능

### 1. Adder (상태 동기화)
- **기능**: 두 숫자(X, Y)를 입력하면 합계(Sum)를 자동으로 계산합니다.
- **모델**: `AdderModel`
    - `in x`, `in y`: Slint의 버튼 클릭으로 변경된 값을 Rust가 감지합니다.
    - `out sum`: Rust가 계산한 합계를 Slint로 보냅니다.
- **동작**: `x`나 `y`가 변경될 때마다 `tokio::select!` 루프에서 감지하고 즉시 합계를 업데이트합니다.

### 2. Screen (이벤트 처리)
- **기능**: 시작 화면과 결제 화면 간의 전환을 관리합니다.
- **모델**: `ScreenModel`
    - `out current_screen`: 현재 표시할 화면 상태 (Enum)
    - `in confirm_start`: 시작 화면에서 확인 버튼 클릭 시 (이벤트)
    - `in cancel_pay`: 결제 화면에서 취소 버튼 클릭 시 (이벤트)
- **동작**: 이벤트를 수신하면 `current_screen` 상태를 변경하여 화면을 전환합니다.

## 실행 방법

### 필수 조건
- Rust (최신 안정 버전)
- Cargo

### 실행

```bash
cargo run
```

앱이 실행되면:
1. "Start" 화면이 표시됩니다.
2. 버튼을 눌러 상태 변화를 확인하거나 화면을 전환해 볼 수 있습니다.

## 라이선스

이 프로젝트는 MIT 라이선스 하에 배포됩니다. 자세한 내용은 [LICENSE](LICENSE) 파일을 참조하세요.