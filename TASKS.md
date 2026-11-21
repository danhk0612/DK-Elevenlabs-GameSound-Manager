# 개발 Task

상태: PREP 완료 / T00-A 실기 미확인·사용자 지시로 개발 대기 해제 / T00-B 최소 실행 기반 구현·Windows 빌드 성공 / T01 로컬 관리 구현 / T02 연결·한도·생성 경로 구현, 라이브 미검증 / T03 기본 편집·분석 자동화 구현 / T04 비동기 제출·조회·취소 보강. 최신 지시 “실기, 점검 무시하고 진행해”는 이전 실기 대기와 T00-B 착수 금지를 해제한다. 검증되지 않은 결과를 통과로 바꾸지 않는다. D01~D06 채택은 유지하며 병합·Release/tag·공개 배포·실제 Key/크레딧·실제 게임/음원 변경은 이번 범위에 포함하지 않는다. 현재 T04 로컬 비동기 큐/조회·취소를 구현했다. 다음은 검토 대기 제출·후보 이력 조회와 클라이언트 호환 자동화다.

## 구현 우선순위 — AI 자동화와 최소 UI

2026-11-20 사용자 지시에 따라 각 기능은 공통 명령 → CLI·MCP 구조화 입출력/오류 → GUI 없는 회귀를 우선 구현한다. T01~T03에서 해당 기능의 CLI·MCP 연결을 함께 만들고 T04는 누락·호환·통합 검증을 마무리한다. GUI는 상태/실패·재생/비교·필수 설정·사람 승인에 필요한 최소 화면으로 둔다. 후속 사용자 지시에 따라 실기/점검 대기를 개발 선행 조건에서 해제하고 T00-B를 진행한다.

T00-A 안에서는 [읽기 전용 증거 검사 CLI](spikes/t00-a/tools/README.md)를 추가했다. Linux Python 3.14 회귀 12개 및 기존 실제 ZIP의 24개 manifest/33개 CI/별도 ACL 대조 통과. 사용자 두 JSON이 없어 exit 2/pending이다. 새 도구는 제품 엔진/GUI/collector와 분리되어 있고 Task 완료나 다음 Task 승인을 자동으로 만들지 않는다.

## PREP — 최초 분석·저장소 초기화

- 목적·범위: 입력 요구사항 분류, 아키텍처 비교·권고, 기준 문서, Task·다음 지시문, 최소 텍스트/Git 구조.
- 선행: 최신 저장소·기존 작업 확인.
- 완료: 기준 문서가 원격 저장소에 반영되고 이후 작업이 대화 없이 시작 가능.
- 금지: 미결정 기술 확정, 제품 전체 구현, 실제 음원 생성·게임 적용·Release/tag.
- 환경·상태: 고성능 Work, 이번 작업에서 완료. 원격 반영 확인은 완료 보고의 HEAD로 식별.

## T00-A — Windows 기술 검증 및 설계 결정 준비

- 목적: D01~D06에 필요한 실측을 확보하고 권고안을 사용자가 결정할 수 있게 만든다.
- 선행: PREP 원격 반영. Windows x64 실행 환경 또는 정확한 SHA의 사용자 실기 결과 수집 경로 확보.
- 범위: task/t00-a-technical-spike에서 격리된 최소 GUI·독립 CLI·stdio MCP 실험. 공통 명령 하나와 임시 DB/파일 작업을 재사용한다. 단일 엔진 소유 방식과 직접 라이브러리 접근 방식의 위험 비교, GUI 종료 후 작업 유지·중복 기동·클라이언트 동시 요청·작업 중단 후 재기동·한글/공백 경로·stdout 오염·자격증명 저장 후보·ffprobe/FFmpeg와 GUI 재생을 검사한다. 임시 검증 설정을 제품 기본값으로 삼지 않는다.
- 완료: 정확한 코드 SHA의 Windows 빌드·artifact와 실제 실행 증거, 도구 버전·명령·합격 기준·결과 기록. 미검증은 이유·재개 조건 명시. D01~D06의 비교/권고/사용자 선택 항목과 T00-B의 작은 구현 계약을 문서화. CI 성공과 실제 실행을 구분. 실기 미확보를 검증 성공으로 기록하지 않는다. 최신 지시로 실기/점검 대기 없이 후속 구현을 진행한다.
- 금지: 사용자 Key·유료 생성, 실제 게임/음원 수정, 모든 제품 기능 구현, 배포·라이선스 확정, 직접 병합·Release/tag. T00-B 착수 금지는 최신 사용자 지시로 해제했다.
- 환경: 고성능 Work. 구조 비교가 끝난 반복 검증은 범위가 정해진 별도 저성능 Work로 인계 가능.

