# Linux ElevenLabs 인증 세션·HTTPS (v0.10)

Windows Credential Manager/WinHTTP는 유지한다. Linux는 GUI/secret service/평문 저장을 추가하지 않고 엔진 시작 때만 받는 메모리 인증 세션과 Rust HTTPS 전송을 추가한다. 공통 명령/CLI/MCP 47/46개·DB schema 5/1은 같다. 생성은 기존 job.submit/get/list/cancel/recover·예산·후보 보존 계약을 따른다. 이 기능을 실제 Key/과금/청감 검증 성공으로 해석하지 않는다.

## 실행 절차

1. [Linux private root](T04_LINUX_HEADLESS.md)를 정하고 기존 엔진을 정상 shutdown한다. 진행 작업이 있으면 종료 거절을 확인하고 기존 상태를 대조한다. 키 없이 자동 기동한 기존 엔진에 세션이 소급 주입되지는 않는다.
2. 같은 root 환경에서 아래 런처를 실행한다. 숫자와 계정 ID는 예시 placeholder이며 실제 한도·TTL을 사용자가 명시해야 한다. Key는 터미널의 숨김 입력에 넣고 채팅/명령 인자/JSON 작업 파일에 넣지 않는다.

```bash
export DK_SOUND_ROOT=/absolute/private-state
python3 app/automation/linux_session.py \
  --engine /absolute/path/dk-sound-engine \
  --account-id account-id \
  --session-seconds TTL_SECONDS \
  --request-limit REQUEST_LIMIT \
  --credit-limit CREDIT_LIMIT
```

기본은 조회 전용이다. 생성 허용에는 `--enable-generation`과 양수 한도를 명시한다. 실제 API Key 자체의 ElevenLabs quota도 별도 설정한다. 엔진은 foreground로 유지되고 다른 터미널/AI가 같은 root의 CLI/MCP를 호출한다. 일반 자동 기동은 secret을 갖지 않는다. 첫 기동에 실패하면 출력된 정적 오류 코드만 확인하고 기존 엔진을 강제 종료하거나 새 요청 ID로 반복하지 않는다.

3. `account.create`의 provider를 `elevenlabs`로 등록하고 `account.get`의 credential_configured=true를 확인한다. 이 두 명령은 네트워크를 호출하지 않는다. `account.refresh`는 실제 GET /v1/user를 호출하므로 실제 연결 허용 범위에서만 실행한다. voice/model 조회도 네트워크 요청이다.
4. 실제 생성은 [T02 명령/schema](T02_GENERATION.md)에 따라 명시한 text/model/output/예약량·고유 job_id로 job.submit한다. 실제 POST 전에는 엔진 메모리의 enabled·TTL·계정과 workspace identity/누적 예산을 다시 대조한다. TTS/SFX는 같은 경로를 사용한다. 생성 상태와 결과 후보 SHA를 확인한다. 검토 제출은 게임 적용 승인이 아니다.

외부 비밀정보 공급자가 있는 무인 환경은 런처의 `--key-fd FD`에 private inherited pipe/socket FD를 제공할 수 있다. 파일·TTY FD는 거절하며 key 파일명/환경 변수 옵션은 없다. FD 내용은 ASCII key 한 개(최대 256자, 선택 마지막 줄바꿈)여야 하며 공급자는 EOF를 닫아야 한다. 런처는 Python 3.11+ 표준 라이브러리만 사용한다. 숨김 터미널 입력이 불가능하면 echo fallback 대신 거절한다. 실제 키를 쉘 문자열로 만드는 예시는 제공하지 않는다.

## 엔진 시작 입력과 수명

Linux 엔진 전용 `dk-sound-engine --session`은 pipe/socket stdin에 UTF-8 JSON 한 개와 EOF를 받는다. 일반 CLI request/validate·MCP·GUI 명령이 아니다. 필드: account_id, key, enabled, request_limit, credit_limit, session_seconds. 전부 필수이며 unknown/duplicate 필드는 거절한다. 계정 ID는 기존 공통 규칙, key는 1..256 ASCII graphic 문자, TTL은 60..86400초다. disabled는 0 한도를 허용하고 enabled는 request_limit 1..1,000,000·credit_limit 1..1,000,000,000을 요구한다. 입력 최대 4096 bytes이며 크기/IO/구문/정책 오류에 입력이나 parser 상세를 출력하지 않는다. 세션 입력의 escape가 필요한 key 문자열은 지원하지 않는다.

엔진은 core dump와 Linux dumpable을 먼저 끄며 실패하면 secret을 읽지 않는다. 입력은 Zeroizing buffer에서 borrowed deserialize하고 key는 Zeroizing 메모리에 둔다. root·계정에 한정한 하나의 세션이며 변경/조회/갱신 IPC를 추가하지 않는다. TTL은 monotonic clock이며 읽기로 연장되지 않는다. 만료 시 새 조회/전송을 막고 엔진 tick에서 저장 key를 제거한다. 진행 중 HTTP가 이미 가진 일시적 key/header 복사는 TTL 만료로 취소되지 않는다. 종료/실패 후 세션은 복원하지 않으며 queued/interrupted/outcome_unknown의 재시작 무재생 규칙은 유지한다.

