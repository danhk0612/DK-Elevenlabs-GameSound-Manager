# 아키텍처와 채택 경계

## 채택 상태

Windows GUI, CLI·MCP, 공통 작업 규칙, 로컬 파일, ElevenLabs 연동과 사람 승인 경계가 요구사항이다. **2026-11-20 사용자의 기술 선택 위임에 따라 D01~D06을 채택했다.** 구체 선택과 미검증은 [현재 결정](docs/T00_A_DECISIONS.md)의 위임 결정 절을 따른다. 후속 사용자 지시 “실기, 점검 무시하고 진행해”로 실기 대기를 해제하고 app/에 T00-B 최소 엔진·CLI·MCP·GUI 실행 기반을 구현했다. 사용자 Windows 화면/청감은 미확인으로 남긴다. `spikes/t00-a/`의 격리 실험은 [결과](docs/T00_A_RESULTS.md)와 [결정 자료](docs/T00_A_DECISIONS.md)를 따르며 채택 구조가 아니다.

## 기술 비교와 선택

| 후보 | 장점 | 비용·위험 | 판단 |
| --- | --- | --- | --- |
| Tauri 2 + React/TypeScript + Rust | GUI와 독립 CLI가 Rust 작업 계층을 재사용하기 좋고 웹 UI 개발 가능 | Rust·MSVC·WebView2, 오디오 재생/파형 연결, IPC·MCP 라이브러리 검증 필요 | 위임 채택. 사용자 실기/제품 검증은 별도 |
| Electron + React/TypeScript + Node | UI·CLI 언어 통일, 웹 생태계 사용 | 런타임 배포 크기, DB·오디오/자격증명 네이티브 의존성 및 패키징 검증 | 비교 대안 |
| .NET + Windows GUI | Windows 파일·인증정보·배포 환경과 통합하기 좋음 | UI 기술 선택, CLI·MCP 재사용과 오디오 처리 경험 검증 필요 | Tauri 실패/부적합 시 비교 |

격리 Windows 자동 실행 증거를 근거로 Tauri/Rust 후보를 선택했다. 사용자 데스크톱과 제품 배포 검증은 자동 실행과 별도다. 데이터는 SQLite, 편집·분석은 FFmpeg/ffprobe 외부 프로세스를 우선 검토한다. 생성용 모델은 서비스 요청 설정이며 제품 내부 AI 모델을 만들지 않는다.

## 권장 구성과 책임

개발 순서는 자동화 경로 우선이다. 각 기능은 공통 명령의 구조화 요청/응답·오류와 CLI·MCP 어댑터, GUI 없는 회귀를 함께 만든다. GUI는 그 명령을 사용하는 최소 상태/실패 확인·재생/비교·설정·사람 승인 창으로 둔다. T04는 누락 연결·실제 클라이언트 호환·전체 흐름 검증을 마무리한다. 장식·상세 대시보드·전문 편집 화면은 현재 우선순위에 포함하지 않는다.

| 구성 | 책임 | 하지 않는 일 |
| --- | --- | --- |
| GUI | 프로젝트·목록·검색·재생·파형·후보 비교·작업 상태·사람 확인 | JS에서 API Key 또는 DB 직접 소유 |
| CLI | 구조화 입력/JSON 출력, 작업 제출·상태 조회 | 별도 생성·검증·적용 규칙 구현 |
| MCP stdio 어댑터 | 도구 스키마와 공통 명령 연결, stdout에는 프로토콜만 출력 | 단순 도구 호출을 사람 승인으로 간주 |
| 공통 작업 계층 | 프로젝트 격리, 버전·큐·한도·승인 정책, 요청 검증 | 화면·클라이언트별 정책 분기 |
| 로컬 엔진 호스트 | 장시간 작업 실행, DB·파일 변경 소유, 클라이언트 연결 | 원격 협업 서버 역할 |
| ElevenLabs 연결 모듈 | 계정·Voice 조회, TTS·SFX 요청, 사용량·결과 기록 | 게임 경로에 직접 저장 |
| 오디오 처리 모듈 | ffprobe 분석, FFmpeg 변환·편집, 기술 검사 | 원본·승인 버전 덮어쓰기 |

작업 흐름은 GUI/CLI/MCP → 공통 명령 → 큐 → 서비스/오디오 처리 → 후보 저장 → 검토이며, 게임 적용은 별도 승인 계약을 통과한다.

## 프로세스·동시 접근 권고

