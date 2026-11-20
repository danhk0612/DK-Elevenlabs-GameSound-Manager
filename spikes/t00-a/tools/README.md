# T00-A 증거 검사 CLI

Python 3.10 이상과 표준 라이브러리만 사용한다. Linux/Windows에서 AI가 실행할 수 있으며 GUI, 설치, 네트워크, Key가 필요 없다. 제품 CLI/MCP 구현이 아닌 T00-A 검증 보조 도구다. ZIP을 풀거나 실행하지 않고 입력 파일을 변경하지 않는다.

저장소 루트에서 실행한다. Windows에서는 `python3` 대신 준비된 `python` 또는 `py -3`를 사용할 수 있다.

```sh
python3 spikes/t00-a/tools/audit_evidence.py \
  --zip /path/to/downloaded.zip \
  --receipt docs/t00-a-evidence/refresh-20261120/artifact-inspection.json
```

사용자 결과를 받은 뒤에는 `--user-automatic /path/to/user-automatic.json --user-observation /path/to/user-observation.json`을 추가한다. 경로에 공백이 있으면 인용한다. stdout은 JSON 한 개이며 입력 원문·사용자 전체 경로·관찰 내용을 출력하지 않는다. `--help`는 일반 도움말이다.

| 종료 코드 | status | 의미 |
| --- | --- | --- |
| 0 | manual_review_required | 자동 대조 통과, 사람의 증거 검토 필요. Task 완료나 T00-B 승인 아님 |
| 1 | invalid | 입력 구조 오류·SHA/manifest/검사 불일치·명시된 실패. reason으로 원인 구분 |
| 2 | pending | ZIP/CI 대조 통과, 사용자 JSON 또는 관찰 항목 미수령/미확인 |

`schema_version: 1` 출력은 source SHA·run/artifact ID·ZIP/receipt/사용자 JSON의 원시 바이트 SHA256, 24개 manifest 파일·33개 CI 검사, 별도 ACL 범위, 미확인 항목과 사람 검토 목록을 포함한다. `task_completed`·`t00_b_authorized`는 항상 false다. 결과 저장은 호출자가 stdout을 새 파일로 리다이렉트한다. 입력 경로에 덮어쓰지 않는다.

## 입력과 신뢰 경계

`--receipt`는 검토자가 GitHub run head·성공 상태·artifact digest·만료와 대조한 **신뢰하는 영수증**이다. ZIP과 함께 임의로 만든 영수증을 넣어서는 진위를 증명할 수 없다. 이 도구는 GitHub 조회·서명 검증·run 재실행·다운로드 가용성 판단을 하지 않는다. 만료 시에도 보관된 정확한 ZIP의 정적 무결성은 확인할 수 있지만, 새 다운로드를 제공할 때는 GitHub에서 다시 확인한다. 원래 보고서의 hash와 CI 검사 이름은 영수증으로 고정되며 user-automatic에는 그 core 16개 이름 전부와 passed 결과가 필요하다. Server CI 보고서 자체를 사용자 결과로 재사용하면 거절한다.

기존 수집 스크립트의 `user-observation.json` 필드를 유지한다. `executor`, `os`, `notes`를 실제 내용으로 작성하고 `gui_visible`, `gui_common_command`, `close_job_reconnect`, `synthetic_audio_audible`에는 직접 확인한 경우에만 `passed`, 실패면 `failed`, 확인하지 못했으면 `unverified`를 기록한다. 기존 한국어 서술 결과도 자료로 보존할 수 있지만 CLI는 이를 자동 합격으로 해석하지 않고 pending으로 남긴다. `notes`에는 정확한 Windows 에디션/빌드·x64·일반 사용자 실행, GUI 작업 제출·작업 중 닫기·동일 엔진/작업 재연결, 실제 청감 관찰을 적는다. 원래 템플릿 그대로거나 빈 항목은 pending이다.

문자열 passed는 관찰자의 진술일 뿐 도구가 화면/스피커를 관측한 증거가 아니다. OS 버전/권한·제출/닫기 시점·실제 청감·실행자·D01~D06은 사람이 원문과 대조해야 한다. 결과가 모두 있어도 manual_review_required만 반환한다. 파일 hash 계산은 결과를 보관/전달할 때 동일 바이트인지 대조하기 위한 것이며 실행 주체나 실제 청감을 인증하지 않는다. 별도 native pipe ACL 성공을 엔진 IPC 통합/원격 차단 성공으로 확대하지 않는다.

## 검증

```sh
python3 -m unittest discover -s spikes/t00-a/tools/tests -v
```

12개 회귀 검사: 변조 ZIP, 경로 이탈/중복/미포함 파일, archive/user source SHA, 사용자 검사 누락/실패, CI Server 오용, 청감 누락/실패/ended 혼동, 중복 JSON 키, 종료 코드/출력 비밀정보 경계, 입력 보존·한글/공백 경로. 정상 결과 fixture도 Task를 완료하지 않는지 확인한다.

2026-11-20 Linux Python 3.14에서 12개 통과. 보관된 실제 artifact 14618906066 ZIP에 실행해 manifest 24개·CI 33개·별도 ACL 대조 통과, 사용자 두 JSON 미수령으로 exit 2/pending 확인. Windows에서 이 새 Python 도구 자체를 실행하지 않았으며 기존 Windows 런타임/GUI/collector/워크플로는 변경하지 않았다. 검증된 Windows 실행 source는 여전히 `828e74540ae0aa0013d6ec3425389cb0631b0b1a`다.
