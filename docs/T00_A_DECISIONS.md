# T00-A 결정 자료 및 T00-B 조건부 계약

**D01~D06은 2026-11-20 사용자의 기술 판단 위임에 따라 아래와 같이 채택했다. 사용자 Windows 화면/청감은 미수령이다. 후속 사용자 지시로 실기/점검 대기를 해제하고 T00-B를 진행한다.**
실측은 [결과](T00_A_RESULTS.md), Windows 수집은 [절차](T00_A_WINDOWS_CHECK.md)를 따른다.

## 2026-11-20 최신 사용자 지시 — 이전 선행 금지 해제

사용자 원문: **“실기, 점검 무시하고 진행해”**. 이전 실기/점검 게이트와 T00-B 착수 금지를 해제하고 자동화 우선 구현을 진행한다. 아래 과거 문단/조건부 지시문의 실기 대기·T00-B 금지는 역사적 기록이며 현재 실행 지시가 아니다. D01~D06은 유지한다. 사용자 실기 성공을 선언하지 않고 미확인으로 남긴다. 병합·Release/tag·공개 배포·실제 Key/유료 생성·실제 게임 적용은 이번 지시에 포함시키지 않는다. 현재 명령/수명 구현은 [제품 기반](../app/README.md)을 따른다.

## 비교 자료 — 채택 전 검토 근거

| ID | 근거·비교 | 현재 권고 | 사용자 결정 필요 / 남은 증거 |
| --- | --- | --- | --- |
| D01 기술 | Rust 공통 request 함수를 GUI invoke·독립 CLI·stdio MCP에서 재사용. CLI/MCP/SQLite/FFmpeg Linux 실제 프로세스 검사 통과. GUI UI TypeScript 빌드 통과. Windows GUI/독립 진입점 빌드·자동 실행도 통과(정확한 SHA는 결과 문서). 정확한 c778048 SHA에서 실제 WebView2 버튼 명령·창 종료/재연결·WAV ended까지 자동 실행 통과. 사람의 화면/청감은 미확인. Electron/.NET은 실행 비교하지 않았음 | Tauri 2+React/TS+Rust+SQLite를 우선 후보로 유지. 런타임 크기·개발 비용 우열은 미실측 | Windows 사용자 화면·청감 후 이 후보 채택 또는 비교 실험 추가. 실패 원인 없이 스택 교체 금지 |
| D02 지원 OS | CI는 Windows Server 2022 x64, 사용자 Windows 데스크톱과 다름. Linux 검사는 Windows 근거 대체 불가 | 우선 Windows x64로 실기, ARM/32비트 약속하지 않음 | 지원할 Windows 버전과 아키텍처. Windows 10/11, WebView2 미설치, 비관리자 환경 검사 필요 |
| D03 배포 | artifact는 설치 없는 실험 EXE+검사 스크립트. FFmpeg와 WebView2는 외부 준비, 설치/제거/갱신 실험 없음 | 실행 검증에는 압축 artifact. 정식 포터블·설치형 채택 보류 | 최초 제품 제공 형식과 런타임 동봉 여부. 서명·업데이트·고지·배포 허용은 T06 |
| D04 저장 | 필수 환경변수로 임시 작업공간 분리, 한글/공백 경로 DB·파일·FFmpeg. 실제 원본 대신 합성 신호 hash 보존. 네트워크/동기화 폴더 미검증 | 앱 설정과 사용자 작업공간 분리, 가져오기는 복사 권고 유지 | 설정 위치·프로젝트 폴더 선택 방식·복사 정책. 기본값 미확정; 프로그램 폴더 저장 금지 권고 |
| D05 자격증명 | Windows Dummy Credential Manager 쓰기/읽기/삭제와 DPAPI CurrentUser 메모리 왕복 스크립트. API Key 사용 없음. Windows CI에서 두 후보의 dummy 왕복과 Credential 삭제 통과; 원시 보고서에 기록. 다른 사용자·재부팅·프로필 이동·내보내기·로그 경계 미검증 | Credential Manager 우선 검토: 직접 암호문 파일·ACL 관리 부담 감소. DPAPI는 앱이 암호문 파일 수명·ACL·백업을 책임져야 함 | 어느 후보 채택할지. 양쪽 모두 같은 사용자 프로세스에 대한 보안 격리 보장 아님. dummy 왕복만으로 R04 충족 선언 금지 |
| D06 프로세스 | 단일 엔진이 DB/파일 writer. 클라이언트 종료 후 지속, 중복 기동 거절, 동시 id/모의 예산, 강제/정상 종료 복구 검증. 직접 접근은 Python SQLite writer 잠금 fixture만 비교; 완성 대안 구현 아님 | 단일 로컬 엔진 소유 방식 우선 유지. GUI 닫기≠취소; 중단 작업 자동 재시도 금지 | 수동/자동 기동, busy 종료, idle 수명·시간값, IPC 방식과 사용자 범위. 이 실험 TCP/token을 제품 IPC로 채택하지 않음. opt-in 자동 기동 경쟁·busy 거절·idle 수명·두 작업공간 독립성은 검증 통과. 별도 타 계정 pipe ACL은 최신 SHA에서 통과. 엔진 IPC 통합·원격 차단·사용자 전체 소유권·파일/DB 대조 복구 계약은 남은 게이트 |

