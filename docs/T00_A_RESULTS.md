# T00-A 검증 기록

상태: **진행 중 — Windows 실기·IPC/수명 증거 미확보. 기술 미채택.**

시작 기준: main `4c3537c55084b71feb5c42eb2340005bf922f197`, 기본 브랜치 main,
깨끗한 로컬 main 및 원격 main만 존재, 열린 PR 0개 확인. 작업 브랜치 task/t00-a-technical-spike.
현재 결과/수집 PR: [#1](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/pull/1).

## 증거 구분

| 항목 | 정확한 코드·환경·명령 | 관찰 결과 / 합격 기준 |
| --- | --- | --- |
| Linux 독립 엔진/CLI/MCP | `b4a8483168bf7a93e797382bef156fa99c761db2`, Rust 1.99.0, Python 3.12.14, Linux x64. `cargo build --locked --release`; `python scripts/verify.py --bin target/release --sha <SHA> --out <report>` | 자동 검사 통과. 원시 보고서 [linux-automatic.json](t00-a-evidence/linux-automatic.json). Windows 증거 아님 |
| GUI UI | 같은 코드, Node 24.19.0; `npm ci && npm run build` | TypeScript·Vite 빌드 통과. GUI 실행·명령 호출 증거 아님 |
| 공통 규칙·동시성 | 실제 Rust 프로세스, SQLite WAL. 같은 request 함수·엔진 명령. 8 CLI 동시 동일 ID/다른 ID | 동일 ID 최초 1건만 수락, 인자 다르면 거절, 모의 예산 10 초과 거절. MCP 조회/거절 결과 동일; 실제 크레딧 통제 근거 아님 |
| 수명·종료 | CLI 종료 뒤 작업, 엔진 중복 기동, 정상 종료 및 kill 후 재기동 | 작업 지속·새 CLI 재연결, 중복 엔진 거절, 중단 상태/예약 보존·자동 재실행 없음. GUI 닫기 실기는 별도 |
| 파일/DB 종료 틈 | after_temp / after_rename fault injection | interrupted와 잔여 파일 확인. 자동 대조 복구 미구현이라는 한계 발견. 복구 기능 완료 아님 |
| 한글·오디오 | 임시 한글/공백 경로. FFmpeg/ffprobe 6.1.1. 1초 합성 WAV 생성 → 별도 fade/24kHz WAV 후보 | 길이/샘플레이트와 원본 SHA256 보존 통과. 실제 사용자 음원·MP3/OGG·WebView2 디코딩·청감 미검증 |
| 직접 접근 비교 | Python sqlite3 두 connection begin immediate | 두 번째 writer 잠금 재현. Rust 직접 접근 완성 대안/성능 비교 아님 |
| 자격증명 후보 | Windows dummy roundtrip 스크립트 준비 | Linux에서는 미실행. Windows 결과는 다음 절에 기록. Key 사용·서비스 호출 없음 |
| Windows CI/EXE/artifact | 아래 CI 결과 절 | Linux 결과·문서로 성공 대체 금지 |
| 사용자 GUI/재연결/재생 | docs/T00_A_WINDOWS_CHECK.md + Collect-UserResult.ps1 | 사용자 결과 미수령. GUI 프로세스 생존을 화면/명령/청감 확인으로 집계하지 않음 |

## Windows CI 결과

첫 코드 SHA `b4a8483168bf7a93e797382bef156fa99c761db2`의
[run 38692091599](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/38692091599) 실패: Tauri Windows resource 생성에 기본 icon.ico가 없어 GUI 빌드 중단.
CLI/엔진/MCP 빌드 후 GUI 실패했으며 자동 실행은 건너뛰었다. 실패 artifact `11952822204`는 GUI가 없는 진단용이며 실기용으로 사용하지 않는다.
후속 수정은 결정적 fixture 아이콘을 build.rs에서 생성하고 custom-protocol feature로 정적 GUI 자산을 포함한다. 수정 SHA의 CI를 다시 검증한다.

## 미검증과 재개 조건

1. 정확한 최신 코드 SHA Windows GUI 빌드와 artifact를 확보하고 자동 결과·실제 사용자 결과를 분리 검토한다.
2. 사용자 Windows에서 GUI 공통 상태/작업, 창 종료 후 작업 지속·CLI 조회·GUI 재연결, 합성 음원 실제 청감을 기록한다. 빈 템플릿은 성공 증거가 아니다.
3. 사용자 ACL named pipe 또는 검토·승인된 IPC의 타 사용자 차단, 사용자별 소유권·동시 첫 기동·자동 기동/idle 종료 정책을 추가 실험한다. 현재 TCP/token은 보안 채택 근거가 아니다.
4. 파일/DB 틈 복구 계약, 실제 API의 불확실 비용과 재시도 계약을 분리한다. 이번 fixture 비용/시간값을 제품 기본으로 사용하지 않는다.
5. D01~D06 근거를 고성능 Chat/Work에서 사용자와 검토한다. T00-A 실기/설계 게이트 및 사용자 결정 전 T00-B 금지.

Windows artifact는 합성 신호·EXE·스크립트·SHA/도구 정보·해시만 포함하며 FFmpeg 외부 바이너리, 임시 DB·endpoint token·실제 음원은 포함하지 않는다. FFmpeg 재배포 적합성·제품 라이선스는 판정하지 않았다.
