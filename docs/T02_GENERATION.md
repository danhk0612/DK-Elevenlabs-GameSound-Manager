# T02 — 계정 연결·한도·후보 생성 v0.3.0

v0.5.0의 catalog schema 4·41개 명령/MCP 40개 도구, 비동기 제출/조회·취소는 [T04 보강 계약](T04_ASYNC_AUDIO.md)을 함께 따른다. 이 문서의 동기 명령은 유지한다.

이 문서는 v0.3.0의 생성 계약이다. v0.4.0 catalog schema 3·총 39개 명령/MCP 38개 도구와 로컬 편집/분석은 [T03 계약](T03_AUDIO.md)을 함께 따른다.

AI 자동화용 공통 명령을 먼저 제공한다. Windows engine/CLI/stdio MCP가 같은 정책·큐를 사용한다. T01 기능은 유지하며 총 32개 명령, shutdown을 제외한 MCP 31개 도구다. `dk-sound-cli schema`가 정확한 필수/선택 필드와 형식의 기준이다. GUI는 기존 목록/재생에 Key·한도 설정/연결 해제만 추가했다.

## v0.10 Linux 실행 확장

[Linux 인증 세션/HTTPS](T02_LINUX_PROVIDER.md)는 엔진 시작 시 private pipe의 명시적 key/계정/허용/한도/TTL을 받는다. Windows GUI/Credential Manager는 유지한다. 아래 GUI 설정 필수 문장은 Windows의 계약이며 Linux는 저장 없는 세션 설정을 사용한다. 공통 CLI/MCP에 key/실제 예산 설정 명령을 추가하지 않는다. Linux에서도 기존 생성 큐·workspace ledger·POST 단일 시도·후보/복구를 재사용한다. 실제 ElevenLabs 키·과금은 미검증이다.

## 연결과 비밀정보

GUI의 계정 ID는 lowercase stable ID다. API Key를 password 입력으로 받아 Windows Credential Manager 현재 사용자 Generic credential에 보관한다. 저장 시 입력을 지운다. Key 조회·CLI/MCP 전달·환경변수·평문 설정/DB fallback은 없다. 연결 해제는 credential과 생성 허용 설정을 삭제한다. 계정 메타데이터·기존 작업/음원/누적 한도 기록은 남는다. 해제는 이미 전송된 요청을 취소하지 않는다.

서비스 공급자 `elevenlabs` 또는 `mock`은 account.create로 고정된다. 동일 ID에 공급자를 바꾸지 못한다. 실제 계정은 GUI에서 설정 전에는 연결되지 않으며 유료 생성은 기본 해제다. Key 없이 mock으로 전체 자동화 경로를 실행할 수 있다. Mock은 독립적인 합성 WAV를 만들 뿐 ElevenLabs 음성·효과음 품질/과금의 대용 증거가 아니다.

account.refresh는 GET /v1/user의 workspace_id를 hash한 identity로 계정을 결합한다. 같은 workspace의 서로 다른 Key/계정 ID는 여러 프로젝트에서도 한 ledger를 공유한다. 기존 ID의 workspace가 바뀌면 account_identity_changed로 거절한다. workspace_id가 없는 응답도 거절하며 ID별 한도로 우회하지 않는다. 서비스 사용량 character_count/character_limit는 조회 시점의 참고값이다.

## 한도 계약과 제한

D09의 기본 숫자는 확정하지 않았다. **미설정은 생성 불가**다. 실제 유료 생성 활성화와 요청/예약 크레딧 한도는 GUI의 명시적인 설정을 요구한다. budget.set은 mock 전용이며 CLI/MCP로 실제 계정 정책을 설정할 수 없다.

한도는 이 PC의 해당 catalog 수명 전체에 적용된다. 일/월 자동 초기화가 없다. 실제 계정의 기존 한도는 refresh 시 더 작은 값으로만 갱신한다. 별칭 Key로 한도를 늘릴 수 없다. 한도 증액·초기화·다른 PC와의 동기화는 미구현이다. DB 수동 삭제/변조를 보안 경계로 막는 제품이 아니다.

job.submit은 transaction 안에서 요청 1회와 입력 reserve_credits를 예약한다. 실제 비용 산식으로 자동 추정하지 않는다. ledger는 취소된 미전송 작업만 제외하고, 각 작업의 max(예약량, character-cost 응답 헤더 확인량)를 계속 잡아 둔다. 헤더 누락·실패·불확실 결과도 예약을 풀지 않는다. 확인 비용이 예약을 초과하면 초과량까지 계산하여 후속 작업 제출/실행을 막는다. 이미 전송한 요청의 초과 비용을 되돌릴 수는 없다.

