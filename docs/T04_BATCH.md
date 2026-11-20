# T04 여러 사운드 작업 정의 — v0.8.0

선택형 Python 3.11+ `automation/sound_batch.py`로 [기존 단일 작업 정의](T04_AUTOMATION.md)를 여러 개 묶어 실행한다. batch는 클라이언트 기능이며 DB schema/공통 명령 47개·MCP 46개/GUI를 바꾸지 않는다. 유료 생성·사람 승인·게임 적용을 호출하지 않는다.

## 사용

`automation/local-batch.json`은 같은 원본에서 외부 후보 두 개를 등록하는 예제다. 프로젝트/workspace/음원 경로·버전/검토 ID를 수정한다. 단일 recipe의 편집·검사 형식도 그대로 사용할 수 있다. 도구/출력/검사 조건을 자동으로 보충하지 않는다. 예제 수치는 제품 품질 기준이 아니다.

```powershell
python automation/sound_batch.py C:\GameAudio\batch.json --cli C:\Tools\DKGameSound\dk-sound-cli.exe --validate-only
python automation/sound_batch.py C:\GameAudio\batch.json --cli C:\Tools\DKGameSound\dk-sound-cli.exe
python automation/sound_batch.py C:\GameAudio\batch.json --cli C:\Tools\DKGameSound\dk-sound-cli.exe --mcp C:\Tools\DKGameSound\dk-sound-mcp.exe --jsonl
python automation/sound_batch.py C:\GameAudio\batch.json --cli C:\Tools\DKGameSound\dk-sound-cli.exe --only hit-take-2
```

모든 경로/필드·상태/해시·원본 보존은 단일 작업 정의/core 규칙을 따른다. --request-seconds/--wait-seconds/--poll-seconds도 단일 클라이언트와 같은 범위다. 작업 중 클라이언트를 닫아도 엔진 작업을 취소하지 않는다.

## 입력과 사전 검사

```json
{"schema_version":1,"batch_id":"my-batch","items":[{"item_id":"hit-take-1","recipe":{"schema_version":1,"project":{},"source":{},"candidate":{},"review":{}}}]}
```

위 축약 JSON의 빈 객체는 실제 입력이 아니다. 전체 예제와 단일 recipe의 필수 필드를 사용한다. batch_id/item_id는 1~64자의 소문자 영숫자·하이픈·밑줄이며 클라이언트 식별자다. 서버의 새 namespace/DB 기록/승인 식별자가 아니다. 최대 32개, manifest 파일 최대 1 MiB, 각 recipe의 정규화 JSON 최대 64 KiB다. UTF-8/BOM을 읽고 중복 JSON key·NaN/Infinity·미지원 schema/알 수 없는 필드를 거절한다. manifest_sha256은 정렬한 JSON의 해시이며 음원 내용 해시나 승인 증거가 아니다.

실행 전 **전체 batch**를 compile하고 모든 core 요청을 CLI offline validate로 검사한다. 같은 요청은 한 번만 검증한다. 마지막 항목이 잘못됐어도 앞 항목을 먼저 등록하지 않는다. preflight 실패는 item_id/item_index 또는 core 검증 대상 item_id와 error.code를 반환한다. --validate-only는 엔진을 기동하거나 DB를 생성하지 않는다. 파일 존재 확인과 명령 문법 검사이며 실제 hash/DB 상태·도구 실행 성공 보장은 아니다.

다음 충돌을 실행 전에 거절한다.

- 같은 item_id, 같은 프로젝트의 서로 다른 name/workspace
- 같은 프로젝트/asset의 서로 다른 source 정의(원본 버전·출처·name/kind/경로 포함)
- 같은 프로젝트/asset/version을 중복 candidate로 예약
- 프로젝트가 달라도 같은 operation_id 또는 review_id(해당 core catalog에서 전역 ID)
- 다른 항목이 만드는 candidate를 source 버전으로 삼는 항목 간 의존

같은 원본의 **완전히 동일한 source 정의**를 공유하고 서로 다른 candidate/검토 ID를 만드는 것은 허용한다. 서로 다른 도구 설정도 순차 적용할 수 있으며 PC 공유 audio.tools 설정을 명시적으로 갱신한다. 기존 대기 작업의 tool snapshot은 바뀌지 않는다. batch 바깥의 기존 데이터 충돌은 실제 core 요청에서 거절할 수 있다.

## 실행·실패·재실행