- 2026-11-20 인수 확인: main `4c3537c55084b71feb5c42eb2340005bf922f197` 및 기존 작업 브랜치/draft PR #1을 재사용했다. 새 artifact `828e74540ae0aa0013d6ec3425389cb0631b0b1a`의 ZIP/manifest/보고서/PE/source 대조를 완료했다. 사용자 `user-automatic.json`·`user-observation.json`은 미수령이다. 이후 사용자가 기술 선택을 위임해 D01~D06을 채택했다. 사용자 직접 선택/실기 성공과 구분해 결정 원문을 보존했다. [정확한 실기 ZIP과 명령](docs/T00_A_WINDOWS_CHECK.md)을 제공했으며 당시 T00-A 진행 중·T00-B 미착수를 유지했다. 이후 실기/점검 대기 해제로 T00-B를 진행했다.

## T00-B — 확정 스택의 최소 실행 기반

- 목적: 이후 구현에 사용할 공통 작업 계층과 빌드 기반을 만든다.
- 선행: D01~D06 위임 채택과 관련 계약 반영. T00-A 실기는 사용자 지시로 개발 차단 조건에서 해제했다.
- 범위: 확정된 GUI·엔진·CLI·MCP 최소 진입점과 버전/상태 명령, 공통 명령 계약, 최소 프로세스 수명, 의존성·lockfile, Windows CI·artifact. 실제 기능은 placeholder와 명확히 구분한다.
- 완료: GUI/CLI/MCP에서 같은 상태 응답, GUI 없이 실행, Windows 시작·종료·연결 검사와 정확한 SHA의 CI·artifact. README 실행 방법, 실제 상태·다음 Task 갱신.
- 금지: 자산 관리 전체 구현, 원격 생성, 게임 적용, 미결정 기본값 도입, 직접 병합·배포.
- 구현 상태: app/에 Windows SID별 singleton mutex·owner-only/local-only named pipe, 공통 version/status/shutdown·자동 기동·5분 idle, CLI JSON·stdio MCP·최소 GUI를 구현했다. 작업 큐/사운드 기능은 없다. Windows 실제 실행은 사용자 지시로 대기하지 않으며 빌드/자동 검사 결과와 구분한다.
- 환경: 확정된 단일 Task 범위 구현. 실기 미수령을 이유로 추가 확인을 반복 요청하지 않는다.

## T01 — 프로젝트·로컬 사운드 관리

- 목적: 외부 서비스 없이 사용할 수 있는 게임별 라이브러리를 만든다.
- 선행: T00-B, D04 저장·가져오기 정책 확정.
- 범위: 프로젝트 생성·선택·목록, BGM/SFX/Voices 등록·검색·재생, 원본 복사/보존·해시·출처, 자산/후보/버전의 최소 스키마·마이그레이션, 필요한 사운드 대응, 캐릭터·대사 로컬 관리, 공통 명령과 GUI. 파형 분석은 T03과 연결할 최소 정보만 둔다.
- 완료: 두 프로젝트 격리, 같은 이름·경로 충돌, 가져오기 중단·재시작, 원본 해시 보존, 오프라인 목록·검색·재생, 설치 폴더와 데이터 분리. 문서와 회귀 검사.
- 금지: API 생성·Voice 원격 변경, 게임 경로 쓰기, 사용자 원본 이동/삭제, 전문 편집.
- 환경: 저성능 Work. 스키마의 다기능 변경은 고성능 Work 검토.

T01 구현 상태: v0.2.0 공통 명령 18개/CLI JSON/MCP 17개 도구, 프로젝트별 SQLite·원본/후보 버전·출처/hash/복사 journal·중단 대조, 필요 사운드 연결·캐릭터/대사·페이지/검색, 최소 목록/재생 GUI를 추가했다. [상세 계약](docs/T01_LOCAL_LIBRARY.md)을 따른다. 파형·코덱/음질 검사·원격 생성·승인/게임 적용은 후속 단계다. 실제 Windows 화면/청감 대기를 요구하지 않으며 실제 빌드/미확인은 CURRENT_STATE에 구분한다.