**예약 크레딧 한도는 실제 청구액 상한 보장이 아니다.** 비용 헤더가 없는 요청, 잘못 정한 예약량, 외부 프로그램/다른 PC의 사용을 모두 통제할 수 없다. 실제 호출 전 서비스 측 API Key quota도 설정한다. 실제 요금·무료 계정 권한·초과 과금 동작은 이번에 실측하지 않았다. billing_cap_guaranteed는 false다.

## 공통 명령

| 명령 | 필수 입력 | 설명 |
| --- | --- | --- |
| account.create | account_id, provider | metadata 생성/동일 입력 재사용 |
| account.get / account.refresh | account_id | 로컬 연결 상태 / 원격 identity·사용량 조회 |
| budget.set | account_id, request_limit, credit_limit | mock 한도 설정 |
| budget.get | account_id | identity, 한도·누적 예약/확인량 |
| voice.list | account_id | 최대 10개, 선택 page_token |
| voice.get / voice.settings | account_id, voice_id | 저장 Voice 메타데이터/설정 읽기 |
| model.list | account_id | 모델·TTS 가능 여부 읽기 |
| job.submit | 아래 예시·schema | async 큐 등록, 즉시 상태 반환 |
| job.get / job.cancel / job.recover | project_id, job_id | 상태 / 미전송 취소 / 받은 파일 등록 복구 |
| job.list | project_id | after/limit 페이지, 최대 10개 |

Voice preview_url은 HTTPS 메타데이터로만 반환한다. 원격 preview 다운로드/GUI 재생은 아직 제공하지 않는다. Voice Design 미리보기/저장·Remix·Music·Voice 변경/삭제는 없다. 캐릭터 Voice 할당은 기존 character.put으로 한다.

TTS는 voice_id 필수, model.list에 있는 TTS 가능 model_id를 명시한다. 선택 language, voice_settings(stability/similarity_boost/style 0..1, speed 0.7..1.2, use_speaker_boost boolean)를 지원한다. dialogue_id를 지정하면 저장 대사의 text/language/캐릭터 Voice와 요청 snapshot이 모두 같아야 한다. SFX 전용 필드는 TTS에 허용하지 않는다.

SFX는 duration_ms 500..30000 필수, loop boolean, prompt_influence 0..1 선택이다. 실제 model_id는 eleven_text_to_sound_v2/v3를 지원한다. SFX에 음성/대사 필드는 허용하지 않는다. 초기 원격 출력은 mp3_44100_128만 허용한다. 보유 요금제/모델의 추가 제한은 provider_access_denied/provider_http_error 등으로 남으며 자동 대체 호출하지 않는다. mock 출력은 요청 형식과 구별되는 합성 PCM WAV다.

text는 UTF-8 최대 2048 bytes다. request_limit은 1..1,000,000, credit_limit/reserve_credits는 1..1,000,000,000 정수다. 기타 공통 ID/필드/64 KiB frame/페이지 계약은 T01을 따른다.

## Key 없는 CLI 예시

기존 로컬 workspace에 project_id=demo를 먼저 만든다. 다음 JSON을 각각 UTF-8 파일로 저장해 `dk-sound-cli request < request.json`으로 제출한다. PowerShell은 `Get-Content -Raw -Encoding utf8 request.json | .\dk-sound-cli.exe request`를 사용할 수 있다. 예시 한도는 mock fixture용이고 D09 기본값이 아니다.

```json
{"protocol_version":1,"op":"account.create","account_id":"demo-key","provider":"mock"}
```
```json
{"protocol_version":1,"op":"budget.set","account_id":"demo-key","request_limit":3,"credit_limit":300}
```
```json
{"protocol_version":1,"op":"job.submit","project_id":"demo","account_id":"demo-key","job_id":"click-001","asset_id":"click","version_id":"take-001","name":"버튼 클릭","generation_kind":"sfx","text":"A short dry wooden click, no ambience","model_id":"mock-sfx","output_format":"mp3_44100_128","reserve_credits":100,"duration_ms":500}
```
```json
{"protocol_version":1,"op":"job.get","project_id":"demo","job_id":"click-001"}
```

