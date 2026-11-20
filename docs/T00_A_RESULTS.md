# T00-A 검증 기록

**진행 중. Windows 실제 프로세스·GUI 자동 실행 통과. 별도 타 계정 pipe ACL 검사도 통과. 사람의 화면/청감, 제품 IPC 통합은 미완료. D01~D06은 2026-11-20 사용자 위임으로 채택했으며 사용자 실기 게이트는 유지한다.**

main `4c3537c55084b71feb5c42eb2340005bf922f197`, 브랜치 `task/t00-a-technical-spike`, [draft PR #1](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/pull/1). 아래 결과는 정확한 코드 SHA에 귀속한다. 문서 커밋의 최신 HEAD CI는 PR에서 별도로 식별하며 다른 SHA의 바이너리 동일성을 가정하지 않는다.

## 현재 실기용 ZIP 검증 — 2026-11-20 KST

[run 44713166683](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/44713166683)의 head와 source SHA는 **`828e74540ae0aa0013d6ec3425389cb0631b0b1a`**다. [artifact 14618906066](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/44713166683/artifacts/14618906066)는 4,271,017바이트, ZIP SHA256 `368d9d873265edade0fa7203d947353e032795144d26e0bac2c313cc98b8a631`, 만료 예정은 **2026-12-04 18:49:13 KST**다. GitHub 메타데이터 digest와 실제 다운로드 해시를 대조했다.

| 대조 | 확인 결과 / 증거 |
| --- | --- |
| ZIP 내용 | CRC 통과, 경로 이탈/중복 없음, `source-sha.txt`와 run head/이름 일치 |
| manifest | 24개 hash 모두 일치. manifest 자체를 제외한 ZIP 파일 전체 포함 |
| core/lifecycle/GUI | 원시 JSON의 source SHA가 모두 위 SHA. 16+11+6=33개 자동 검사 성공. [core](t00-a-evidence/refresh-20261120/windows-automatic.json), [lifecycle](t00-a-evidence/refresh-20261120/windows-lifecycle.json), [GUI](t00-a-evidence/refresh-20261120/windows-gui.json) |
| 별도 pipe ACL | owner 연결=0, 다른 SID 허용 대조군=0, owner 전용 거절=5, 원래 SID 복귀·임시 계정 삭제 성공. [원시 보고서](t00-a-evidence/refresh-20261120/windows-pipe-acl.json). 제품 엔진 IPC/원격 차단 성공 아님 |
| 실행 파일 | CLI·엔진·MCP·GUI 4개 PE AMD64. 각 hash와 크기는 [영수증](t00-a-evidence/refresh-20261120/artifact-inspection.json) |
| 포장된 소스 | 스크립트 7개·수집 PS1·README, 총 9개를 해당 Git SHA의 원문과 UTF-8 BOM/CRLF 정규화 후 대조 |
| GUI 캡처 | [상태](t00-a-evidence/refresh-20261120/gui-status.png), [재생 종료](t00-a-evidence/refresh-20261120/gui-playback.png)를 검토. 1초 WAV ended는 사람의 청감 아님 |
| 사용자 실기·결정 | PR Conversation/리뷰와 현재 사용자 메시지에 사용자 실기 결과 없음. 사용자 JSON 두 개의 source SHA/파일 hash는 **미수령**. 기술 선택은 후속 사용자 위임으로 별도 기록 |

검사 환경은 Linux Python 3.14의 정적 ZIP 검사다. Windows EXE를 새로 실행한 것이 아니며 실행 결과는 위 정확한 SHA의 Windows Server 2022 CI에 귀속한다. `f8aa463..828e745` 비교에서 실행 Rust/UI/검사 스크립트/lockfile/워크플로 변경은 없고 포장 README 설명만 달랐다. 과거 EXE/ZIP hash를 새 파일에 재사용하지 않았다. 유효한 성공 artifact와 변경 없는 실행 코드가 있으므로 이 문서·증거 보존을 이유로 CI를 반복하지 않는다.

수령할 사용자 증거는 [Windows 절차](T00_A_WINDOWS_CHECK.md)의 ZIP/run head, source-sha, manifest, `user-automatic.json`, 직접 작성한 `user-observation.json`이다. 각 JSON의 source SHA와 파일 SHA256을 함께 기록하고 빈 관찰은 미검증으로 남긴다. D01~D06은 후속 사용자의 판단 위임으로 채택했고 [결정 기록](t00-a-evidence/refresh-20261120/delegated-decisions.json)에 보존했다. 이는 사용자 실기 증거가 아니다. T00-A 완료·T00-B·병합·배포는 하지 않았다.

## 과거 Windows 기준선 — 2026-10-16 KST

검증 코드 SHA **`f8aa463bfd02fa84a10c39f32b353cf83d2c7872`**. [성공 run 38809024678](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/38809024678), [artifact 12007313423](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/38809024678/artifacts/12007313423).
ZIP SHA256 `e7895dc45ebea7c7271e64e6dd4a10061a0fb443b871135aa70c1c80416b45a3`.
artifact source-sha와 보고서 4개의 SHA, 전체 manifest 24개 hash와 4개 EXE PE AMD64를 대조했다. Rust 제품 코드는 변경하지 않았으며 pipe 검사 스크립트와 해당 CI 호출을 수정했다.

| 검사 | 실제 결과 | 원시 증거 |
| --- | --- | --- |
| core / lifecycle / 실제 GUI | 16 + 11 + 6 = 33개 통과. GUI status·재연결·재생 종료 스크린샷도 검토. 사람 청감 아님 | [core](t00-a-evidence/distinct-user/windows-automatic.json), [lifecycle](t00-a-evidence/distinct-user/windows-lifecycle.json), [GUI](t00-a-evidence/distinct-user/windows-gui.json) |
| `./scripts/pipe-acl.ps1 -UseEphemeralCiUser -SourceSha <SHA> -OutFile <report>` | 현재 사용자 연결=0, 다른 실제 표준 로컬 계정 SID의 보호 pipe=5(거절), 같은 SID의 허용 대조군=0(연결). impersonated SID가 test SID와 일치하며 원래 SID 복귀·계정 삭제 확인 | [native evidence](t00-a-evidence/distinct-user/windows-pipe-acl.json) |
| 다운로드 무결성 | source SHA 일치·전체 manifest hash 일치·EXE 4개 x64 | [artifact inspection](t00-a-evidence/distinct-user/artifact-inspection.json) |

임시 계정 모드는 `GITHUB_ACTIONS=true`, `RUNNER_ENVIRONMENT=github-hosted`, `RUNNER_OS=Windows`를 모두 확인한다. runner에서만 임시 표준 로컬 계정을 생성해 토큰으로 신원을 전환한다. 무작위 비밀번호는 명령 인자·로그·보고서에 쓰지 않고 unmanaged 버퍼를 지우며, finally에서 계정을 제거한다. 정리 실패도 CI 실패다. 기존 계정·그룹 정책은 변경하지 않았다. 사용자 PC에서는 계정 생성 모드를 실행하지 않는다.

별도 pipe 객체의 DACL만 검증한 결과다. 실제 엔진은 TCP/token이므로 엔진의 타 사용자 접속 방지·원격 컴퓨터 차단·제품 IPC 통합 검증과 구분한다. 보안 검사를 기존 33개에 섞어 집계하지 않는다.

실패 보존: `c1e2e016`의 [run 38807808089](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/38807808089)은 LocalService 토큰 발급 실패로 전체 failure였으며 [원시 실패](t00-a-evidence/localservice-attempt/windows-pipe-acl.json)를 남긴다. `44b5104e`의 [run 38808453768](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/38808453768)은 계정 설명 48자 제한으로 생성 전에 실패했다([원시 실패](t00-a-evidence/ci-user-attempt/windows-pipe-acl.json)). 두 run 모두 core/lifecycle/GUI 33개는 통과했지만 전체 성공으로 취급하지 않는다. 설명 길이 수정 후 위 최신 SHA에서 통과했다.

계정 생성/토큰 근거(2026-10-16 KST 공식 페이지 열람): [New-LocalUser](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.localaccounts/new-localuser), [LogonUserW](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-logonuserw), [named pipe security](https://learn.microsoft.com/en-us/windows/win32/ipc/named-pipe-security-and-access-rights). 문서 확인과 CI 실측을 구분한다.

## 이전 Windows 기준선 — c778048 (증거 보존)

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

이전 c778048의 pipe 후보는 owner SID DACL과 remote reject 플래그를 시험했으나 익명 positive control이 실패했다. 당시 OS 계정/정책 변경은 없었다. 후속 계정 간 검증과 그 한계는 위 최신 증거를 따른다.

파일/DB fault injection은 temp 기록 뒤와 rename 뒤 강제 종료를 재현한다. 재기동 시 interrupted와 예약 보존·잔여 파일을 확인했으며 **자동 파일/DB 대조 복구는 구현하지 않았다**. 모의 비용/시간은 제품 정책이 아니며 실제 API 비용·응답 유실은 시험하지 않았다.

이전 `80993133331c134e8a6bea12c7a9c8e0f7493a4e`의 Linux/core Windows 원시 결과는 [기존 evidence](t00-a-evidence/)에 보존한다. Linux는 Windows 증거가 아니다. 초기 Windows icon·socket 실패와 후속 lifecycle timeout/익명 대조군 실패는 해당 run의 실패이며 최신 성공으로 소급하지 않는다.

## 재개 조건

1. [수집 절차](T00_A_WINDOWS_CHECK.md)로 정확한 SHA의 Windows 10/11 사용자 화면·청감 결과를 PR에 받는다. 빈 observation은 미검증이다. 소스가 바뀌면 해당 수정 SHA로 재실행한다.
2. 별도 타 계정 pipe ACL 검증은 위 최신 SHA에서 확보했다. 엔진 통합·원격 차단은 미검증으로 유지하고, D06 선택 후 필요한 최소 IPC 검증을 확정한다. 이미 통과한 실험을 근거 없이 반복하지 않는다.
3. 파일/DB 대조 복구와 계정 전체 소유권·busy/idle 제품 정책의 제안을 결정 자료에 추가했다. 사용자와 검토한다. fixture 구현을 제품 완료로 해석하지 않는다.
4. [D01~D06 및 조건부 T00-B 계약](T00_A_DECISIONS.md)을 사용자와 검토한다. T00-A 실기·설계 게이트와 사용자 결정 전 T00-B 금지.

artifact 보존 기간은 14일이다. 합성 fixture·EXE·스크립트·SHA/도구 정보·hash만 포함하며 FFmpeg 바이너리·임시 DB·endpoint token·실제 음원·Key는 포함하지 않는다. 배포/라이선스는 미확정이다.

## 자동 증거 대조 도구 추가

[읽기 전용 CLI](../spikes/t00-a/tools/README.md)를 추가했다. Linux Python 3.14에서 12개 회귀와 정확한 기존 ZIP artifact 14618906066의 manifest 24개·CI 33개·별도 ACL 대조를 통과했다. 사용자 JSON은 미수령이므로 실제 결과는 exit 2/pending이다. 이 검사는 기존 CI 런타임을 재실행한 것이 아니며 Windows 11 사용자 실기/청감을 대신하지 않는다. 제품 코드·GUI·collector·Windows 워크플로 변경은 없다. 도구의 Windows 직접 실행은 미검증이다.