## T02 — ElevenLabs 핵심 연결·제한된 생성

- 목적: 계정 Voice를 활용해 후보를 생성하고 사용량을 통제한다.
- 선행: T01, D05 인증정보 저장. D09 수치는 미설정 차단으로 유지하며 실제 활성화 시 GUI의 명시적인 한도를 요구한다. 라이브 실험은 사용자가 제공한 Key와 허용된 비용 범위 필요.
- 범위: 공식 API 재확인, Key 연결/해제·계정/사용량·Voice 목록/상세/설정·미리듣기, 캐릭터 Voice 할당·대사 언어/설정, TTS/SFX 생성·후보·작업 이력, 공통 큐·계정 예산 예약·횟수 제한·취소·불확실 응답 처리. 저장 Voice와 Design 미리보기 구분.
- 완료: 모의 응답 회귀와 허용된 실제 계정 검증 분리, 무료 계정 가능/거절 기능 기록, Key 비노출, GUI/다른 클라이언트 동시 요청 한도 준수, 응답 유실 시 무조건 재생성하지 않음, 원본 보존·오프라인 기능 유지. Key 미확보면 라이브 검증을 미완료로 남김.
- 금지: 무제한 생성, 계정 요금제·초과 과금 설정 변경, Voice 원격 삭제·Design/Remix/Music 본 구현, 게임 적용.
- 환경: 저성능 Work. 예산·과금·복구 계약 변경 판단은 고성능 Work.

T02 구현 상태: v0.3.0에서 Windows Credential Manager/WinHTTP adapter, 계정·Voice/모델·사용량 조회, workspace 공유 예산, 단일 POST 큐·중단 상태·취소·수신 파일 등록 복구, TTS/SFX 후보, 공통 명령 32개/MCP 31개 도구와 최소 연결 GUI를 구현했다. [상세 계약](docs/T02_GENERATION.md). mock 회귀 19개 통과. 실제 Key/허용 비용이 없어 라이브 권한/과금·Credential Manager 수명은 미확인이다. 원격 preview는 URL 조회까지만 제공한다. T02 전체 완료로 닫지 않으며 실기 대기로 후속 개발을 막지 않는다.

## T03 — 기본 편집·버전·기술 검증

- 목적: 후보 비교와 게임용 파일 준비를 지원한다.
- 선행: T01, T00의 FFmpeg 근거. D08 기본값은 미결정으로 유지하고 작업마다 출력·loop·검사 조건을 명시한다. T02와의 연결은 통합 시 확인.
- 범위: 자르기·음량·페이드·변환, 파형·구간·루프 정보, 새 버전과 편집 계보, 형식·길이·볼륨·클리핑·무음 분석, 후보 재생·비교와 주관 검토 분리.
- 완료: 알려진 신호 fixture로 검사 수치·변환을 확인하고 한글 경로·잘못된 파일·중단·출력 충돌 검사. 기존 원본/승인본 해시 보존, 조건·도구 버전·결과 연결. 실음원 청감 확인을 수치 검사와 구분.
- 금지: 전문 DAW·효과 추가, 원본/승인 버전 덮어쓰기, 임의 음질·음량 기본값, 게임 적용.
- 환경: 저성능 Work.

T03 구현 상태: v0.4.0에서 7개 audio 명령을 추가해 총 39개/CLI·MCP 38개 도구다. 명시적 FFmpeg 경로/hash/version, trim/gain/fade·WAV/OGG/MP3·rate/channel, 첫 스트림의 peak/RMS/quiet/파형/loop 수치, operation 이력·입력 조건·새 후보·등록 복구, 최소 저장 검사 조회/비교 재생을 구현했다. [상세 계약](docs/T03_AUDIO.md). 합성 신호·실제 FFmpeg 통합 4개 포함 26개 통과. D08 게임별 기본값, loop tag·LUFS/true peak, async 로컬 큐/취소·주관 검토·사용자 Windows 청감은 후속이다. T03 전체 수용 완료로 과장하지 않으며 실기 대기 없이 개발한다.