후보 A는 GUI·CLI·MCP가 각각 공통 라이브러리와 DB를 여는 구조다. 프로세스 수는 적지만 작업 소유권·한도 예약·DB와 파일 변경을 모든 진입점에서 조정해야 한다.

후보 B는 사용자별 단일 로컬 엔진이 큐·DB·파일 쓰기를 소유하고 GUI·CLI·MCP가 연결하는 구조다. GUI를 닫아도 장시간 작업을 유지하기 쉽고 쓰기 규칙을 모으지만, 기동 경쟁·인증된 로컬 IPC·종료·재연결 검증 비용이 있다. **사용자 SID별 후보 B를 위임 채택했다.** 항상 실행되는 Windows 서비스는 요구하지 않는다.

제품 IPC는 Windows local named pipe와 현재 사용자 SID 접근 제한으로 채택한다. 별도 DACL 실험과 제품 IPC 통합/원격 차단 검증은 구분한다. 원격 HTTP 서버는 초기 요구가 아니다. 최초 GUI/CLI/MCP 요청에서 자동 기동하고 SID별 중복 기동을 막는다. GUI 종료는 작업 취소가 아니다. 정상 busy 종료는 거절하며 연결된 클라이언트·대기/실행 작업이 모두 없는 상태가 5분 지속되면 idle 종료한다. 이 5분은 위임 설계값이며 기존 fixture 수치와 다르다. 사용자 지시로 이전 실기 선행 게이트를 해제하고 T00-B에서 구현한다. GUI 종료와 작업 취소를 동일하게 취급하지 않는다.

SQLite WAL은 읽기/쓰기 동시성에 도움을 주지만 동시 writer는 하나다. WAL만으로 작업·파일의 안전성을 보장하지 않는다. 프로젝트 DB는 로컬 디스크를 우선 검증하며 네트워크 공유의 DB를 지원한다고 가정하지 않는다. DB 트랜잭션을 원격 생성이나 FFmpeg 실행 동안 유지하지 않는다.

## 저장·데이터 계약 권고

앱 설정·계정 참조·프로젝트 인덱스와 사용자 사운드 작업공간을 분리한다. 앱 설정·엔진 발견 루트는 `%LOCALAPPDATA%\DK-Elevenlabs-GameSound-Manager`, 작업공간은 사용자가 지정한 로컬 디렉터리다. 세부 스키마·프로젝트 하위 경로는 T01 계약에서 확정한다. 제안 논리 구조는 프로젝트별 메타데이터 DB와 originals/, candidates/, exports/, recovery/다. 실행 파일 폴더를 사용자 자료 저장소로 삼지 않는다.

| 개체 | 최소 관계·속성 |
| --- | --- |
| Project | 독립 ID, 표시 이름, 작업공간, 선택적 게임 대상 경로 |
| SoundRequirement / Asset | 필요한 사운드 키와 자산 연결, BGM/SFX/Voice 종류 |
| AssetVersion | 원본/후보, 파일 해시·경로·기술 정보, 부모 버전, 출처 |
| Character / Dialogue | 프로젝트 ID, 캐릭터, Voice 참조, 대사·언어·생성 설정 |
| VoiceReference | 계정 참조 + 원격 Voice ID, 목록 캐시. 프로젝트 연결과 원격 소유 구분 |
| Job / Attempt | 요청 스냅샷, 상태, 시도별 결과·불확실성, 후보 버전·진행률 |
| BudgetReservation | 계정과 작업의 한도 예약·사용·불확실 비용 |
| Validation / Review | 버전 해시, 검사 조건·결과와 주관 검토를 분리 |
| ApplyPlan / Approval / Application | 버전 해시·대상·사람 승인·기존 파일·복구 관계 |

자산·대사·작업은 프로젝트 ID를 포함해 조회/변경을 제한한다. 같은 Voice를 여러 프로젝트에서 참조할 수 있지만 음원·설정은 암묵적으로 공유하지 않는다. 스키마 버전과 마이그레이션은 T01 계약에서 확정한다.

파일 쓰기는 임시 파일 → 검사 → 새 버전 경로로 확정 → 메타데이터 기록을 기본 후보로 삼는다. DB와 파일의 원자적 트랜잭션은 없으므로 중단 시 고아 파일·누락 메타데이터를 대조하는 복구 계약이 필요하다. 가져오기 원본 해시와 외부 원본 위치를 기록하고 승인 버전의 바이트는 불변으로 취급한다.

## 큐·크레딧 권고

작업 상태 후보: queued → running → succeeded / failed / cancelled / interrupted / outcome_unknown. 후보 생성과 검토·승인·게임 적용 상태를 별도로 둔다. 검증 실패는 생성 파일 소실과 같지 않다.

