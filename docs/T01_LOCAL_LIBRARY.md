# T01 로컬 라이브러리 계약

이 문서는 v0.2.0의 로컬 관리 계약이다. 최신 v0.4.0의 편집/분석·catalog schema 3·39개 명령/MCP 38개 도구는 [T03 계약](T03_AUDIO.md)을 따른다. v0.3.0의 계정·생성, catalog schema 2, 총 32개 명령/31개 MCP 도구는 [T02 계약](T02_GENERATION.md)을 함께 따른다.

AI 자동화 경로를 우선 구현한다. 실제 Key·크레딧·원격 생성·게임 파일 적용은 없다. 화면/청감을 개발 착수 조건으로 요구하지 않는다.

## 명령과 스키마

`dk-sound-cli schema`는 엔진 기동 없이 MCP와 같은 도구 스키마를 JSON으로 출력한다. 실제 호출은 `dk-sound-cli request`의 UTF-8 JSON stdin 또는 해당 MCP 도구다. 요청에는 `protocol_version:1`, `op`와 아래 필드를 넣는다. MCP에는 protocol_version/op를 넣지 않고 arguments만 넘긴다. CLI의 version/status/shutdown 단축 명령도 유지한다.

| op | 필수 필드 | 선택 필드 |
| --- | --- | --- |
| project.create | project_id, workspace, name | — |
| project.list | — | after, limit, search |
| project.get | project_id | — |
| project.recover | project_id | after, limit |
| asset.import | project_id, asset_id, version_id, name, kind, source_path, source_note | parent_version_id |
| asset.list | project_id | after, limit, search, kind |
| asset.versions | project_id, asset_id | after, limit |
| asset.file | project_id, asset_id, version_id | — |
| requirement.put | project_id, requirement_id, name, kind, description | — |
| requirement.list | project_id | after, limit, search, kind |
| requirement.link | project_id, requirement_id, asset_id, version_id | — |
| character.put | project_id, character_id, name | voice_id, notes |
| character.list | project_id | after, limit, search |
| dialogue.put | project_id, dialogue_id, character_id, text, language | requirement_id |
| dialogue.list | project_id | after, limit, search, character_id |

MCP 도구명은 `sound_` + op의 마침표를 밑줄로 바꾼 이름이다. 예: `sound_project_create`, `sound_asset_import`, `sound_requirement_link`. version/status와 함께 총 17개 도구다. shutdown은 CLI만 제공한다. 실제 유효성 검증은 모든 진입점에서 공통 코드가 수행하며 추가 필드/잘못된 종류/경로를 거절한다.

- `project_id`는 모든 프로젝트 자료 명령에 명시한다. 전역 current project는 없다. 동일한 이름/asset_id/character_id를 다른 프로젝트에 사용해도 독립이다.
- ID는 1~64바이트, 소문자 ASCII 영숫자·`_`·`-`만 허용한다. 대소문자 파일 경로 충돌과 Windows CON/PRN/AUX/NUL/COM1~9/LPT1~9를 거절한다. Voice ID는 외부 서비스의 문자열 참조여서 이 ID 규칙을 적용하지 않는다.
- kind는 bgm/sfx/voice다. 이름/일반 텍스트 최대 200 UTF-8바이트, description/text는 2048, source_note/notes는 1024, workspace/source_path는 4096바이트다. `source_note`, `description`은 빈 문자열도 가능하다. 원격 Voice ID 검증이나 사용 권리 판정은 하지 않는다.
- list는 최대 10개, 기본 10개다. items/next_after를 반환하며 다음 호출에 next_after를 그대로 after로 넣는다. 응답 byte 한도 때문에 더 적게 반환할 수도 있다. 커서를 변경/분해하지 않는다. 마지막 페이지 크기가 정확히 한도와 같으면 빈 마지막 페이지를 한 번 더 조회할 수 있다.
- search는 파라미터 바인딩으로 처리하는 부분 문자열 검색이다. SQLite 기본 lower의 ASCII 대소문자 동작을 사용하며 언어별 대소문자/형태소 검색은 제공하지 않는다.
- put은 해당 ID의 메타데이터를 저장/교체한다. 생략한 선택 필드는 빈 값/연결 없음으로 바뀐다. requirement.put은 기존 파일 연결을 유지하며 연결 종류 변경 충돌을 거절한다. source/후보 파일은 put으로 변경할 수 없다.