실제 연결은 GUI에서 생성한 account_id를 사용하고 공식 SFX model_id로 바꾼다. 이 예시를 실제 계정에 자동 실행하지 않는다. 생성 결과는 asset.versions/file·GUI로 확인한다. 이미 존재하는 asset에는 동일 name/kind와 ready parent_version_id가 필요하다.

## 실행·중단·복구

단일 writer가 명령과 생성 요청을 직렬 처리한다. 등록된 큐는 클라이언트/GUI 종료 후에도 엔진에서 진행된다. status와 별도 read-only DB의 job.get/list는 원격 요청 중에도 조회할 수 있다. 다른 자료 명령·취소는 진행 중 요청 뒤에서 처리될 수 있다. 큐/실행 중 정상 shutdown은 거절하고 idle 종료하지 않는다.

job_id는 전역 고유하며 같은 project와 정확히 같은 입력 snapshot 재제출 시 기존 상태를 돌려준다. 다른 입력은 job_id_conflict다. 대상 project/asset/version도 예약하여 별도 import와 충돌을 막는다. 취소 이후에도 대상 ID를 다른 요청에 재사용하지 않는다.

상태: queued → running → succeeded. 전송 전 실패는 failed_preflight, HTTP 실패는 failed, 네트워크/응답 불명은 outcome_unknown이다. 출력 수신 후 등록 실패는 output_ready다. job.cancel은 attempted=false인 queued/interrupted/failed_preflight만 cancelled로 만든다. 서버에 전송한 요청의 중단/환불은 제공하지 않는다.

POST 직전 attempted=1을 영속 기록한다. POST를 자동 반복하지 않는다. 재기동 시 queued/running은 attempted 여부에 따라 interrupted/outcome_unknown으로 바꾸고 재전송하지 않는다. 미전송 interrupted는 명시적으로 취소할 수 있다. outcome_unknown은 예약을 보존하며 원격 비용 대조/정산 기능은 후속이다. 새 ID로 반복 생성해 문제를 숨기지 않는다.

성공 응답은 LOCALAPPDATA/generated의 새 job_id 파일에 create_new·sync 후 hash와 output_ready를 기록하고 T01 복사 journal로 후보를 등록한다. 처음 만든 음원도 candidate이며 원본/기존 버전·게임 파일을 바꾸거나 승인하지 않는다. job.recover는 이 파일의 hash를 재검사하여 등록만 반복하고 원격 생성은 하지 않는다. spool 파일은 증거로 남기며 자동 정리하지 않는다. 파일 수신/DB 기록 사이 장애 등 모든 전원 장애에 대한 복구를 보장하지 않는다. output_ready 외 상태의 파일을 임의 채택하지 않는다.

catalog schema 1→2 migration은 transaction으로 accounts/budgets/generation_jobs를 추가한다. 프로젝트 DB는 schema 1 유지다. 업데이트 전 정상 엔진 종료 후 DB/작업공간과 WAL을 함께 일관되게 백업한다. v0.2.0은 schema 2를 읽지 못하므로 되돌릴 때 새 자료를 버리는 자동 다운그레이드를 하지 않는다. 정상 종료된 이전 백업과 대응 바이너리를 별도 위치에서 사용한다.

## 전송·검증 경계

Windows WinHTTP 시스템 TLS, 고정 api.elevenlabs.io:443, redirect 거절, 제한된 응답 크기 32 MiB와 connect/send/read timeout을 사용한다. 오류 body와 Key를 반환/로그하지 않는다. 생성 요청은 POST 한 번이며 429/backoff/다른 모델로 자동 재시도하지 않는다. 응답 형식은 WAV/mock 또는 MP3 header의 최소 식별만 수행한다. 실제 디코딩·길이/음질·루프 적합 검사는 T03다.

Rust bundled SQLite 19개 테스트에서 mock TTS/SFX 후보 생성, 원본 보존, 정책 미설정 거절, 프로젝트/계정 별칭 공유, ID 중복, 취소/재시작, 불확실 응답/429의 단일 POST, 비용 초과 차단, 파일 등록 복구, secret/잘못된 입력 거절을 확인했다. Windows credential/HTTP는 타입 검사와 빌드 대상으로만 확인하며 실제 Key/요금제/청감 검증을 대신하지 않는다. 실기 대기 없이 후속 자동화 개발을 계속한다.