## 2026-11-20 위임 결정 — 현재 기준

사용자 원문: **“네가 직접 적절하게 결정해서 작업해”**. 이는 D01~D06 선택을 위임한 지시로 반영했다. 사용자가 아래 세부 항목을 직접 선택했다거나 Windows 실기까지 통과했다는 뜻은 아니다. 최초 지시의 실기 게이트·T00-B/병합 금지는 유지한다. [구조화 결정 기록](t00-a-evidence/refresh-20261120/delegated-decisions.json)을 따른다. 위 표와 아래 과거 문단의 '미결정/권고'는 당시 비교 기록이며 현재 채택 상태는 이 절이다.

| ID | 채택한 선택 | 이유와 남은 확인 |
| --- | --- | --- |
| D01 | Tauri 2·React/TS GUI, Rust 엔진/공통 명령, SQLite, FFmpeg/ffprobe, stdio MCP | 33개 Windows 자동 실행 증거를 활용한다. Electron/.NET 우열을 실측했다고 주장하지 않는다. MCP의 검증 baseline은 2025-11-25이며 제품 클라이언트 호환은 T04 |
| D02 | 첫 제품/검증 대상 **Windows 11 x64** | Windows 10·ARM64·x86 지원은 초기 범위에서 약속하지 않는다. Server 2022 CI는 Windows 11 사용자 실기가 아니므로 비관리자 화면/청감과 OS 빌드 확인이 남는다 |
| D03 | 초기 제품 제공 방식 **포터블 ZIP**. WebView2 별도 준비, FFmpeg/ffprobe는 사용자 지정 경로 또는 PATH | 설치/업데이트/자료 보존·외부 구성요소 고지 검사는 T06. FFmpeg 자동 설치·동봉·재배포 적합 판정 없음. 현재 실험 ZIP은 제품 배포가 아님 |
| D04 | 앱 설정/엔진 발견은 `%LOCALAPPDATA%\DK-Elevenlabs-GameSound-Manager`, 작업공간은 사용자가 명시적으로 고른 로컬 디렉터리. 원본은 관리 저장소로 **복사** | 프로그램 폴더에 자료 저장 금지, 원본/승인본 불변. T01에서 schema/import/대조 복구와 실제 경로 검증. 네트워크/동기화 DB는 초기 지원하지 않음 |
| D05 | Windows **Credential Manager** 현재 사용자 범위, 제품 연결은 T02 | DPAPI 자동 fallback은 넣지 않는다. 인증정보 실패 시 오프라인 관리 유지·원격 생성 차단. 실제 Key·로그/내보내기·재부팅 등은 T02에서 별도 검사 |
| D06 | **사용자 SID별 엔진 1개 + 로컬 named pipe**. 최초 클라이언트에서 자동 기동. GUI 닫아도 작업 유지. busy 정상 종료 거절. 클라이언트 0·작업 0이 **5분 연속**이면 idle 종료 | owner SID DACL·원격 거절·사용자 범위 발견/잠금으로 설계한다. 전용 pipe DACL 실험은 제품 통합/원격 차단 성공 아님. 5분은 이번 위임 설계값이며 3초/30초 fixture나 측정 최적값에서 가져온 것이 아님 |

D06 세부 계약: 프로젝트마다 엔진/예산을 따로 만들지 않는다. GUI·CLI·MCP 연결은 같은 사용자 엔진을 사용한다. 클라이언트 재연결 시 idle 타이머를 리셋하고 queued/running이 있으면 idle 종료하지 않는다. 정상 종료는 busy에서 거절하며 T00-B에 강제 종료 API를 추가하지 않는다. Windows 서비스/로그인 자동 실행은 요구하지 않는다. interrupted/outcome_unknown은 자동 재실행하지 않는다. SID별 기동 경쟁·여러 작업공간·실제 IPC·원격 차단·수명 검증은 T00-B의 최소 기반 범위에서 확인한다. 계정 전체 예산은 T02, 파일/DB 대조 복구와 실제 게임 적용은 각 후속 Task 경계를 유지한다.

