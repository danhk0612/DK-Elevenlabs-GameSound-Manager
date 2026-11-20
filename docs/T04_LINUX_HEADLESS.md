# T04 Linux headless 실행 경로 (v0.9)

현재 Codex Linux 작업 환경에서도 공통 엔진을 직접 호출하도록 추가했다. Windows GUI/LOCALAPPDATA/Credential Manager/named pipe 계약은 유지한다. Linux GUI·설치 패키지·공개 서버를 제공하지 않는다. 공통 47개 명령/MCP 46개 도구·DB schema 5/1은 같다.

## 실행

Rust로 `cargo build --locked --manifest-path app/Cargo.toml --bins` 후 같은 폴더의 engine/cli/mcp 세 바이너리를 사용한다. Linux FFmpeg/ffprobe 실행 파일 경로를 `audio.tools.set`에 명시한다. 아래 예시 경로는 사용자가 관리하는 로컬 디스크 경로로 정한다.

```bash
mkdir -m 700 /tmp/dk-sound-private
export DK_SOUND_ROOT=/tmp/dk-sound-private
app/target/debug/dk-sound-cli status
app/target/debug/dk-sound-cli request < request.json
python3 app/automation/sound_pipeline.py recipe.json --cli app/target/debug/dk-sound-cli
python3 app/automation/sound_batch.py batch.json --cli app/target/debug/dk-sound-cli --mcp app/target/debug/dk-sound-mcp --jsonl
app/target/debug/dk-sound-cli shutdown
```

위 `/tmp`는 일회용 예시다. 장기 자료는 삭제되지 않는 상태 디렉터리를 명시한다. 작업공간은 recipe/project.create에서 별도로 지정한 기존 절대 경로다. Linux 클라이언트와 엔진 모두 동일한 `DK_SOUND_ROOT`를 사용해야 한다. MCP 클라이언트는 `command`를 dk-sound-mcp 절대 경로로, `env.DK_SOUND_ROOT`를 같은 루트로 설정한다. stdio 프로토콜과 JSON 호출은 Windows와 동일하다.

root 환경 변수가 없거나 상대 경로면 거절한다. 루트가 없으면 기존 부모 아래 한 단계만 0700으로 생성한다. 기존 루트는 현재 effective UID 소유·정확한 0700·canonical 절대 경로여야 한다. 루트/부모 symlink 경유·공유 권한을 자동 수정하지 않고 거절한다. socket 경로 UTF-8/OS 바이트가 Linux sockaddr_un 제한에 맞아야 한다. 상태 루트에 비밀정보를 저장하지 않는다.

## 엔진/파일 경계

- root별 `engine.lock`의 nonblocking flock으로 engine 1개만 소유한다. lock은 no-follow/regular file/UID/0600/단일 hardlink 검사 후 열며 lock inode를 삭제하지 않는다. 서로 다른 root는 별개 카탈로그·mock 예산이다. Windows SID별 전역 단일성과 구분한다.
- `engine.sock`은 Unix domain socket/0600이며 루트 0700으로 다른 일반 UID의 접근을 제한한다. 양쪽 `SO_PEERCRED` UID도 대조한다. TCP/HTTP/원격 MCP 수신을 추가하지 않는다. root 권한 계정/같은 UID의 임의 파일 조작을 차단하는 제품은 아니다.
- lock 소유 후에만 남은 현재 UID 소유 socket을 제거한다. 다른 종류 파일/symlink는 `socket_path_conflict`로 보존한다. 정상 종료는 socket을 제거하고, 강제 종료는 flock 해제 후 다음 기동에서 stale socket을 대조한다.
- 첫 호출은 같은 바이너리 폴더의 engine을 별도 process group으로 기동해 클라이언트 터미널 Ctrl+C가 엔진으로 전파되지 않게 한다. 요청을 쓰기 시작한 후 응답 유실/timeout이면 자동 재전송하지 않는다. 동시 기동 중 lock 패자는 종료하며 모든 클라이언트는 같은 socket으로 연결한다.
- storage worker/async fast 조회·취소/queue/검토/5분 idle/busy shutdown은 공통 구현을 공유한다. 파일/후보·원본 보존과 작업 ID 재사용·중단 무재생 정책은 기존 계약 그대로다.

## 현재 가능한 작업과 남은 생성 경로

Linux에서 CLI/MCP 파일 등록, 로컬 FFmpeg 편집/분석, async queue, 후보 해시·검토 제출·이력, 단일/다건 recipe를 실행할 수 있다. 모의 provider는 명시한 요청/credit 한도로 합성 WAV만 만들며 실제 게임용 생성 음원이 아니다.

v0.10에서 [Linux stdin 인증 세션/HTTPS](T02_LINUX_PROVIDER.md)를 추가했다. Windows Credential Manager/WinHTTP는 유지한다. Linux 일반 자동 기동에는 키가 없고, 별도 `--session` 시작 입력으로 account/허용 여부/한도/TTL을 제공해야 한다. secret은 메모리에만 보관한다. 실제 key·허용 생성 범위·비용 한도 없이 라이브 요청을 하지 않았다. D08/D09 기본 수치는 미결정이며 서로 다른 root/기기의 예산 공유를 보장하지 않는다.

이번 증거는 Codex가 쉘 도구로 native CLI와 자체 MCP stdio 클라이언트를 실행한 결과다. Codex 앱 MCP 서버 등록이나 다른 AI 제품의 연결 수용 결과는 아니다. 검토 pending/기술 검사 통과는 승인/게임 적용이 아니다. T04 전체/T05를 완료 처리하지 않는다.

## 검증

```bash
python3 app/automation/ci_transport_smoke.py --bin-dir app/target/debug --local-linux
python3 app/automation/ci_batch_smoke.py --bin-dir app/target/debug --local-linux
python3 app/automation/ci_linux_smoke.py --bin-dir app/target/debug --ffmpeg /absolute/ffmpeg --ffprobe /absolute/ffprobe
```

모두 새 임시 상태/작업공간·합성 음원만 사용한다. 세 번째 검사는 미설정/상대/공유/링크 root·endpoint 충돌 보존, 8개 동시 기동의 동일 PID·중복 engine 거절·stale socket·강제 종료/기존 작업 재조회·mock 생성/한도/동일 ID·실제 FFmpeg async edit/analyze와 CLI/MCP 재호출/원본 보존을 검사한다. 모의 큐의 일시적 output_ready 상태는 정상 완료 전 관측할 수 있으며 성공으로 처리하지 않고 기다린다. 실패 정리에서도 busy shutdown 거절을 대조한다.

CI는 별도 Linux headless job에서 실제 프로세스와 FFmpeg를 실행한다. Windows job은 별도로 기존 native/GUI/실제 CLI·MCP/다건/패키징 회귀를 실행한다. 최신 SHA/실행 결과는 CURRENT_STATE.md를 따른다.