## T04 — 외부 AI CLI·MCP 연결 완성·호환 검증

- 목적: T01~T03에서 함께 구현한 CLI·MCP 연결을 완성하고 실제 AI 클라이언트와 통합 검증한다.
- 선행: T00-B·T01·T02·T03의 공통 명령 안정화.
- 범위: 조회·사운드 필요 목록·Voice/대사·생성·편집·검증·작업/후보 이력·검토 제출을 CLI와 stdio MCP에 연결. 명령 도움말·구조화 스키마·버전/오류·클라이언트 설정 예시. 게임 분석은 외부 AI의 책임으로 유지.
- 완료: 로컬 CLI 1개와 지원되는 MCP 클라이언트 1개에서 end-to-end 검사, GUI 종료 상태의 작업과 GUI 재연결 일치, stdout 프로토콜 준수, 프로젝트 격리·한도·비밀정보·승인 거절 공통 회귀. 확인된 제품/환경과 미확인 환경을 표로 구분.
- 금지: 모든 ChatGPT/GrokBot 연결 보장, 원격 공개 MCP 서버, AI가 스스로 승인하는 도구, 게임 파일 직접 쓰기.
- 환경: 저성능 Work. 클라이언트 보안·프로세스 계약 변경은 고성능 Work.

T04 보강 상태: v0.5.0에서 audio.submit/cancel, 실행 중 별도 get/list, 도구/입력 snapshot·32개 큐·재시작 무재생·진행 단계·협조 취소/자식 종료·최종 등록 경계, 원격 생성과 원자적 대상 예약·큐 교대 실행을 추가했다. 총 41개 명령/MCP 40개 도구이며 동기 명령은 유지한다. [계약](docs/T04_ASYNC_AUDIO.md). 실제 FFmpeg 6개 포함 35개 회귀 통과. 실제 AI 클라이언트 연결·Windows IPC/실행 중 취소 실측은 미완료이며 T04 전체를 완료로 닫지 않는다.

v0.6.0 보강: review.submit/get/list/withdraw/check와 asset.history, 불변 후보/검사 근거·철회·현재 일치 확인·최소 검토 UI를 구현했다. [계약](docs/T04_REVIEW.md). 총 47개 명령/MCP 46개 도구, 실제 FFmpeg 7개 포함 39개 회귀 통과. 승인 도구/게임 적용은 없으며 T04 실제 클라이언트 수용은 미완료다.

v0.7.0 보강: CLI offline validate와 선택형 Python JSON 작업 정의 클라이언트(CLI/MCP), 입력 preflight·async polling·hash/검사 근거 연계·중단 ID 반환을 구현했다. [계약](docs/T04_AUTOMATION.md). 클라이언트 회귀 9개·native offline preflight/MCP를 Linux에서 확인했고, Windows 실제 engine CLI/MCP 무GUI 작업 정의 재호출/재연결 CI를 추가했고 `3f47e5852942dd28d951354484ecd8d37107e582`의 [Windows run 44751508856](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/44751508856)에서 전체 성공했다. 자동 기동·같은 검토 기록·원본 보존/변경 거절과 inherited-pipe 정리 회귀를 포함한다. 실제 AI 제품 클라이언트 호환은 별도다. 공통 47개/MCP 46개·schema 5·UI는 유지한다.

v0.8.0 보강: 선택형 Python batch manifest(최대 32개), 전체 preflight·source/후보/전역 작업/검토 ID 충돌·독립 순차 실행·항목별 JSON/JSONL·선택 재호출·확정 오류 keep-going과 불확실 결과 중단을 구현했다. [계약](docs/T04_BATCH.md). Python 18개 회귀와 native offline batch 검사를 확인하고 실제 Windows engine 다건 CI를 추가했다. UI/DB schema/공통 47개·MCP 46개는 유지한다. 코드 `6af11ecf00b59554a4ef98b725e5e48bd3a528a2`의 [Windows run 44830379629](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/44830379629)에서 Python 18개·실제 engine batch의 선택/shared source/CLI·MCP/JSONL/중단·keep-going/원본 보존·개발 패키징까지 전체 성공했다. 사용자 Windows 11/실제 AI 제품 연결은 별도다.

