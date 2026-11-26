# T04 보강 — 비동기 로컬 오디오 v0.5.0

기존 audio.analyze/edit 동기 명령과 T03 편집/검사 조건은 유지한다. 긴 작업의 기본 자동화 경로로 **audio.submit → audio.get/list → audio.cancel**을 추가했다. 총 41개 공통 명령 / MCP 40개 도구다. 도구 schema의 action별 oneOf가 필수/허용 입력을 구분한다. 주관 검토 제출·승인·게임 적용은 이번 범위가 아니다.

## 제출과 조회

T03 동기 요청의 op를 audio.submit으로 바꾸고 action=edit 또는 analyze를 넣는다. 나머지 필드는 해당 동기 명령과 동일하다. 내부에는 기존 audio.edit/analyze로 정규화한 불변 snapshot을 저장한다. 같은 operation_id/같은 정규화 입력은 기존 상태를 돌려주며 설정이 해제되어도 이미 기록된 요청을 다시 실행하지 않는다. 다른 프로젝트/입력은 audio_operation_id_conflict다.

```json
{"protocol_version":1,"op":"audio.submit","action":"edit","project_id":"demo","asset_id":"click","parent_version_id":"original","version_id":"take-002","operation_id":"edit-click-002","start_ms":0,"end_ms":450,"gain_db":-3,"fade_in_ms":5,"fade_out_ms":25,"output_format":"wav_pcm_s16le","sample_rate":48000,"channels":1}
```
```json
{"protocol_version":1,"op":"audio.submit","action":"analyze","project_id":"demo","asset_id":"click","version_id":"take-002","operation_id":"inspect-click-002","waveform_bins":128,"silence_db":-60,"silence_min_ms":20,"clip_level":0.999}
```

분석 대상/부모는 ready가 필요하므로 편집의 succeeded를 확인한 뒤 분석을 제출한다.

각 JSON을 UTF-8 파일로 저장하고 `dk-sound-cli request < request.json`로 실행한다. PowerShell은 `Get-Content -Raw -Encoding utf8 request.json | .\dk-sound-cli.exe request`를 사용할 수 있다. MCP는 sound_audio_submit/get/list/cancel과 같은 schema를 사용한다.

```json
{"protocol_version":1,"op":"audio.get","project_id":"demo","operation_id":"edit-click-002"}
```
```json
{"protocol_version":1,"op":"audio.list","project_id":"demo","limit":10}
```
```json
{"protocol_version":1,"op":"audio.cancel","project_id":"demo","operation_id":"edit-click-002"}
```

제출은 도구를 실행하거나 큰 음원/도구 파일을 hash하지 않는다. ready 부모/대상과 설정 metadata를 확인하고 catalog transaction에서 대상 버전과 자리를 예약한 뒤 queued 상태를 반환한다. 실제 source hash·도구 실행 파일 검사는 실행 직전에 한다. 대기열은 엔진 전체 최대 32개이며 가득 차면 audio_queue_full이다. 취소된 작업 ID와 대상 버전도 이력으로 남기고 재사용하지 않는다.

제출 당시 도구 경로/hash/version snapshot과 입력 버전 hash를 고정한다. 나중에 설정을 바꾸거나 해제해도 기존 작업이 다른 도구로 바뀌지 않는다. 고정된 실행 파일이 변경/삭제되면 작업이 실패한다. audio.get의 tool_snapshot은 hash/version을 표시한다. 실행 전에는 queued, 실행 시 processing과 단계가 보이고, 종료 시 succeeded/failed/cancelled/interrupted 또는 output_ready다. 단계는 checking_tools/checking_input/probing/decoding/measuring/encoding/checking_output/publishing/completed 등이며 가짜 진행률을 만들지 않는다.

## 취소 경계

queued는 즉시 cancelled로 바꾼다. processing은 cancel_requested=true를 기록하고 실행기가 확인한 뒤 cancelled로 확정한다. 요청 응답만 보고 취소 완료로 취급하지 말고 audio.get으로 최종 상태를 확인한다. 같은 취소 요청은 반복할 수 있다. interrupted도 명시적으로 취소할 수 있다.

실행기는 외부 프로세스 대기 중 취소를 확인하여 자식 프로세스에 kill/wait를 수행한다. PCM 통계 루프와 단계 사이에도 확인한다. 큰 파일 hash·파일시스템 I/O는 중간에 끊지 않으므로 취소 확정까지 시간이 걸릴 수 있다. 실제 Windows 프로세스 종료/파일 핸들 수명은 Linux 실행 증거와 구분한다.

편집 결과를 output_ready로 전환하는 DB 갱신은 cancel_requested=0인 processing에만 허용한다. 취소가 먼저 기록되면 후보 등록을 시작하지 않는다. output_ready가 먼저 확정되면 등록 복구 대상으로 보존하고 audio_not_cancellable로 거절한다. 이미 등록한 후보/원본을 삭제하지 않는다. 분석 성공 기록도 취소 플래그와 원자적으로 대조한다. 성공/실패/수신 완료 등 취소 불가 상태를 취소 성공으로 보고하지 않는다.

