# T04 로컬 사운드 작업 정의 클라이언트 — v0.7.0

선택형 Python 3.11+ 클라이언트 `app/automation/sound_pipeline.py`가 JSON 작업 정의 하나를 CLI 또는 MCP로 실행한다. 게임 코드나 음원 생성 모델에 종속하지 않으며 현재 게임 파일/DB를 수정하지 않는다. 핵심 엔진/CLI/MCP 실행에는 Python이 필요 없다. 별도 Python 패키지는 사용하지 않는다. GUI를 추가하지 않는다.

## 실행

`app/automation/local-import.recipe.json` 또는 `edit-analyze.recipe.json`을 복사하고 로컬 workspace/음원/도구 경로와 프로젝트·버전·작업·검토 ID를 지정한다. 모든 파일 경로는 존재하는 절대 경로여야 하고 workspace는 미리 만든 로컬 디렉터리여야 한다. 실행 파일/설정 폴더와 겹치지 않는 workspace를 사용한다. 예제의 수치·형식은 호출 예시이며 D08 제품 기본값/품질 합격 기준이 아니다. 원본의 출처·이용 조건은 별도로 기록한다.

```powershell
python automation/sound_pipeline.py C:\GameAudio\my-recipe.json --cli C:\Tools\DKGameSound\dk-sound-cli.exe --validate-only
python automation/sound_pipeline.py C:\GameAudio\my-recipe.json --cli C:\Tools\DKGameSound\dk-sound-cli.exe
python automation/sound_pipeline.py C:\GameAudio\my-recipe.json --cli C:\Tools\DKGameSound\dk-sound-cli.exe --mcp C:\Tools\DKGameSound\dk-sound-mcp.exe
```

CLI·MCP·engine는 같은 버전 세트로 준비한다. v0.7 CLI에 새 `validate` 하위 명령을 추가했다.

```powershell
Get-Content -Raw -Encoding utf8 request.json | .\dk-sound-cli.exe validate
```

validate는 core의 공통 명령/필드/타입/범위만 오프라인으로 검사한다. 엔진 자동 기동·DB 생성·음원/도구 hash·비용·현재 버전 상태 조회는 수행하지 않는다. 성공의 scope는 syntax_and_fields_only다. 오류도 기존 공통 JSON/exit 1을 사용한다. 공통 명령/MCP 도구 수는 여전히 47/46이다. validate는 CLI 전용 로컬 편의 명령이다.

클라이언트는 파일 64 KiB 이하·schema_version=1·허용 필드/경로/서로 다른 버전/작업 ID를 검사하고, 실행할 모든 요청과 polling 요청을 이 validate로 검증한 뒤 엔진 요청을 시작한다. --validate-only는 여기서 성공 JSON을 반환하고 끝난다. 부분적으로 등록한 뒤 뒤쪽 입력 오류를 발견하는 일을 줄이지만 실제 파일·DB·도구 상태 오류까지 사전에 보장하지 않는다. MCP를 선택해도 preflight에는 CLI가 필요하다.

## 작업 정의와 흐름

| 필드 | 내용 |
| --- | --- |
| schema_version | 1 |
| project | project_id, name, workspace |
| source | asset_id, version_id, name, kind, source_path, source_note |
| candidate | import: version_id, source_path, source_note / edit: version_id, operation_id, start_ms, end_ms, gain_db, fade_in_ms, fade_out_ms, output_format, sample_rate, channels |
| tools | 편집/검사 시 필수: ffmpeg_path, ffprobe_path. 사용하지 않으면 생략 |
| analysis | 선택: operation_id, waveform_bins, silence_db, silence_min_ms, clip_level; loop_start_ms/loop_end_ms 선택 쌍 |
| review | review_id, summary; notes/기존 requirement_id 선택 |

project.create → asset.import(source) → 필요 시 audio.tools.set → candidate import 또는 audio.submit(edit) → 선택적 audio.submit(analyze) → asset.file → review.submit → review.check → asset.history 순서다. 편집·검사는 audio.get으로 succeeded를 확인한 후 다음 단계로 진행한다. 제출 시 실제 확인한 후보 SHA-256과 선택 검사 ID를 자동 연결한다. source/candidate는 같은 asset에 속하고 source를 부모로 사용한다. requirement를 생성/변경/연결하는 동작은 없다.

audio.tools.set은 PC의 공유 도구 설정을 지정한 경로/hash/version으로 갱신한다. 이미 대기한 작업의 snapshot은 변경하지 않는다. 작업 정의에 없는 출력/검사 기본값을 보충하지 않는다. 최초 source가 기존 candidate여도 부모로 사용할 수 있으며, 핵심 버전/해시 규칙을 우회하지 않는다.