## AI 자동화 예시

PowerShell 7의 UTF-8 파이프를 기준으로 한다. PowerShell 5.1에서 한글 JSON을 파이프할 때는 먼저 `$OutputEncoding = [System.Text.UTF8Encoding]::new($false)`로 native stdin을 UTF-8로 맞춘다. UTF-8 JSON 파일을 stdin으로 전달할 수도 있으며 CLI는 UTF-8 BOM을 허용한다.

```powershell
'{"protocol_version":1,"op":"project.create","project_id":"game-a","workspace":"D:\\GameSoundWorkspace","name":"Game A"}' | .\dk-sound-cli.exe request
'{"protocol_version":1,"op":"requirement.put","project_id":"game-a","requirement_id":"battle-hit","name":"Battle hit","kind":"sfx","description":"Short impact on a single target"}' | .\dk-sound-cli.exe request
'{"protocol_version":1,"op":"asset.import","project_id":"game-a","asset_id":"hit","version_id":"original","name":"Hit","kind":"sfx","source_path":"D:\\Sources\\hit.wav","source_note":"Describe source and rights separately"}' | .\dk-sound-cli.exe request
'{"protocol_version":1,"op":"requirement.link","project_id":"game-a","requirement_id":"battle-hit","asset_id":"hit","version_id":"original"}' | .\dk-sound-cli.exe request
```

workspace는 미리 존재하는 로컬 폴더를 지정한다. 도구가 `dk-gamesound-game-a` 하위 폴더를 새로 만들며 기존 임의 폴더 위에 덮어쓰지 않는다. 같은 project_id·이름·작업공간으로 다시 요청하면 기존 프로젝트를 반환하고 다른 값이면 conflict다. 같은 ID의 기존 자료를 삭제하거나 다른 프로젝트를 선택하지 않는다.

새 후보는 기존 asset_id에 새 version_id와 ready 상태의 parent_version_id를 지정해 asset.import한다. 첫 버전은 original, 후속 버전은 candidate다. 동일한 asset/version ID에 다른 hash·종류·이름·부모·출처 설명을 요청하면 conflict다. 데이터 삭제/원본 이동/승인/적용 명령은 없다.

## 저장 구조와 DB

- `%LOCALAPPDATA%\DK-Elevenlabs-GameSound-Manager\catalog.sqlite3`: 프로젝트 ID·이름·사용자가 지정한 작업공간과 관리 루트의 인덱스.
- `<workspace>\dk-gamesound-<project_id>\project.json`: schema/project/name 소유 표식.
- 같은 관리 루트의 `project.sqlite3`: 프로젝트별 자산·버전·필요 사운드·캐릭터·대사·작업 이벤트. foreign_keys=ON, WAL, synchronous=FULL, 3초 busy timeout.
- `originals/<asset_id>/<version_id>/source.<extension>`, `candidates/...`: 불변 버전 파일. 사용자 파일명은 출력 경로 구성에 사용하지 않는다.
- `imports/<asset_id>/<version_id>.part`: journal에 대응하는 복사 중간 파일.

두 DB의 application_id와 user_version=1을 확인한다. 빈 신규 DB만 초기화하고 알 수 없는 schema/버전/기존 DB를 임의 마이그레이션/초기화하지 않는다. 이후 schema 변경은 명시적 마이그레이션을 추가해야 한다. 프로젝트 DB가 사라지면 조용히 빈 DB를 만들지 않는다.