중간 파일은 자동 채택하지 않는다. 취소/실패의 spool/PCM을 다른 요청에 재사용하거나 기존 파일을 덮어쓰지 않는다. 받은 파일 등록 복구는 기존 audio.recover 계약을 따른다.

## 엔진과 동시성

FFmpeg/원격 생성/최종 후보 파일 등록은 기존 단일 실행 스레드에서 직렬 처리한다. 동기 요청은 이전 동작을 유지하고 async 제출은 별도 짧은 DB 연결로 예약한다. audio.get/list/cancel도 별도 연결로 처리하므로 긴 FFmpeg 또는 원격 생성 중에도 사용할 수 있다. 이 연결은 catalog를 새로 만들거나 migration/재시작 복구를 실행하지 않는다. 엔진은 최초 저장소 초기화 시도 후 pipe 요청 처리를 시작해 새 조회가 migration과 경합하지 않게 한다. SQLite transaction은 예약·취소/상태 전환에만 사용하며 외부 프로세스 동안 열어 두지 않는다.

로컬 편집과 원격 생성은 같은 대상 project/asset/version을 동시에 예약하지 못하도록 catalog transaction에서 대조한다. 실행 직전/최종 import에서도 원본·버전 충돌을 대조한다. engine worker의 시간 경계에서 큐를 확인하고 local/remote 대기열이 모두 있으면 번갈아 실행한다. 이미 실행 중인 작업은 선점하지 않는다. 기존 동기 명령/일반 자료 명령은 계속 실행 스레드 뒤에서 대기할 수 있다. 모든 서비스 명령이 병렬이 된 것은 아니다.

status에 audio_queued/audio_running과 전체 queued/running을 표시한다. 짧은 제출/취소 처리와 실제 큐/실행 중 정상 shutdown 및 idle 종료를 막는다. fast 응답의 오래된 개수로 새 큐를 지우지 않도록 상태 반영 시 다시 조회한다. GUI/CLI/MCP 종료는 작업 취소가 아니다.

재기동은 자동 재실행을 하지 않는다. queued/processing은 interrupted, 취소 플래그가 이미 있으면 cancelled로 정리한다. 기존 output_ready는 유지하여 명시적으로 복구한다. 전체 데이터/전원 장애 복구를 보장하는 것은 아니다.

## 최소 UI와 업그레이드

기존 중립색 화면에 작업 ID의 상태/단계 조회와 취소 요청 버튼만 추가했다. 조회는 선택한 프로젝트에 한정하며 파일 버전을 선택하지 않아도 가능하다. 취소는 별도 Tauri wrapper에서 같은 audio.cancel 명령만 호출한다. Key·승인 입력이나 게임 파일 쓰기 경로를 추가하지 않는다. 취소 요청됨과 최종 cancelled를 구분해서 표시한다.

catalog schema 3→4 transaction migration으로 cancel_requested/phase/tools_snapshot을 추가한다. 프로젝트 DB는 schema 1이다. 이전 완료 보고서/후보/원본은 유지한다. migration/중단 정리는 소유 엔진 초기화에서만 실행하며 조회 연결은 수행하지 않는다. 업데이트 전 정상 엔진 종료와 일관된 백업을 사용한다. 이전 v0.4 바이너리는 schema 4를 읽지 못하므로 새 자료를 삭제하는 자동 다운그레이드는 없다.

## 검증 경계

Linux musl Rust 1.100.0·bundled SQLite와 T03의 FFmpeg/ffprobe 7.0.2 static을 사용하며 합성 자료만 처리했다. 기존 T01~T03 회귀에 큐 제출/ID 재사용·프로젝트 격리·32개 제한·중단 무재생, 두 연결의 대상 예약 및 원격 생성과 충돌, 실행 중 조회/새 제출/취소, 도구 snapshot 고정, 취소/후보 등록 경계, 실제 대기 자식 프로세스 kill/wait, 큐의 shutdown/idle 보호를 추가했다.

외부 도구가 필요한 테스트는 다음과 같이 포함한다. Windows CI 기본 테스트에서 FFmpeg 통합까지 실행한 것으로 확대하지 않는다.

```sh
DK_TEST_FFMPEG=/tmp/dk-audio-tools/ffmpeg \
DK_TEST_FFPROBE=/tmp/dk-audio-tools/ffprobe \
cargo test --locked --manifest-path app/Cargo.toml --lib -- --include-ignored
```

Windows native IPC/실제 GUI 취소·FFmpeg 자식 종료와 사용자 청감·실제 유료 API는 미확인이다. 별도 실제 AI 클라이언트와의 연결 검증도 native stdio smoke와 구분한다. 수동 실기 대기로 개발을 멈추지 않는다. 다음은 검토 대기 제출·후보 이력 조회 등 T04 자동화이며 T05 사람 승인/게임 적용은 별도 설계 범위다.