같은 작업 중복 제출을 식별하고, 모든 진입점에서 계정 단위 한도와 작업당 횟수 제한을 원자적으로 예약한다. 세부 단위와 수치는 T02 전 결정한다. 원격 과금 후 응답 유실은 자동 재시도로 중복 과금하지 않고 outcome_unknown으로 남겨 확인한다. 로컬 중복 방지는 서비스의 멱등성을 보장하지 않는다. 취소가 이미 소모된 크레딧 환불을 의미하지 않는다.

서비스 사용량 조회값, 사전 추정, 실제 확인 비용을 구분한다. 다른 프로그램이 같은 계정으로 사용한 비용까지 매니저의 한도로 통제할 수 없다. 서비스 측 API Key quota도 보조 수단으로 검토한다. 미설정 한도를 무제한으로 해석하지 않는다.

## 사람 승인·게임 적용 설계 게이트

이 절은 R12·R13을 충족하기 위한 권고이며 T05에서 실패 복구까지 확정한다.

1. AI는 후보·검증 결과와 적용 계획 초안만 제출한다.
2. GUI에서 사람이 후보, 버전 해시, 대상 경로·기존 파일, 변경 내용을 확인한다.
3. 승인 기록은 해당 계획에만 묶고 AI가 전달한 approved=true 같은 입력으로 만들 수 없게 한다.
4. 실행 직전 후보·대상 상태가 계획과 같은지 재검사한다. 변경되면 재승인을 받는다.
5. 기존 파일을 보존하고 적용 저널을 남긴 뒤 대상 파일을 교체한다.
6. 실패·중단 시 실제 파일·저널을 대조한다. 외부에서 새로 수정한 파일에 과거 복구본을 무조건 덮어쓰지 않는다.

CLI·MCP와 GUI가 같은 공통 정책을 통과해야 한다. 자동 호출에 적용 실행을 제공할지는 승인된 계획만 실행할 수 있는지 검증한 뒤 결정한다. 단순 클라이언트 이름이나 CLI --human 옵션을 신뢰 경계로 삼지 않는다. 같은 OS 사용자의 악성 프로세스까지 차단하는 보안 보장은 범위 밖이다.

## API·비밀정보·오디오

연결 모듈만 API Key를 읽고 GUI password 입력 이외의 조회·프로젝트 DB·인자·로그·내보내기에 Key를 노출하지 않는 구조를 검증한다. D05 위임 결정으로 Windows Credential Manager 현재 사용자 범위를 사용한다. silent DPAPI fallback은 넣지 않고 실제 Key·수명 검증은 T02다. 실제 API·계정 제한 근거는 API_RESEARCH.md를 따른다.

FFmpeg/ffprobe 호출은 셸 문자열 대신 인자 배열을 사용하고 원본과 다른 출력 경로를 쓴다. 재생 지원 형식과 변환 지원 형식은 다를 수 있으므로 WebView2 재생을 실제 파일로 검사한다. 기본 형식·음량/무음 임계값·루프 단위는 D08이며 임의 상수로 확정하지 않는다.

## 검증 게이트와 근거

T00-A: Windows GUI + 독립 CLI + stdio MCP, GUI 종료 후 작업, 두 클라이언트 경쟁, 강제 종료·재기동, 한글/공백 경로, 자격증명 보안, 실제 FFmpeg 파일 처리, 정확한 SHA의 Windows artifact·실행 결과를 확보한다. 실기 미확보는 기록하되 2026-11-20 후속 사용자 지시로 개발 진행을 막지 않는다.