계정 metadata와 누적 예산은 보존하지만 key는 DB/상태 파일/로그/argv/env에 저장하지 않는다. Rust 원본 버퍼와 저장 key는 지우지만 Python/HTTP/TLS 라이브러리의 모든 내부 복사·swap·관리자 메모리 접근까지 제거를 보장하지 않는다. Python 런처도 core dump를 끄고 key/입력을 출력하지 않는다. 동일 UID/관리자/수동 DB 삭제로 한도를 우회하는 OS 보안 제품은 아니다. Linux root별 카탈로그이므로 다른 root/기기/외부 프로그램의 지출을 합산한다고 주장하지 않는다.

## 한도와 오류

생성 활성화·TTL·한도는 세션의 명시적 설정이다. `budget.set`은 계속 mock 전용이며 실제 계정에 `live_policy_requires_gui`를 반환한다. 이 기존 오류명은 Windows 호환을 위해 유지하되 Linux에서는 새 세션 시작 입력이 설정 경로다. 실제 identity refresh가 같은 workspace 예산을 결합하고 기존 한도를 더 작게만 적용한다. 세션 재시작/더 큰 한도를 보내도 이미 소진한 예산이나 기존 한도를 초기화하지 않는다. 한도 증액/정산은 이번 범위 밖이다.

키 없음/다른 계정/일반 재시작은 credential_unavailable, 만료 관측 직후는 credential_session_expired 또는 제거 후 credential_unavailable이다. 생성 disabled·미설정 예산·부족/비용 초과는 기존 오류를 따른다. POST attempted 기록 후 전송/응답 실패는 outcome_unknown과 예약 보존으로 처리하며 재시도하지 않는다. 서버 비용 헤더와 실제 청구액은 다르고 billing_cap_guaranteed=false다. D08/D09 기본 수치는 확정하지 않았다.

## 전송

Linux reqwest 0.13.5 blocking/rustls/ring, OS trust store의 인증서·호스트 검증, 고정 https://api.elevenlabs.io, GET/POST, HTTP/1, connect 10초/전체 120초다. 자동 retry/redirect/proxy/Referer/TLS key logging을 명시적으로 끈다. endpoint·CA·proxy override를 제품 옵션/환경 변수로 추가하지 않는다. HTTP 라이브러리 오류는 정적 provider_transport_error로만 돌려주며 URL/header/body를 출력하지 않는다. 인증 header는 sensitive 표시한다.

200 응답만 본문을 읽고 최대 32 MiB를 Content-Length 및 실제 읽기로 제한한다. 비200 오류/리디렉션 body는 읽거나 반환하지 않는다. character-cost는 비음수 정수일 때만 confirmed credit으로 사용하고 없으면 예약을 유지한다. 생성 POST는 한 번이며 응답 유실/429/다른 모델로 재요청하지 않는다. 실제 Key/요금제/네트워크/TLS/응답/청감은 라이브 검증 전까지 미확인이다.

## 비용 없는 증거

- Rust 세션 입력/TTL/계정 범위와 고정 origin/path 검사.
- 실행 시 생성하는 임시 인증서·가짜 키·로컬 TLS 서버로 같은 client 설정의 200/비용 헤더·401/429·302 리디렉션 차단·본문 크기·응답 유실 단일 시도·미신뢰 인증서/HTTP 거절을 확인한다. 제품 origin override는 없다.
- `python3 app/automation/ci_linux_session_smoke.py --bin-dir app/target/debug`: 실제 engine의 제한 stdin 입력·계정 scope·비밀정보/정책 IPC 거절·재시작 후 미설정·DB/파일에 가짜 키 미보관·private FD 런처를 확인한다. metadata-only 명령만 호출하며 HTTP는 0회다.
- 기존 mock 큐·예산·중단 무재생·실제 FFmpeg와 CLI/MCP 회귀를 유지한다. 인증 세션과 로컬 TLS 단위 검사를 실제 ElevenLabs 생성으로 보고하지 않는다.

구현 근거를 2026-11-20 UTC에 확인했다: [Get user/workspace](https://elevenlabs.io/docs/api-reference/user/get), [TTS](https://elevenlabs.io/docs/api-reference/text-to-speech/convert), [SFX/character-cost](https://elevenlabs.io/docs/api-reference/text-to-sound-effects/convert), [reqwest 기능/기본 proxy·redirect](https://docs.rs/reqwest/0.13.5/reqwest/), [retry::never](https://docs.rs/reqwest/0.13.5/reqwest/retry/fn.never.html). 문서 확인은 계정/과금 실측이 아니다.