## T05 — 사람 승인·게임 적용·복구

- 목적: 검토한 후보를 명시적 사람 승인 범위에 한해 적용한다.
- 선행: T01~T04, D07 설계 확정. 고성능 Work에서 승인 출처·불변 계획·대상 검증·중단 복구 계약을 먼저 검토하고 사용자의 미결정 선택을 확인한다.
- 범위: 매핑·적용 미리보기, 사람 승인, 계획과 버전 해시/대상 일치 검사, 기존 파일 보존·적용 저널·복구. 게임 코드 자동 변경은 별도 사용자 범위 결정 없이는 포함하지 않는다.
- 완료: 승인 없는 GUI/CLI/MCP 실행 거절, 승인 후 후보/대상 변경 거절, 경로 이탈·중복 적용·각 단계 강제 종료·재시작 복구·외부 파일 변경 충돌 검사. 테스트용 게임 경로에서 이전 해시 복구 확인. 사용자 실제 게임 적용은 별도 명시 승인.
- 금지: AI 자기 승인, 검증 통과를 사람 승인으로 취급, 원본/승인본 임의 변경, 실제 사용자 자료 무단 쓰기.
- 환경: 설계·통합 고성능 Work, 확정된 구현·회귀 저성능 Work.

## T06 — 범용성·Windows 배포 준비

- 목적: 지원 범위를 실측하고 배포 가능한 구성을 정리한다.
- 선행: T01~T05, D02·D03·D11 사용자 결정. 사용할 FFmpeg 바이너리·의존성 고지/소스 조건 확인.
- 범위: 다른 두 게임의 매핑/출력 검증, 실제 AI 클라이언트 호환표, Windows 설치/갱신/제거·데이터 보존·오프라인·동시 작업 통합 검사, 패키징·사용자 README·라이선스/고지 준비.
- 완료: 정확한 SHA의 빌드·artifact·Windows 실기와 제한 표, 계정 없이 로컬 관리, 갱신/제거 뒤 데이터 보존, 의존성·배포 조건 확인, 남은 차단 명시.
- 금지: 승인 없는 공개 배포·Release/tag, 사용자 데이터 제거, 후속 Music/Voice 기능 혼입.
- 환경: 고성능 Work 통합 판단, 반복 패키징·검증 저성능 Work. 배포 실행은 별도 사용자 지시.

## 다음 Codex에 입력할 지시문

[새 Codex 인수인계](docs/NEXT_CODEX.md)의 지시문을 사용한다.
현재 T04 로컬 작업의 비동기 큐/조회·취소를 구현했다. 검토 제출·후보 이력은 v0.6에서 구현했다. 작업 정의 클라이언트·CI 무GUI 전송은 v0.7에서 추가했다. Windows 실제 엔진 CI도 통과했다. 다건 작업 정의는 v0.8에서 구현했다. 다건 Windows CI도 통과했다. 다음은 지원 AI 제품 클라이언트 호환·필요한 사운드/작업 이력 자동화 보강이다. 실제 Key/허용 비용이 없으면 모의 응답·오프라인 경로를 개발하고 라이브 생성은 하지 않는다. 실기·청감 결과를 추가로 요구하며 대기하지 않는다. 결과가 없으면 미확인으로 기록한다. 정확한 구현/빌드 상태는 CURRENT_STATE.md를 따른다.

### T04 v0.9 Linux headless 실행 경로

사용자가 현재 AI 환경에서 사용할 수 있게 진행하도록 지시해 Linux native engine/CLI/MCP를 연결했다. [계약](docs/T04_LINUX_HEADLESS.md). private root/UID·flock/socket/동시 기동·stale 복구와 공통 worker/queue를 사용한다. Linux에서 실제 단일·다건 CLI/MCP/합성 mock 한도·강제 종료 재연결, 실제 FFmpeg async 편집·검사·후보 검토를 확인했다. 전체 수용/실제 ElevenLabs 제작 완료는 아니다. 다음은 Linux 실제 provider를 위한 secret 전달/수명·명시 예산·HTTP/과금 불확실성 계약과 오프라인 회귀를 구현하는 것이다. 실제 Key/비용 범위 없이 라이브 생성하지 않는다. Linux GUI/T05/게임 변경은 범위 밖이다. 정확한 SHA/CI는 CURRENT_STATE.md를 따른다.

