# Windows에서 NAS 설정 파일로 연결

사용 환경: Windows에서 NAS 파일을 편집하고 Codex CLI/사운드 엔진은 Docker Linux에서 실행한다. 사용자가 명시적으로 요청한 방식이다. 명령어로 key를 입력할 필요가 없다.

## 사용자가 할 일

저장소 최상단의 **elevenlabs.local.json**을 Windows 편집기로 연다. 현재 NAS 연결에서는 폴더 `DK-Elevenlabs-GameSound-Manager` 안에 있다. 없으면 Git에 포함된 elevenlabs.config.example.json을 같은 폴더에 elevenlabs.local.json 이름으로 복사한다. 이미 있는 실제 설정 파일은 예제나 업데이트로 덮어쓰지 않는다.

| 항목 | 입력 |
| --- | --- |
| api_key | 실제 ElevenLabs API key |
| enable_generation | 생성 검증을 허용하려면 true. false면 조회만 가능 |
| request_limit | 허용할 총 생성 요청 횟수. 생성 허용 시 양수 정수 |
| credit_limit | 총 예약 크레딧 한도. 생성 허용 시 양수 정수 |
| session_seconds | 세션 유효 초, 60..86400. 현재 예제 3600은 1시간 |
| account_id | 현재 elevenlabs-main 유지 가능 |
| state_root | 빈 문자열 유지 가능. Docker 사용자 기본 경로, 쓰기 불가한 새 기본 경로는 전용 임시 경로 사용 |
| schema_version | 1 유지 |

UTF-8로 저장한다(BOM/CRLF 허용). 숫자/true/false에는 따옴표를 붙이지 않고 JSON 주석은 넣지 않는다. 처음 만든 파일은 key가 비어 있고 생성 false/한도 0이므로 실제 생성하지 않는다. 횟수/크레딧의 유료 기본 수치는 정하지 않았다.

저장 후 **“파일 설정 완료”**라고 알리면 Codex가 값을 출력하지 않고 검사한 뒤 Docker에서 연결을 진행한다. 키나 파일 내용은 채팅으로 보내지 않는다. 일반 파일에는 key가 평문으로 남으므로 NAS 공유 권한으로 접근을 제한한다. 실제 local 파일/편집기 백업은 Git 제외 대상이고, 예제 파일에는 실제 key를 쓰지 않는다.

설정은 세션을 시작할 때 한 번 읽는다. 파일을 고쳐도 이미 실행 중인 엔진의 설정은 바뀌지 않는다. 변경 적용은 기존 작업 상태를 확인해 정상 종료한 다음 새 세션을 연다. Codex가 임의로 바쁜 엔진을 강제 종료하거나 한도를 늘리지 않는다.

## Docker/Codex가 할 일

아래 명령은 사용자가 Windows에서 실행할 필요가 없다. Codex가 실제 Docker 경로를 사용하며 cat/파일 내용 출력 대신 전용 검사만 호출한다.

```bash
python3 app/automation/linux_session.py --config elevenlabs.local.json --check-config
python3 app/automation/linux_session.py --config elevenlabs.local.json
```

첫 번째는 설정과 key 형식/필수 한도를 검사하고 정적인 JSON 결과만 반환한다. 엔진/폴더/HTTP를 만들지 않는다. 두 번째는 Docker-local runtime 부모 경로를 준비하고 기본 debug→release 순서에서 Linux 엔진을 찾은 뒤 기존 전용 stdin 세션으로 전달한다. 엔진 경로는 필요하면 --engine의 절대 경로로 명시한다. key·한도 CLI 옵션과 --config를 섞지 않는다. 상태 root는 기본값 또는 명시한 절대 Linux 경로이며 Windows 드라이브/UNC 경로를 넣지 않는다.

NAS 공유가 POSIX mode를 0777로 보여도 입력 파일을 일률 거절하지 않는다. 파일은 non-symlink regular file·최대 8192 bytes로 읽고, unknown/duplicate 필드·잘못된 타입·NaN·빠진 key/미설정 생성 한도는 거절한다. 경로/JSON parser/키 내용을 오류나 traceback으로 출력하지 않는다. engine runtime의 UID/0700/root·socket 정책은 유지한다. NAS의 SQLite/보안 mode를 지원한다고 가정하지 않으므로 runtime은 Docker 내부 경로에 둔다. Docker container/volume를 삭제하면 해당 경로의 예산/상태도 사라질 수 있어 지속 운용에는 별도 지속 volume 경로를 state_root로 지정한다.

Codex는 현재 세션/계정·한도를 확인하고 명시한 생성 범위에서만 실제 API를 호출한다. 파일 검사가 성공했다는 사실은 서비스 인증/잔액/실제 생성/과금 검증 성공이 아니다. reserved credit은 실제 청구액 상한 보장이 아니며 ElevenLabs key 자체 quota도 별도 설정한다. 생성/편집 후보는 게임에 자동 적용하지 않는다.

## 계약 변경 범위

이전 stdin/숨김 입력/private FD 방식은 유지한다. 사용자 요청으로 **명시적인 --config만** 추가했으며 자동 파일 검색/secret persistence fallback이나 Windows Credential Manager 변경이 아니다. 읽은 key를 DB/다른 파일/로그/argv/env로 복사하지 않는다. 사용자가 직접 작성한 NAS 원본 설정은 삭제/변경하지 않는다. 공통 명령/DB schema/GUI 변경은 없다.

빈 state_root는 기본 ~/.local/state/dk-gamesound가 없고 부모 경로 생성이 권한 오류이면 OS 임시 디렉터리의 dk-gamesound-<effective UID>를 사용한다. 기존 기본 경로나 명시 경로는 자동 변경하지 않는다. 컨테이너/임시 경로 정리 후에도 예산을 유지하려면 쓰기 가능한 private 지속 volume 절대 경로를 state_root로 지정한다. 실제 생성 검증은 [실기 기록](T02_LIVE_RESULTS.md)을 따른다.

기존 전용 temp root가 있으면 기본 home의 쓰기 권한이 회복돼도 그 root를 재사용한다. home/temp 기본 root가 둘 다 있으면 암묵적으로 하나를 선택하지 않고 명시 경로를 요구한다. 새 ledger로 조용히 갈아타 한도가 초기화되는 상황을 막는다.