공식 근거(2026-10-15 열람): [Tauri process model](https://tauri.app/concept/process-model/), [Windows prerequisites](https://tauri.app/start/prerequisites/), [SQLite WAL](https://www.sqlite.org/wal.html), [MCP stdio](https://modelcontextprotocol.io/specification/2026-07-28/basic/transports/stdio), [FFmpeg 배포 조건](https://www.ffmpeg.org/legal.html). 문서 지원 사실과 이 프로젝트의 실측을 구분한다.

초기 패키징은 포터블 ZIP을 선택했다. WebView2는 외부 준비, FFmpeg/ffprobe는 사용자 지정 경로/PATH로 사용하고 자동 설치/동봉하지 않는다. 설치/업데이트/고지·공개 배포 허용은 T06 및 D11의 별도 게이트다.

## T01 실제 저장·명령 계약

[로컬 라이브러리 계약](docs/T01_LOCAL_LIBRARY.md)을 현재 구현 기준으로 사용한다. LOCALAPPDATA catalog와 사용자 선택 workspace의 프로젝트별 schema 1 SQLite/불변 버전 파일을 분리하고 모든 자료 명령에 project_id를 요구한다. 엔진의 단일 writer 스레드가 로컬 명령을 순차 처리하며 queued/running 중 정상 종료를 거절한다. import는 예상 SHA journal → 별도 복사/hash/sync → no-replace hard link 게시 → ready 기록 순서다. 중단 대조는 copying 버전에 한정하며 임의 파일/전원 장애 전체 복구를 보장하지 않는다. CLI schema와 MCP 17개 도구는 같은 명령 카탈로그·검증을 사용한다. 상세 필드/byte 제한/페이지/preview/오류 경계는 위 계약을 따른다. 계정 예산·원격 작업 큐와 별개다.

## T02 실제 계정·큐 계약

[생성 계약](docs/T02_GENERATION.md)을 현재 구현 기준으로 사용한다. catalog schema 2에 계정/workspace identity·공유 한도·불변 요청 snapshot과 attempted 상태를 저장한다. 프로젝트 DB는 schema 1이다. Windows Credential Manager만 실제 Key와 유료 허용/한도 정책을 보관하고 CLI/MCP는 credential 입력을 받지 않는다. D09 숫자 미설정은 거절이며 실제 청구액 상한을 보장하지 않는다.

단일 writer의 timeout tick으로 persistent 큐를 실행하고 status/job.get/list를 원격 요청과 분리한다. POST 직전 attempted를 기록하며 원격 자동 재시도가 없다. output_ready spool을 T01 journal로 후보 등록하고 재기동 시 미완료 요청을 자동 재전송하지 않는다. 상세 한도 유지·취소/복구·오류·32개 명령/31개 MCP 도구는 T02 계약을 따른다. 과거 T01의 원격 큐 미포함/17개 도구 표기는 v0.2.0 기반 설명이다.

## T03 실제 편집·분석 계약

[편집/분석 계약](docs/T03_AUDIO.md)에 따라 catalog schema 3에 도구 경로/hash/version과 프로젝트별 operation snapshot·입력 hash·상태/결과를 추가했다. 프로젝트 DB는 schema 1이다. FFmpeg/ffprobe는 명시적인 신뢰 경로로 설정하며 shell 없이 제한된 인자/timeout/출력 크기로 실행한다. 현재 local audio 명령은 writer에서 동기 실행하며 status는 분리된다. 장시간 async·취소는 후속이다.

출력/검사 수치는 호출마다 명시하고 D08의 제품 기본값을 만들지 않는다. decoded 첫 스트림의 채널별 샘플을 유지해 peak/RMS/quiet/파형·loop seam을 측정한다. 후보는 별도 출력·hash/output_ready → T01 journal 등록 순서로 저장하고 원본/부모/게임 파일을 보존한다. 처리 중 엔진 종료는 interrupted, 수신 완료 파일은 명시적 등록 복구 대상이다. 검증 수치는 주관 적합성/승인과 별개다. 총 39개 명령/MCP 38개 도구가 같은 검증을 사용한다. 과거 절의 schema/명령 수는 해당 버전 기반이다.

## T04 비동기 오디오 보강

[비동기 큐/취소 계약](docs/T04_ASYNC_AUDIO.md)에 따라 catalog schema 4에 cancel_requested/phase/tools_snapshot을 추가했다. 기존 동기 명령은 유지하고 audio.submit은 동일 편집/분석 입력을 정규화하여 즉시 예약한다. 실행 당시 도구/hash·입력 버전을 고정하고 단일 엔진 실행기가 큐를 처리한다. 짧은 제출/상태 조회/취소는 migration/재시작 정리를 하지 않는 별도 catalog 연결로 처리한다.

취소 플래그를 외부 프로세스/PCM 측정·단계 사이에서 대조하고 output_ready 전환/분석 완료 기록과 원자적으로 조정한다. output_ready 이후에는 취소 대신 등록/복구를 보존한다. 재기동 자동 재생은 없고 큐/실행은 interrupted 또는 이미 취소 요청됐으면 cancelled다. 실행 중 status와 audio 제출/조회/취소를 사용할 수 있으며 다른 일반 명령은 계속 직렬일 수 있다. 전체 41개 공통 명령/MCP 40개 도구다. T04 실제 AI 클라이언트 호환·검토 제출과 T05 승인/게임 적용은 별도다.