성공 stdout은 JSON 한 개이며 검토 snapshot·현재 일치 확인·첫 이력 페이지/next_after·approval=not_granted를 담는다. 이력을 전부 수집한 것으로 주장하지 않는다. 단계 진행은 stderr JSON으로 내보내고 API Key/요청 본문은 출력하지 않는다. 실패 exit 1은 error.code와 stage/project_id/review_id/가장 최근 operation_id를 제공한다. Ctrl+C는 exit 130과 work_may_continue를 반환한다. --help/인자 파싱 오류는 일반 Python CLI 출력을 사용한다.

## 중단과 재실행

--wait-seconds 기본 300(1~86400), --poll-seconds 기본 0.5(0.1~60)는 클라이언트 대기 설정이다. 작업 완료를 기다리는 제한과 별개로 단일 전송의 timeout은 --request-seconds 기본 300초(1~300)이며 실행 중 core 명령의 한도도 적용된다. timeout·EOF·프로토콜 오류가 발생한 요청을 자동 반복하지 않는다. 프로그램/전송 종료는 서버 작업 취소가 아니다. 실패한 단계의 결과를 audio.get/asset.versions/review.get으로 대조한다.

같은 JSON/같은 ID로 다시 실행하면 project/import/audio/review의 기존 idempotency 계약에 따라 기록을 대조한다. 중간 단계부터 건너뛰는 로컬 checkpoint는 사용하지 않는다. 원본 경로의 내용 변경, ID/입력 충돌, 프로젝트 경계 위반은 core가 거절한다. 같은 ID의 작업이 failed/cancelled/interrupted/output_ready이면 멈추고 attention 오류를 반환한다. 새 ID로 자동 재생성하거나 audio.recover를 자동 호출하지 않는다. 복구 가능한 받은 파일은 기존 명시적 복구 계약을 따른다.

withdrawn 검토를 재호출해도 다시 열지 않으며 클라이언트는 review_not_pending으로 멈춘다. review.check에서 현재 자료가 불일치해도 성공 처리하지 않는다. 자동 롤백/음원 삭제는 없고 이미 완료한 단계는 그대로 남는다. 변경한 입력으로 새 후보를 만들려면 새 버전/작업/검토 ID를 명시한다. 도구 설정 변경은 명시적으로 실행되는 공유 설정 변경이다.

이 클라이언트에는 account/budget/job 생성·유료 API·승인/적용·게임 출력 경로·shell 명령 실행 필드가 없다. 핵심 매니저의 다른 유료 도구를 삭제하거나 차단하는 기능은 아니다. summary/notes와 성공한 수치 검사는 사람이 승인한 증거가 아니다.

## 전송과 검증 범위

CLI는 request stdin/JSON stdout과 exit code를 대조한다. MCP는 stdio initialize(2025-11-25) → initialized → tools/list 필수 도구 확인 → tools/call을 사용하고 ID·공통 structuredContent/isError를 대조한다. 한 번에 한 요청만 보낸다. 프레임 64 KiB·유한 읽기/쓰기 timeout·EOF·자식 종료를 처리하며 shell을 사용하지 않는다. 종료 시 CLI/MCP 자식만 정리하고 서버 엔진은 임의로 종료하지 않는다.

Linux에서 Python 클라이언트 회귀 8개, native CLI offline validate/preflight와 MCP handshake/discovery/불량 hash 거절을 확인했다. Rust 기존 39개 중 일반 32개/외부 도구 ignore 7개를 구분한다. Rust native build·Windows x64 타입 검사와 UI build는 별도 확인한다. v0.6 SHA `2c4faed4983dfb0e53e6e17f7953d3724cd792d5`의 Windows [run 44746167182](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/44746167182)은 성공했다.

새 CI에는 Python 회귀와 `ci_transport_smoke.py`를 추가했다. 후자는 격리된 Windows Actions에서만 실제 엔진을 기동하여 합성 WAV 원본/외부 후보 등록→검토 제출, CLI/MCP 재호출의 동일 기록, 클라이언트 종료 후 재연결, 원본 해시 보존과 변경 원본 재등록 거절을 검사한다. FFmpeg를 설치하지 않으므로 이 CI가 편집/분석을 실제 수행한 것은 아니다. 같은 스크립트의 --offline 모드는 엔진 없는 CLI preflight/MCP 계약만 확인한다.

실제 지원 AI 제품의 MCP 클라이언트 연결, 사용자 Windows 화면/청감·실제 Key/과금은 여전히 미확인이다. 이 선택형 자체 클라이언트를 ChatGPT 등 제품 연결 성공으로 보고하지 않는다. T04 전체 수용·T05/D07·병합/공개 배포는 별도다.

초기 v0.7 `97d13df9c402e602d888af6f888519e8992aa8d5`의 Windows run 44749313675는 native/UI build·Rust 29개 통과/외부 7개 ignore·Python 8개 통과 후 첫 실제 엔진 recipe에서 60초 timeout으로 실패했다. 실행 단계 진단을 위해 요청별 대기 제한과 실패 시 진행 로그를 추가했다. 이 실행을 실제 IPC 성공으로 기록하지 않는다.