항목 순서대로 실행하며 각 항목에 독립 CLI/MCP 클라이언트를 사용한다. MCP 세션 오류가 난 인스턴스를 다음 항목에 재사용하지 않는다. 편집/검사 대기 완료 후 후보 hash와 검사 근거를 검토 제출에 연결한다. 기본은 첫 항목 실패에서 중단한다. 이전 완료 작업은 남고 이후 선택된 항목은 not_run이다. 전체 batch를 하나의 원자적 transaction으로 취급하거나 자동 롤백/삭제하지 않는다.

--keep-going은 클라이언트의 명시적 CONTINUABLE 목록에 있는 결과 확정 오류에만 계속한다. 예: version_id_conflict, 원본/후보 해시 불일치, 철회한 검토, requirement 불일치, confirmed failed/cancelled/interrupted/output_ready 오디오 상태다. 수신 완료 output_ready의 자동 복구는 하지 않는다. 한 항목 실패라도 전체 exit는 1이다.

전송 timeout/EOF·응답 ID/프로토콜 오류·wait deadline·엔진/저장소 문제·미지의 오류는 --keep-going이어도 중단한다. 결과가 불확실한 요청을 재제출하거나 새 작업 ID를 만들지 않는다. Ctrl+C는 현재 항목 interrupted/exit 130, 이후 항목 not_run과 현재 stage/operation_id를 반환한다. 파이프 I/O를 정리하는 것은 엔진 작업 취소가 아니다.

--only ITEM_ID를 반복해 선택한 항목만 다시 실행할 수 있다. **선택하지 않은 항목도 전체 입력/충돌 검사를 통과해야 한다.** 미선택 상태는 not_selected다. 전체 입력을 바꾸지 않고 실패한 항목을 골라 기존 ID로 대조할 때 사용한다. 같은 ID 재호출의 core idempotency를 따르며 완료 항목을 신뢰하고 건너뛰는 로컬 checkpoint는 없다. 기존 성공 기록도 재호출하면 파일/검토 상태를 다시 확인하므로 이후 변조나 철회는 성공으로 숨기지 않는다. 변경된 입력에는 새 버전/작업/검토 ID가 필요하다.

## 구조화 결과

기본 stdout은 summary JSON 한 개다. items에는 항목별 item_id/project_id/asset_id/version_id/review_id, status, 마지막 stage, 실패 시 operation_id/error.code가 있다. 성공에는 candidate_sha256/review_state/matches와 approval=not_granted만 요약한다. 임의 사용자 메모·음원 경로·큰 검사 waveform을 반복 출력하지 않는다. 전체 snapshot/검사는 review.get/audio.get을 사용한다.

status는 succeeded/failed/interrupted/not_run/not_selected다. counts는 각 상태 개수다. 선택한 항목이 모두 성공하면 ok=true/exit 0, 실패가 있으면 ok=false/batch_incomplete/exit 1이다. 사전 입력 오류 역시 exit 1이지만 실행 항목 결과는 없고 context.stage=preflight다. CLI 사용법/잘못된 옵션은 일반 argparse 출력이다.

--jsonl은 각 항목 결과를 `type=item` 한 줄로 즉시 출력하고, 마지막 `type=summary`에는 전체 counts와 manifest 식별자만 넣어 items를 중복하지 않는다. stage 진행은 item_id와 함께 stderr JSON으로 출력한다. 출력 파일은 프로그램이 생성/덮어쓰지 않는다. 필요하면 호출하는 AI/셸이 stdout을 저장한다. 강제 종료된 실행에서 마지막 summary가 없으면 완료로 간주하지 않고, 이미 받은 item 기록과 core 상태를 대조한다. JSONL은 승인/복구 journal이 아니다.

## 확인과 한계

Linux에서 단일 클라이언트 9개 + batch 9개 회귀, 실제 native CLI의 offline batch preflight/뒤 항목 오류/ID 충돌 거절을 확인했다. 같은 원본 공유·ID/입력 충돌·선택 실행·단계 식별자·fail-stop/keep-going·불확실 결과 중단·Ctrl+C/자식 정리를 검사한다. 기존 Rust/core/단일 클라이언트 IPC 계약은 유지한다.

Windows CI에는 실제 engine에서 shared source·특정 항목 실행·CLI/MCP 재호출 일치·JSONL·철회 기록에 대한 기본 중단/keep-going·원본 보존을 실행하는 ci_batch_smoke.py를 추가했다. --offline은 실제 engine 없이 preflight만 확인한다. Windows 실행 결과와 개발 artifact의 정확한 SHA는 CURRENT_STATE.md를 따른다. FFmpeg를 설치하지 않는 CI이므로 실제 편집/분석까지 검증한 것으로 확대하지 않는다. 실제 지원 AI 제품 연결·Windows 11 사용자 청감·유료 API·T05 승인은 별도다.
