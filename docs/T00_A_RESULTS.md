# T00-A 검증 기록

**진행 중. Windows 실제 프로세스·GUI 자동 실행 통과. 사람의 화면/청감, IPC 다른 신원 검증과 D01~D06 결정은 미완료.**

main `4c3537c55084b71feb5c42eb2340005bf922f197`, 브랜치 `task/t00-a-technical-spike`, [draft PR #1](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/pull/1). 아래 결과는 정확한 코드 SHA에 귀속한다. 문서 커밋의 최신 HEAD CI는 PR에서 별도로 식별하며 다른 SHA의 바이너리 동일성을 가정하지 않는다.

## 최신 Windows 증거

코드 SHA **`c7780481a086174fdf476bfc157694bbb8cef4f6`**.
[성공 run 38738233665](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/38738233665), [artifact 11974424135](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/38738233665/artifacts/11974424135).
ZIP SHA256 `d59a1027f400eeab15e77d981a0836324fb323f7d87117bb2fb7aba5da81be65`.
ZIP의 source-sha.txt·각 보고서 SHA·run head 및 전체 SHA256SUMS를 대조하고 4개 EXE의 PE AMD64를 확인했다.

환경: Windows Server 2022 x64 (10.0.20348), Rust/Cargo 1.99.0, Node 24, Python 3.12.10, FFmpeg/ffprobe 9.0.2, WebView2/EdgeDriver 131.0.2903.86. [driver 원시 정보](t00-a-evidence/continuation/webdriver-tools.json), [artifact hash·EXE 크기](t00-a-evidence/continuation/artifact-inspection.json).

| 실행/명령 (spikes/t00-a 기준) | 실제 관찰·판정 | 원시 증거 |
| --- | --- | --- |
| `python scripts/verify.py --bin target/release --sha <SHA> --out <report>` | 16 통과. 수동 기동 경쟁/중복 거절, token·공통 검증, CLI/MCP 동시 ID·모의 예산, 클라이언트 종료 후 작업, kill/정상 종료 후 interrupted·예약 보존, stdout, DB 무결성, 파일/DB 틈, 한글 FFmpeg·원본 hash, 직접 writer 잠금, dummy 자격증명, GUI 프로세스 생존 | [core](t00-a-evidence/continuation/windows-automatic.json) |
| `python scripts/lifecycle.py --bin target/release --sha <SHA> --out <report>` | 11 통과. CLI/MCP 자동 첫 기동 경쟁, PowerShell 5.1 인자, busy 종료 거절·idle 유지, 명시 종료, 완료 뒤 idle 종료/재기동, kill/stale endpoint·자동 재실행 없음, 두 작업공간 독립, invalid idle 거절 | [lifecycle](t00-a-evidence/continuation/windows-lifecycle.json) |
| `python scripts/gui-webdriver.py --bin target/release --sha <SHA> --out <report>` | 6 통과. 실제 Tauri EXE/WebView2에서 status 버튼=CLI, GUI 제출, native WM_CLOSE, 닫힌 뒤 작업 지속, 새 GUI 같은 엔진/결과, 합성 WAV 재생 ended. IPC mock·자동재생 우회 없음 | [GUI](t00-a-evidence/continuation/windows-gui.json), [상태 화면](t00-a-evidence/continuation/gui-status.png), [재생 종료 화면](t00-a-evidence/continuation/gui-playback.png) |
| `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/pipe-acl.ps1 -SourceSha <SHA> -OutFile <report>` | 부분 조사. 현재 사용자 접속 통과. 익명 허용 대조군도 WinError 5라 다른 신원 ACL 효력은 판정 불가. 33개 통과에 포함하지 않음 | [pipe ACL](t00-a-evidence/continuation/windows-pipe-acl.json) |
| 사용자 데스크톱 실행 | 결과 미수령. 자동 GUI 스크린샷에서 한글/상태/재생 종료는 검토했으나 사용자 화면·스피커 청감 증거가 아님 | [정확한 SHA 수집 절차](T00_A_WINDOWS_CHECK.md) |

합계는 core 16 + lifecycle 11 + GUI 6 = **33개 자동 검사**다. CI build 성공, 실제 EXE 실행, GUI 자동 조작, 사람의 청감은 각각 다른 증거다. WAV currentTime=duration=1, ended=true, error=null은 디코딩/재생 이벤트만 입증한다. MP3/OGG·실제 음원 청감은 검증하지 않았다.

## 발견·수정과 한계

Windows에서 엔진이 부모의 상속 가능한 stdio 핸들을 붙잡아 MCP 종료 후 stdout EOF가 지연됐다. 자식의 null stdio 설정만으로 해결되지 않았다. 안정 Win32 API로 부모 표준 핸들의 상속 플래그를 제거해 백그라운드 엔진을 분리했다. 수정 전 15초 수신 timeout, 수정 뒤 30초 작업 중 MCP EOF 약 0.015초를 관찰했다. PowerShell 5.1의 native 인자 JSON 따옴표도 helper에서 보존해 실제 호스트로 재검사했다.

`T00_SPIKE_AUTOSTART=1`과 fixture idle 환경변수는 opt-in이다. 기본 수동 모드와 구분한다. 작업공간마다 엔진/가짜 예산이 따로 있어 동일 계정 전체 예산·사용자 전체 단일 엔진의 증거가 아니다. 전송된 명령은 무조건 재전송하지 않는다. 엔진 TCP/token은 제품 IPC로 채택하지 않았다.

별도 pipe 후보는 owner SID DACL과 remote reject 플래그를 시험했다. 유효한 다른 신원 positive control을 확보하지 못해 계정 간·원격 차단은 미검증이다. 엔진 통합도 없다. OS 계정/정책은 변경하지 않았다.

파일/DB fault injection은 temp 기록 뒤와 rename 뒤 강제 종료를 재현한다. 재기동 시 interrupted와 예약 보존·잔여 파일을 확인했으며 **자동 파일/DB 대조 복구는 구현하지 않았다**. 모의 비용/시간은 제품 정책이 아니며 실제 API 비용·응답 유실은 시험하지 않았다.

이전 `80993133331c134e8a6bea12c7a9c8e0f7493a4e`의 Linux/core Windows 원시 결과는 [기존 evidence](t00-a-evidence/)에 보존한다. Linux는 Windows 증거가 아니다. 초기 Windows icon·socket 실패와 후속 lifecycle timeout/익명 대조군 실패는 해당 run의 실패이며 최신 성공으로 소급하지 않는다.

## 재개 조건

1. [수집 절차](T00_A_WINDOWS_CHECK.md)로 정확한 SHA의 Windows 10/11 사용자 화면·청감 결과를 PR에 받는다. 빈 observation은 미검증이다. 소스가 바뀌면 해당 수정 SHA로 재실행한다.
2. 실제 다른 신원에서 허용 대조군 성공/보호 pipe 거절을 비교 가능한 실행 환경을 확보한다. CI green이나 anonymous access denied만으로 ACL 통과를 선언하지 않는다. 엔진 통합/사용자 전체 소유권 정책도 별도 검토한다.
3. 파일/DB 대조 복구와 계정 전체 소유권·busy/idle 제품 정책을 결정 자료로 검토한다. fixture 구현을 제품 완료로 해석하지 않는다.
4. [D01~D06 및 조건부 T00-B 계약](T00_A_DECISIONS.md)을 사용자와 검토한다. T00-A 실기·설계 게이트와 사용자 결정 전 T00-B 금지.

artifact 보존 기간은 14일이다. 합성 fixture·EXE·스크립트·SHA/도구 정보·hash만 포함하며 FFmpeg 바이너리·임시 DB·endpoint token·실제 음원·Key는 포함하지 않는다. 배포/라이선스는 미확정이다.
