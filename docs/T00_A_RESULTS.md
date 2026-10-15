# T00-A 검증 기록

상태: **진행 중 — Windows 자동 실행 통과, 사용자 GUI/청감·IPC/수명 게이트 대기. 기술 미채택.**

시작 기준: main `4c3537c55084b71feb5c42eb2340005bf922f197`, 기본 브랜치 main,
깨끗한 로컬 main 및 원격 main만 존재, 열린 PR 0개 확인. 작업 브랜치 task/t00-a-technical-spike.
현재 결과/수집 PR: [#1](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/pull/1).

## 증거 구분

| 항목 | 정확한 코드·환경·명령 | 관찰 결과 / 합격 기준 |
| --- | --- | --- |
| Linux 독립 엔진/CLI/MCP | `80993133331c134e8a6bea12c7a9c8e0f7493a4e`, Rust 1.99.0, Python 3.12.14, Linux x64. `cargo build --locked --release`; `python scripts/verify.py --bin target/release --sha <SHA> --out <report>` | 자동 검사 통과. 원시 보고서 [linux-automatic.json](t00-a-evidence/linux-automatic.json). Windows 증거 아님 |
| GUI UI | 같은 코드, Node 24.19.0; `npm ci && npm run build` | TypeScript·Vite 빌드 통과. GUI 실행·명령 호출 증거 아님 |
| 공통 규칙·동시성 | 실제 Rust 프로세스, SQLite WAL. 같은 request 함수·엔진 명령. 8 CLI 동시 동일 ID/다른 ID | 동일 ID 최초 1건만 수락, 인자 다르면 거절, 모의 예산 10 초과 거절. MCP 조회/거절 결과 동일; 실제 크레딧 통제 근거 아님 |
| 수명·종료 | CLI 종료 뒤 작업, 엔진 중복/동시 첫 기동, CLI/MCP 혼합 경쟁, 정상 종료 및 kill 후 재기동 | 작업 지속·새 CLI 재연결, 중복 엔진 거절, 중단 상태/예약 보존·자동 재실행 없음. GUI 닫기 실기는 별도 |
| 파일/DB 종료 틈 | after_temp / after_rename fault injection | interrupted와 잔여 파일 확인. 자동 대조 복구 미구현이라는 한계 발견. 복구 기능 완료 아님 |
| 한글·오디오 | 임시 한글/공백 경로. FFmpeg/ffprobe 6.1.1. 1초 합성 WAV 생성 → 별도 fade/24kHz WAV 후보 | 길이/샘플레이트와 원본 SHA256 보존 통과. 실제 사용자 음원·MP3/OGG·WebView2 디코딩·청감 미검증 |
| 직접 접근 비교 | Python sqlite3 두 connection begin immediate | 두 번째 writer 잠금 재현. Rust 직접 접근 완성 대안/성능 비교 아님 |
| 자격증명 후보 | Windows Credential Manager dummy 쓰기/읽기/삭제, DPAPI CurrentUser 메모리 왕복 | Windows CI 실행 통과. 다른 사용자·재부팅·프로필 이동·실제 Key·내보내기 경계 미검증. 서비스 호출 없음 |
| Windows CI/EXE/artifact | `80993133331c134e8a6bea12c7a9c8e0f7493a4e`, Windows Server 2022 x64, Rust/Cargo 1.99.0, Node 24.21.0, Python 3.12.10, FFmpeg/ffprobe 9.0.2 | GUI/CLI/엔진/MCP 빌드, 16개 자동 검사 통과. GUI는 프로세스 3초 생존만 확인. 사용자 화면·명령·청감 성공 아님 |
| 사용자 GUI/재연결/재생 | docs/T00_A_WINDOWS_CHECK.md + Collect-UserResult.ps1 | 사용자 결과 미수령. GUI 프로세스 생존을 화면/명령/청감 확인으로 집계하지 않음 |

## Windows CI 결과

검증 코드 SHA: **`80993133331c134e8a6bea12c7a9c8e0f7493a4e`**.
[성공 run 38693413115](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/38693413115),
[artifact 11953730364](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/38693413115/artifacts/11953730364).
원시 [Windows 자동 실행 결과](t00-a-evidence/windows-automatic.json), [artifact 검사](t00-a-evidence/artifact-inspection.json).

ZIP SHA256 `79ee05c2ce61dce573e905af3833b9f2257d15aceab0252d9cce8d3f20455e7c`.
ZIP을 내려받아 source-sha.txt=자동 보고서=run head SHA, 모든 SHA256SUMS 파일 해시를 대조했다.
4개 EXE의 PE x64 헤더를 검사했다. GUI 8,664,064 bytes, CLI 290,304 bytes, 엔진 1,970,688 bytes, MCP 323,584 bytes.
이 수치는 WebView2·FFmpeg 등 외부 준비 항목을 포함한 제품 배포 크기가 아니다. artifact 보존 기간은 14일이다.

자동 통과: 동시 첫 수동 엔진 기동·중복 기동 거절, 잘못된 token 거절, 공통 검증·apply 거절,
동시 동일 ID/충돌, CLI 종료 후 작업 지속·재연결, CLI/MCP 동시 모의 예산,
stdio handshake/도구/오류/stdout, kill·정상 종료 후 interrupted·예약 보존, SQLite 무결성,
파일/DB 틈 한계 재현, 한글 경로 FFmpeg/ffprobe·원본 hash 보존, 직접 SQLite writer fixture,
dummy 자격증명 2후보, GUI 프로세스 3초 생존.
GUI 명령 invoke·화면·실제 소리는 자동 보고서에서 unverified이며 사용자 실기가 필요하다.
후속 문서 커밋은 실행 소스 변경과 구분하며 PR에서 해당 HEAD CI를 별도로 확인한다.

### 실패를 통해 보완한 부분

첫 코드 SHA `b4a8483168bf7a93e797382bef156fa99c761db2`의
[run 38692091599](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/38692091599) 실패: Tauri Windows resource 생성에 기본 icon.ico가 없어 GUI 빌드 중단.
CLI/엔진/MCP 빌드 후 GUI 실패했으며 자동 실행은 건너뛰었다. 실패 artifact `11952822204`는 GUI가 없는 진단용이며 실기용으로 사용하지 않는다.
후속 수정은 결정적 fixture 아이콘을 build.rs에서 생성하고 custom-protocol feature로 정적 GUI 자산을 포함한다. 수정 SHA의 CI를 다시 검증한다.

## 미검증과 재개 조건

1. 위 artifact 또는 동일 실행 코드의 후속 정확한 SHA artifact를 사용해 사용자 결과를 수집한다. 최신 HEAD와 실행 소스 변경 여부를 대조하고 SHA가 다른 결과를 무조건 전용하지 않는다.
2. 사용자 Windows에서 GUI 공통 상태/작업, 창 종료 후 작업 지속·CLI 조회·GUI 재연결, 합성 음원 실제 청감을 기록한다. 빈 템플릿은 성공 증거가 아니다.
3. 사용자 ACL named pipe 또는 검토·승인된 IPC의 타 사용자 차단, 사용자 전체 소유권·클라이언트 자동 기동/idle 종료 정책을 추가 실험한다. 현재 TCP/token은 보안 채택 근거가 아니다.
4. 파일/DB 틈 복구 계약, 실제 API의 불확실 비용과 재시도 계약을 분리한다. 이번 fixture 비용/시간값을 제품 기본으로 사용하지 않는다.
5. D01~D06 근거를 고성능 Chat/Work에서 사용자와 검토한다. T00-A 실기/설계 게이트 및 사용자 결정 전 T00-B 금지.

Windows artifact는 합성 신호·EXE·스크립트·SHA/도구 정보·해시만 포함하며 FFmpeg 외부 바이너리, 임시 DB·endpoint token·실제 음원은 포함하지 않는다. FFmpeg 재배포 적합성·제품 라이선스는 판정하지 않았다.

수정 SHA `78ace87755f96f11543c8e1c03094129284fc4ef`의
[run 38692651462](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/38692651462)는 GUI/CLI/엔진/MCP 빌드와 FFmpeg 준비 통과 후 TCP 인증 거절 검사에서 WinError 10054로 실패했다.
Windows accepted socket의 nonblocking 상속을 고려해 연결별 blocking 모드와 제한시간을 명시하도록 보완한다. 이 실패를 자동 실행 성공으로 취급하지 않는다.
artifact `11952673670`은 실행 실패 진단용이다. 후속 검사는 동시 첫 기동·CLI/MCP 혼합 경쟁도 포함한다.