**T00-A 미완료 이유는 사용자 Windows 화면/청감 실기 미수령이다.** user-automatic.json / user-observation.json의 정확한 SHA 대조가 아직 불가능하다. 선택 위임을 실기 성공으로 대체하지 않는다. 실기가 준비되면 현재 채택 계약과 완료 조건을 대조하고 조건부 T00-B 지시문만 제시한다. 이번 작업에서 T00-B 구현은 하지 않는다.

## 구조·수명 비교

| 방식 | 실험·관찰 | 추가 책임 |
| --- | --- | --- |
| 각 클라이언트 직접 공통 라이브러리/DB 접근 | SQLite 두 writer는 동시에 쓰지 못하는 fixture를 재현. GUI/CLI/MCP 각각의 실제 작업 소유 구현은 하지 않았음 | 진입점마다 예산/중복/파일 소유권을 공유하고 클라이언트 종료에도 작업을 유지해야 함 |
| 단일 엔진 + 어댑터 | 같은 DB에 쓰는 엔진 1개, 모든 요청 직렬 수락. 작은 DB 트랜잭션 후 모의 작업 수행. 읽기/쓰기 부하 성능은 측정하지 않았음 | IPC 인증·엔진 주소 발견·기동 경쟁·수명 정책 필요. 단일 루프라 느린 클라이언트가 잠시 요청 처리를 지연시킬 수 있음 |

임시 모의 한도 10·요청 비용 1/3·최대 지연 30초는 검사 값이며 D09의 제품 정책이 아니다. endpoint token은 엔진 기동마다 교체한다. Windows token 파일 ACL과 타 사용자 접속 방지는 검증하지 않았으므로 실험용 dummy 자료만 허용한다. 엔진 단일성은 작업공간 lock 기준이며 OS 사용자 전체 단일성이 아니다. opt-in 자동 기동·idle 종료 후보는 Windows에서 검증했다. fixture 시간값은 제품 기본값이 아니며 두 작업공간은 독립 엔진이다.

복구 fault injection은 임시 파일 기록 직후와 최종 파일 rename 직후에 프로세스를 종료한다. 재기동 시 작업은 interrupted, 예산은 보존되고 파일은 남는다. **파일·DB 자동 대조와 원자성은 미구현이며 한계 발견을 복구 완성으로 해석하지 않는다.** 네트워크 과금/응답 유실은 시뮬레이션하지 않았으며 T02 outcome_unknown 계약이 필요하다.

## T00-B 계약 초안 — 선행 충족 전 실행 금지

- 선행: T00-A의 정확한 수정 SHA Windows 빌드·artifact, GUI 공통 명령·닫기/재연결·합성 재생의 사용자 실기. D01~D06 위임 결정 및 기준 문서 반영과 제품 IPC/수명 검증 계획 확인. 제품 IPC 통합 자체의 검증은 아래 T00-B 완료 범위다. T00-A를 완료로 닫을 근거 검토.
- 범위: 채택한 스택만 사용해 제품용 최소 공통 명령 `version/status`, 엔진·GUI·독립 CLI·stdio MCP 진입점, 결정된 IPC/시작·종료·연결 수명, lockfile·Windows CI·artifact·실행 안내. 실험의 job/예산/파일 코드를 제품 기능으로 복사하지 않는다.
- 명령 계약: 구조화 request/response, 동일한 검증·오류 코드·프로토콜 버전. stdout은 CLI JSON 또는 MCP 프로토콜만, 진단은 stderr. API Key·DB 내부 정보는 GUI와 출력에 전달하지 않는다.
- 완료: GUI/CLI/MCP status가 동일 엔진·버전을 가리킴; GUI 없이 시작·조회 가능; 동시 첫 기동 시 소유자 1개; GUI 닫기 후 CLI/MCP 사용·GUI 재연결; 결정된 busy/idle/명시 종료·강제 종료 복구 정책; 한글 경로; stdout; 정확한 코드 SHA Windows CI/artifact와 실제 실행 결과 분리 기록.
- 제외: 프로젝트·사운드 CRUD, 원격 생성, 실게임 변경, 사람 승인·게임 적용, 제품 라이선스/공개 배포. 다음 기능은 T01 이후.
- 환경: 결정 검토·남은 구조 검증은 고성능 Work/Chat, 결정 후 위 최소 구현은 저성능 Work. 구조 변경 필요 시 반환.

