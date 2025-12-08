# frand-property-slint

이 프로젝트는 `frand-property`를 사용하여 반응형 상태 관리를 갖춘 Slint UI 애플리케이션을 구축하는 방법을 보여주는 예제 애플리케이션입니다.

## 프로젝트 구조

- **`slint/`**: `.slint` UI 정의 파일들이 위치합니다.
- **`src/`**: `slint_model!` 정의와 `tokio` 비동기 런타임 설정을 포함한 Rust 로직이 위치합니다.

## 시작하기

### 필수 조건

- Rust (최신 안정 버전)
- Cargo

### 애플리케이션 실행

```bash
cargo run
```

이 명령어를 실행하면 프로젝트가 컴파일되고 Slint 윈도우가 실행됩니다. 이 예제는 `ScreenModel`을 통해 입력을 처리하고 상태를 업데이트하는 간단한 흐름(예: 시작 화면 -> 결제 화면)을 보여줍니다.

## 라이선스

이 프로젝트는 MIT 라이선스 하에 배포됩니다. 자세한 내용은 [LICENSE](LICENSE) 파일을 참조하세요.