# T00-A Windows 결과 수집 경로

이 절차는 제품 설치가 아닌 격리 실험이다. 실제 게임·음원·API Key는 사용하지 않는다.

1. PR의 `T00-A Windows spike` run에서 `t00-a-windows-<40자리 source SHA>` artifact를 받는다. run head SHA와 artifact 내부 `source-sha.txt`가 같은지 확인한다. PR merge SHA가 아닌 브랜치 source SHA를 사용한다.
2. 새 한글/공백 경로에 압축을 풀고 Windows PowerShell에서 `powershell -NoProfile -ExecutionPolicy Bypass -File .\Collect-UserResult.ps1`을 실행한다. FFmpeg/ffprobe가 PATH에 있어야 한다. 없으면 결과에 실패를 남기며 사용자 설치를 자동 변경하지 않는다. WebView2가 필요하다.
3. 자동 검사 후 GUI를 직접 열어 상태 조회 → 작업 제출 → 창 닫기 → CLI 조회 → GUI 재연결을 확인한다. 합성 WAV 재생 버튼으로 소리가 실제로 들리는지 확인한다.
4. 결과 JSON의 미확인 항목을 화면/청감 관찰로 작성하고 PR Conversation에 JSON과 필요 시 화면 캡처를 첨부한다. 실행자·OS 빌드·SHA·도구 버전·명령·각 결과와 실패 메시지를 포함한다. 비밀정보/개인 경로/실제 음원을 첨부하지 않는다.

CI의 자동 검사, GUI 프로세스 기동, WebView 명령 실행, 실제 사용자 화면·청감은 별도 판정한다. 빈 수동 결과는 미검증이다. 검토자는 제출 SHA가 PR 최신 코드와 일치하는지 확인하고 CURRENT_STATE.md와 docs/T00_A_RESULTS.md에 반영한다. 다른 SHA의 성공을 최신 코드에 전용하지 않는다.
