# T00-A 결정 자료 및 T00-B 조건부 계약

이 문서는 권고안이다. **D01~D06 사용자 결정 없음, T00-B 착수 불가.**
실측은 [결과](T00_A_RESULTS.md), Windows 수집은 [절차](T00_A_WINDOWS_CHECK.md)를 따른다.

## D01~D06

| ID | 근거·비교 | 현재 권고 | 사용자 결정 필요 / 남은 증거 |
| --- | --- | --- | --- |
| D01 기술 | Rust 공통 request 함수를 GUI invoke·독립 CLI·stdio MCP에서 재사용. CLI/MCP/SQLite/FFmpeg Linux 실제 프로세스 검사 통과. GUI UI TypeScript 빌드 통과. Windows GUI/독립 진입점 빌드·자동 실행도 통과(정확한 SHA는 결과 문서). GUI 화면/명령/청감은 미확인. Electron/.NET은 실행 비교하지 않았음 | Tauri 2+React/TS+Rust+SQLite를 우선 후보로 유지. 런타임 크기·개발 비용 우열은 미실측 | Windows GUI 명령·청감 후 이 후보 채택 또는 비교 실험 추가. 실패 원인 없이 스택 교체 금지 |
| D02 지원 OS | CI는 Windows Server 2022 x64, 사용자 Windows 데스크톱과 다름. Linux 검사는 Windows 근거 대체 불가 | 우선 Windows x64로 실기, ARM/32비트 약속하지 않음 | 지원할 Windows 버전과 아키텍처. Windows 10/11, WebView2 미설치, 비관리자 환경 검사 필요 |
| D03 배포 | artifact는 설치 없는 실험 EXE+검사 스크립트. FFmpeg와 WebView2는 외부 준비, 설치/제거/갱신 실험 없음 | 실행 검증에는 압축 artifact. 정식 포터블·설치형 채택 보류 | 최초 제품 제공 형식과 런타임 동봉 여부. 서명·업데이트·고지·배포 허용은 T06 |
| D04 저장 | 필수 환경변수로 임시 작업공간 분리, 한글/공백 경로 DB·파일·FFmpeg. 실제 원본 대신 합성 신호 hash 보존. 네트워크/동기화 폴더 미검증 | 앱 설정과 사용자 작업공간 분리, 가져오기는 복사 권고 유지 | 설정 위치·프로젝트 폴더 선택 방식·복사 정책. 기본값 미확정; 프로그램 폴더 저장 금지 권고 |
| D05 자격증명 | Windows Dummy Credential Manager 쓰기/읽기/삭제와 DPAPI CurrentUser 메모리 왕복 스크립트. API Key 사용 없음. Windows CI에서 두 후보의 dummy 왕복과 Credential 삭제 통과; 원시 보고서에 기록. 다른 사용자·재부팅·프로필 이동·내보내기·로그 경계 미검증 | Credential Manager 우선 검토: 직접 암호문 파일·ACL 관리 부담 감소. DPAPI는 앱이 암호문 파일 수명·ACL·백업을 책임져야 함 | 어느 후보 채택할지. 양쪽 모두 같은 사용자 프로세스에 대한 보안 격리 보장 아님. dummy 왕복만으로 R04 충족 선언 금지 |
| D06 프로세스 | 단일 엔진이 DB/파일 writer. 클라이언트 종료 후 지속, 중복 기동 거절, 동시 id/모의 예산, 강제/정상 종료 복구 검증. 직접 접근은 Python SQLite writer 잠금 fixture만 비교; 완성 대안 구현 아님 | 단일 로컬 엔진 소유 방식 우선 유지. GUI 닫기≠취소; 중단 작업 자동 재시도 금지 | 수동/자동 기동, busy 종료, idle 수명·시간값, IPC 방식과 사용자 범위. 이 실험 TCP/token을 제품 IPC로 채택하지 않음. 사용자 ACL named pipe·부팅 경쟁·두 작업공간 소유권·파일/DB 대조 복구는 추가 T00-A 검증 게이트 |

## 구조·수명 비교

| 방식 | 실험·관찰 | 추가 책임 |
| --- | --- | --- |
| 각 클라이언트 직접 공통 라이브러리/DB 접근 | SQLite 두 writer는 동시에 쓰지 못하는 fixture를 재현. GUI/CLI/MCP 각각의 실제 작업 소유 구현은 하지 않았음 | 진입점마다 예산/중복/파일 소유권을 공유하고 클라이언트 종료에도 작업을 유지해야 함 |
| 단일 엔진 + 어댑터 | 같은 DB에 쓰는 엔진 1개, 모든 요청 직렬 수락. 작은 DB 트랜잭션 후 모의 작업 수행. 읽기/쓰기 부하 성능은 측정하지 않았음 | IPC 인증·엔진 주소 발견·기동 경쟁·수명 정책 필요. 단일 루프라 느린 클라이언트가 잠시 요청 처리를 지연시킬 수 있음 |

임시 모의 한도 10·요청 비용 1/3·최대 지연 30초는 검사 값이며 D09의 제품 정책이 아니다. endpoint token은 엔진 기동마다 교체한다. Windows token 파일 ACL과 타 사용자 접속 방지는 검증하지 않았으므로 실험용 dummy 자료만 허용한다. 엔진 단일성은 작업공간 lock 기준이며 OS 사용자 전체 단일성이 아니다. 자동 기동·idle 종료는 구현하지 않았다.

복구 fault injection은 임시 파일 기록 직후와 최종 파일 rename 직후에 프로세스를 종료한다. 재기동 시 작업은 interrupted, 예산은 보존되고 파일은 남는다. **파일·DB 자동 대조와 원자성은 미구현이며 한계 발견을 복구 완성으로 해석하지 않는다.** 네트워크 과금/응답 유실은 시뮬레이션하지 않았으며 T02 outcome_unknown 계약이 필요하다.

## T00-B 계약 초안 — 선행 충족 전 실행 금지

- 선행: T00-A의 정확한 수정 SHA Windows 빌드·artifact, GUI 공통 명령·닫기/재연결·합성 재생 실기와 남은 IPC/수명 검증. D01~D06 사용자 결정 및 기준 문서 반영. T00-A를 완료로 닫을 근거 검토.
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