조건부 다음 지시문:

```text
D01~D06 사용자 결정이 저장소에 반영되고 T00-A가 실측 근거로 완료됐는지 먼저 확인해.
충족하지 않으면 T00-B에 착수하지 말고 CURRENT_STATE.md의 남은 T00-A만 확인해.
충족한 경우 최신 main·브랜치·PR, AGENTS.md와 루트 기준 문서 및
 docs/T00_A_DECISIONS.md를 읽고 T00-B의 최소 version/status 기반만 구현해.
채택 스택·IPC·수명·경로·자격증명 결정을 임의로 변경하지 마.
정확한 수정 SHA의 Windows CI/artifact와 GUI·CLI·MCP 실제 실행을 분리 기록해.
API Key·크레딧, 사용자 게임·음원 변경, 직접 병합·Release/tag·T01 착수는 금지야.
브랜치 push·PR·해당 검증 CI는 허용해.
```

## 공식 문서 확인 (2026-10-15)

아래 페이지를 다시 열어 접근 가능함을 확인했다. 공식 문서 확인은 실측 대체가 아니다.

- [Tauri prerequisites](https://tauri.app/start/prerequisites/), [Rust command 호출](https://v2.tauri.app/develop/calling-rust/): Windows 도구 준비와 GUI 명령 연결 근거.
- [MCP stdio 2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25/basic/transports): JSON-RPC stdio, stdout 프로토콜 분리 근거. 스파이크 협상 버전은 2025-11-25에 한정; 기존 API_RESEARCH.md의 다른 버전 지원을 주장하지 않음.
- [CredWriteW](https://learn.microsoft.com/en-us/windows/win32/api/wincred/nf-wincred-credwritew), [CryptProtectData](https://learn.microsoft.com/en-us/windows/win32/api/dpapi/nf-dpapi-cryptprotectdata): Credential Manager/DPAPI 비교 근거. 사용자 계정 실제 Key 없음.

- [Winsock accept](https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-accept): accepted socket의 listener 속성 상속 근거. 2026-10-15 재열람. Windows 스파이크에서 연결별 blocking 모드를 명시해 검증한다.

## 후속 격리 검증 결과 — 당시 비교 기록

코드 c778048의 Windows 33개 자동 검사와 원시 보고서는 결과 문서를 따른다. 수명 후보는 환경변수로 opt-in 한다. idle 수치는 fixture 값이며 제품 정책이 아니다.
동시 CLI/MCP 자동 기동, busy 종료 거절, 작업 중 idle 유지, 완료 후 idle 종료,
stale endpoint 재기동·자동 재시도 금지와 두 작업공간 독립성을 검사한다.
당시 사용자 전체 단일 엔진 선택은 미결정이었다. 현재는 위임 결정 절의 SID별 단일 엔진을 채택했고 제품 구현은 하지 않았다.

GUI는 실제 Edge WebDriver로 버튼을 클릭하고 native WM_CLOSE로 닫은 후
같은 엔진의 작업 결과를 새 GUI에서 확인하며 합성 WAV의 ended 이벤트를 검사한다.
사용자 데스크톱·스피커 청감은 별도 결과다. named pipe ACL은 native 별도 후보이며
초기 익명 허용 대조군 실패는 당시 판정 불가로 보존한다. 후속 `f8aa463`에서 실제 다른 표준 로컬 계정 토큰으로 허용 대조군 연결과 보호 pipe 거절·계정 삭제를 확인했다. 이 별도 객체 검사를 엔진 IPC 통합·원격 컴퓨터 차단의 증거로 확대하지 않는다.

추가 공식 근거(2026-10-15 다시 열어 접근 확인):
[Tauri WebDriver](https://v2.tauri.app/develop/tests/webdriver/),
[Microsoft WebView2 WebDriver](https://learn.microsoft.com/en-us/microsoft-edge/webview2/how-to/webdriver),
[Named pipe security](https://learn.microsoft.com/en-us/windows/win32/ipc/named-pipe-security-and-access-rights).

## 추가 구조 검토 — 사용자 전체 소유권과 파일/DB 경계

2026-10-16 검토 당시의 계약 제안이다. 사용자 범위 엔진·수명은 후속 D06 위임 결정으로 채택했으며, 계정 예산 원장·파일/DB 복구의 상세 계약과 기능은 각 후속 Task에서 확정한다. 제품 코드나 새로운 복구 기능을 구현한 결과가 아니다.

### 엔진과 계정 예산의 소유 범위

현재 `owner.lock`과 예산 합계는 `T00_SPIKE_ROOT` 하나에만 적용된다. 다른 작업공간을 열면 별도 엔진과 별도 예산이 생긴다. 따라서 지금의 동시 요청 검사는 동일 ElevenLabs 계정의 프로젝트 간 합산 한도를 증명하지 않는다.

권고는 OS 사용자별 엔진 하나가 프로젝트별 저장소와 계정별 예산 원장을 관리하는 방식이다. 모든 GUI·CLI·MCP 진입점이 같은 엔진을 발견하고, 프로젝트 경로와 별개인 사용자 범위에서 기동 잠금을 획득해야 한다. 프로젝트 DB writer와 계정 예산 writer의 책임을 분리해서 문서화하되 진입점별 독립 예산은 만들지 않는다.

| 경계 | 필요한 계약 | 검증 시나리오 / 구현 시점 |
| --- | --- | --- |
| 사용자 범위 엔진 | 사용자 SID에 묶인 발견·잠금·IPC, 같은 사용자의 여러 작업공간은 같은 엔진 참조 | 두 작업공간 동시 첫 기동에서 엔진 1개. 사용자 결정 후 T00-B 최소 status 검사 |
| 프로젝트 DB | 모든 명령의 프로젝트 ID 검증, 등록된 작업공간만 접근 | 두 프로젝트 간 작업·경로 누출 거절. T01 |
| 서비스 계정 예산 | 같은 서비스 계정 참조의 모든 프로젝트·클라이언트가 예약 원장 공유 | 서로 다른 프로젝트의 동시 요청 합계가 한도를 넘지 않음. T02 |
| 계정 참조의 동일성 | API Key 별칭이 다르더라도 동일 계정이면 한도 원장을 분리하지 않는 매핑 | 계정 전환·동일 계정의 복수 Key·오프라인 캐시 정책을 T02에서 검증 |

계정 예산을 프로젝트 DB마다 복제해 합산하는 방식은 여러 DB 사이 예약 원자성을 추가로 해결해야 한다. 사용자 범위 단일 예산 원장을 우선 권고한다. 별도 프로세스/기기에서 같은 계정을 쓰는 비용까지 통제한다는 보장은 없다.

현재 D06은 SID별 엔진·GUI 종료 후 작업 유지·busy 정상 종료 거절·무클라이언트/무작업 5분 idle 종료로 위임 채택했다. 실험의 작업공간별 엔진·30초 제한·idle fixture를 그대로 제품 정책으로 승격하지 않는다.

### 후보 파일과 메타데이터의 대조 복구

현재 엔진은 `.tmp` 쓰기 → `.fixture` rename → DB succeeded 순서다. 두 fault injection에서 잔여 파일을 확인했으며, DB의 running을 interrupted로 바꾸는 것만으로 파일과 메타데이터가 일치하지 않는다.

권고는 작업·출력 버전 ID, 상대 경로, 예상 파일 해시와 준비/확정 단계를 기록하는 것이다. 파일을 덮어쓰지 않고 새 버전으로 확정하며, 엔진이 프로젝트를 다시 열 때 기록과 실제 파일을 대조한다. 원격 생성 재요청과 로컬 파일 복구는 별도 작업이다.

| 재시작 시 상태 | 권고 처리 | 금지 |
| --- | --- | --- |
| 임시 파일만 있음 | 미완료 후보로 식별하고 검증 전 사용 차단, 작업 interrupted 유지 | 자동 성공 처리·원본으로 취급·무조건 삭제 |
| 최종 파일이 있고 DB 확정 전 | 준비 기록의 버전·경로·해시와 일치할 때만 로컬 확정 재개 가능하도록 설계 | 이름만으로 연결·원격 자동 재생성 |
| DB 확정인데 파일 누락/해시 불일치 | 사용 불가 상태로 표시하고 복구에 필요한 근거 보존 | 원본/승인본 덮어쓰기·조용히 성공 유지 |
| 기록에 없는 파일 | 격리 또는 검토 대기로 표시하는 정책 검토 | 사용자의 파일 자동 삭제·기존 자산에 임의 연결 |
| 원격 응답·과금 결과 불확실 | 예산 예약과 outcome_unknown을 보존하고 별도 확인 | 로컬 파일 누락을 근거로 재요청·예산 자동 환원 |

이 설계는 T01의 가져오기와 T03의 편집 출력, T02의 원격 결과 저장에서 구체화한다. T00-B는 version/status 기반만 구현하므로 후보 복구 기능을 미리 추가하지 않는다. 실제 게임 적용·이전 파일 복원은 T05의 승인/저널 계약을 따르며 이 제안으로 대체하지 않는다.