v0.9 로컬 검증 source SHA: `e2e92a6c67864ceec6ce5f5a30b769a6ebd6bd9d`. [Windows/Linux CI 44833857881](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/44833857881) Windows·Linux 전체 성공. 로컬 Rust 39개·Python 18개·native Linux/FFmpeg와 Windows 타입 검사/UI build는 통과했다.

후속 client signal group 분리 source `32828ea59e73d8b12c525acc7210f73ffbab3114`, 로컬 실제 Linux/FFmpeg 재검사 성공. [CI 44834877114](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/44834877114) 결과 확인 중.

### v0.10 T02/T04 Linux 인증 세션·실제 전송 코드

사용자의 후속 진행 지시에 따라 명시적인 account/key/TTL/한도를 pipe stdin으로 engine에 전달하는 메모리 세션, 만료·재시작 차단, 선택형 숨김 입력/private FD 런처와 fixed-origin reqwest/rustls HTTPS를 구현했다. 일반 CLI/MCP key/예산 변경 명령과 GUI는 늘리지 않는다. [계약](docs/T02_LINUX_PROVIDER.md). 로컬 합성 인증서/가짜 키의 TLS transport와 실제 engine metadata-only session smoke를 통과했고 Rust 43개(실제 FFmpeg 7개 포함)·Python 18개가 통과했다. Windows x64 타입 검사/UI build 및 최신 SHA CI는 CURRENT_STATE.md를 따른다. 실제 키/허용 비용 없이 라이브 검증하지 않으며 T04 전체·T05·게임 적용은 완료/착수하지 않는다. 이후 실제 Key·quota/허용 생성 범위가 안전한 세션으로 준비되면 제한된 실제 생성 검증이 필요하다. 준비 전에는 기존 자동화 경로와 재사용할 검증/사용 안내를 보강한다.

v0.10 구현 source `3a5c52ca1a525785a6cc96ffd4e761a04a7cbfff`, 초과 key pipe 입력 거절 보강 source `54d2fbf2fa5809b251857aac23527743be7afff8`. [최신 Windows/Linux run 44839231602](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/44839231602) Windows·Linux 전체 성공. Linux Rust 43개·Python 18개/실제 세션·TLS/FFmpeg·Windows native/GUI/CLI/MCP·패키징을 확인했다. 실제 키/크레딧 요청은 없다.

## Windows/NAS 파일 입력 확장

사용자는 Windows에서 NAS 파일을 편집하고 Codex는 Docker에서 실행하므로 명시적인 설정 파일 입력을 요청했다. [계약/절차](docs/WINDOWS_NAS_SETUP.md). linux_session.py --config의 strict bounded JSON·UTF-8 BOM/CRLF·NAS mode 표시 허용과 Docker-local runtime을 추가한다. 실제 elevenlabs.local.json/백업은 Git 제외하고 예제는 key 빈 값·생성 false/한도 0으로 둔다. 기존 입력/엔진 정책·공통 명령/schema/GUI는 유지한다. 실제 키 파일을 출력/수정하지 않으며 입력 경로의 원본을 유지한다.

NAS file config source `28a2a654a23a8efe3e71b47c64fbaeed7a1e9187`, 로컬 Linux Python 22개+실제 session smoke 통과, [CI 44841185626](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/44841185626) 진행 중. Windows는 설정 파일 편집만 하고 Codex는 Docker 경로의 --config로 연결한다.

## T02 실제 Linux SFX 생성 검증 (2026-11-21)

사용자의 실제 key 설정 및 진행 지시에 따라 [실기 결과](docs/T02_LIVE_RESULTS.md)의 SFX POST 1회·header cost 5·candidate/실제 decode·pending review·재시작 동일 ID 무중복을 확인했다. 실행 환경의 새 기본 state 부모 권한 실패에만 private temp를 선택하고 기존/명시 ledger 경로는 바꾸지 않는 보완, Linux transport 표시 수정을 적용했다. 사용자 설정/음원은 Git 제외이고 TTS/Windows 인증/청감/최종 과금·T04 전체/T05는 미확인/별도다.