Windows 작업공간은 실제 local fixed/removable drive만 허용하고 UNC/mapped network drive는 거절한다. 앱 데이터/실행 파일 폴더와 겹치는 작업공간도 거절한다. 관리 파일/폴더의 symlink와 Windows reparse point를 거절한다. 같은 사용자의 외부 악성 쓰기나 OS 관리자에 대한 보안 제품은 아니며 동기화 폴더·다중 OS 사용자 공동 작업공간은 지원하지 않는다. 폴더 삭제·이동·프로젝트 이름 변경 UI는 현재 제공하지 않는다.

## 가져오기·중단 복구

1. source 파일은 읽기만 한다. 확장자·일반 파일·0바이트·최대 512 MiB를 확인하고 SHA256/바이트 수를 기록한다. 확장자 허용은 코덱/음질 검사 통과가 아니다. WAV/MP3/OGG/FLAC/M4A/AAC/OPUS/AIFF/AIF/WMA 파일 보관을 허용한다.
2. DB 트랜잭션에서 새 버전과 copying 상태·출처·부모·예상 hash를 먼저 기록한다.
3. 트랜잭션 밖에서 별도 .part에 복사하면서 hash를 계산하고 sync한다. 소스 내용/길이가 바뀌면 성공으로 확정하지 않는다.
4. 같은 볼륨에서 hard link로 최종 경로를 **기존 파일 대체 없이** 게시한다. 이 원자적 게시를 지원하는 파일시스템이 필요하다(Windows 우선 NTFS). FAT 계열 등에서는 atomic_publish_failed를 반환하고 원본/중간 파일을 보존한다. 이 경우 이동/덮어쓰기 fallback을 하지 않는다.
5. hash를 확인한 최종 파일을 ready로 DB 기록한 뒤 .part 링크를 제거한다. 일반 생성 큐·원격 생성 이력·사람 승인 상태와 구분한다.

클라이언트가 닫혀도 전용 writer 스레드의 작업은 계속된다. queue 32개와 현재 처리 중 1개를 엔진 상태에 표시하며 해당 작업 중 shutdown/idle 종료를 거절한다. 연결 대기는 10초, 응답 대기는 최대 300초다. 응답 유실 뒤 자동 재제출하지 않는다. asset.versions/asset.file로 상태를 먼저 조회한다.

중단 뒤 project.recover는 copying 버전을 페이지 단위로 대조한다. 최종 파일 또는 완전한 .part hash가 맞으면 원본 없이도 ready로 확정한다. partial/파일 없음/예상 hash 불일치는 결과 항목의 오류 code로 남기고 완성본을 덮어쓰지 않는다. 같은 asset.import를 같은 입력으로 재제출하면 journal이 소유한 미완성 .part만 다시 복사한다. 기존 ready 파일 변조/누락은 asset.file/재가져오기에서 실패로 반환하며 자동 교체하지 않는다. ready 커밋 직후 .part 제거 전 중단의 잔여 hard link 정리, 전원 장애의 모든 디렉터리 flush, 임의 관리 파일 복구는 현재 보장하지 않는다. 관리 폴더 파일을 직접 편집하지 않는다.

project.create 도중 빈 폴더만 생기고 표식이 저장되지 않은 특수 실패는 workspace conflict로 남긴다. 임의 폴더를 자동 삭제하지 않는다. 표식/DB 생성 후 인덱스 기록 전에 중단됐으면 같은 생성 요청으로 등록을 마무리할 수 있다.

## 최소 GUI

기존 엔진 상태에 프로젝트 목록/명시적 ID 선택, 사운드 검색/종류·목록/버전 선택, native audio controls만 추가했다. 쓰기/등록 폼은 만들지 않는다. 불러온 버전은 엔진과 GUI Rust 측에서 hash를 대조하며 파일당 16 MiB까지 data URL로 제공한다. WebView2에서 지원하지 않는 형식은 재생 오류를 표시한다. 미리듣기 크기/코덱 제한과 최대 512 MiB 보관 한도는 다르다. 미리듣기 성공을 실제 청감이나 T03 품질 검사로 표시하지 않는다.

파형·음량/무음/클리핑 분석·편집은 T03, 원격 Voice 확인/TTS/SFX·Key/비용 한도는 T02, 승인/게임 적용/복구는 T05다.
