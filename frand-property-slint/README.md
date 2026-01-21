# frand-property-slint

이 프로젝트는 `frand-property` 라이브러리를 활용한 Slint UI 애플리케이션 예제입니다.

다양한 데이터 동기화 패턴(단일 값, 배열, 이벤트, 화면 전환)을 `slint_model!` 매크로와 `SlintNotifyModel`을 통해 어떻게 구현하는지 보여줍니다.

## 프로젝트 구조

- **`slint/`**: Slint UI 정의 파일
    - `main.slint`: 메인 윈도우 및 글로벌 정의
    - `components/`: UI 컴포넌트 (`adder.slint`, `repeater.slint` 등)
    - `screen/`: 화면별 레이아웃 (`start.slint`, `pay.slint`)
- **`src/`**: Rust 로직 구현
    - `main.rs`: 애플리케이션 진입점 및 시스템 초기화 (싱글톤 Setup)
    - `adder.rs`: 단일 모델 패턴 예제 (`AdderModel`)
    - `adder_array.rs`: 배열 모델 및 배열 필드 패턴 예제 (`AdderVecModel`)
    - `screen.rs`: 화면 전환 로직 예제 (`ScreenModel`)
    - `repeater.rs`: 반복 UI 패턴 예제 (`RepeaterModel`)

## 예제 시나리오

### 1. Adder (기본 양방향 바인딩)
- **파일**: `src/adder.rs`, `slint/components/adder.slint`
- **설명**: 두 개의 입력값(`x`, `y`)을 받아 합계(`sum`)를 실시간으로 계산합니다.
- **특징**:
    - `in` / `out` 프로퍼티의 가장 기본적인 사용법을 보여줍니다.
    - `tokio::select!`를 사용하여 변경 사항을 비동기적으로 처리합니다.

### 2. Adder Vec (고급 배열 처리)
- **파일**: `src/adder_array.rs`
- **설명**: 여러 개의 입력값을 배열(`values: i32[3]`)로 받아 그 합계를 계산합니다.
- **특징**:
    - `slint_model!`에서 배열 필드를 정의하는 방법을 보여줍니다 (`values: i32[N]`).
    - **`FuturesUnordered`**를 사용하여 배열 내의 개별 요소 변경을 효율적으로 감지하고 처리하는 패턴을 제시합니다.
    - 모델을 동적으로 여러 개 생성하는 방법(`AdderVecModel[N]` 정의 및 `clone_singleton` 사용)을 보여줍니다.

### 3. Screen (이벤트 및 상태 관리)
- **파일**: `src/screen.rs`
- **설명**: 버튼 클릭 이벤트를 수신하여 화면 상태(`current_screen`)를 변경합니다.
- **특징**:
    - Slint의 콜백(버튼 클릭 등)을 Rust의 `in` 프로퍼티(`Sender<()>` -> `Receiver<()>`)로 매핑하여 이벤트를 처리합니다.
    - Enum 값을 정수(int)로 변환하여 화면 인덱스를 제어합니다.

## 실행 방법

### 필수 조건
- Rust (최신 안정 버전)
- Cargo
- `wasm-pack` (웹/WASM 실행 시 필요)

### 데스크탑 실행

```bash
cargo run
```

### 웹 (WASM) 실행

1. 의존성 설치 (필요한 경우)
    ```bash
    cargo install wasm-pack
    ```
2. WASM 빌드
    ```bash
    wasm-pack build --target web
    ```
3. 로컬 서버 실행 및 접속
    ```bash
    python3 -m http.server 8000
    # 브라우저에서 http://localhost:8000 접속
    ```

앱이 실행되면:
1. **Start Screen**: 기본 계산기 예제들과 화면 전환 버튼이 표시됩니다.
2. **Interactive Elements**: 각 계산기의 입력값을 변경하면 즉시 합계가 업데이트되는 것을 확인할 수 있습니다.
3. **Navigation**: "Confirm" 버튼을 눌러 결제 화면 등으로 전환하며 상태 관리를 테스트할 수 있습니다.

## 라이선스

이 프로젝트는 MIT 라이선스 하에 배포됩니다. 자세한 내용은 [LICENSE](LICENSE) 파일을 참조하세요.