import React, {useState} from 'react';
import {createRoot} from 'react-dom/client';
import {invoke} from '@tauri-apps/api/core';
function App() {
 const [result,setResult]=useState('엔진 수동 기동 후 상태 조회');
 const [audio,setAudio]=useState('');
 async function run(command: object) { setResult(JSON.stringify(await invoke('spike_command',{command}),null,2)); }
 return <main style={{fontFamily:'sans-serif',padding:24}}><h1>T00-A 격리 실험</h1><p>제품 기능 아님 · API/게임 자료 사용 금지</p>
 <button onClick={()=>run({op:'status'})}>공통 상태 조회 / 재연결</button>{' '}
 <button onClick={()=>run({op:'submit',id:`gui-${Date.now()}`,delay_ms:10000,cost:1})}>10초 모의 작업 제출</button>
 <p>작업 제출 후 창을 닫고 다시 열어 상태를 조회하세요. 엔진은 별도 프로세스입니다.</p>
 <button onClick={async()=>setAudio(await invoke<string>('fixture_audio'))}>합성 WAV 불러오기</button>
 <audio controls src={audio} onError={()=>setResult('오디오 디코딩/재생 실패')} onEnded={()=>setResult('재생 ended 이벤트 확인 — 실제 청감은 사용자 기록 필요')}/>
 <pre style={{whiteSpace:'pre-wrap'}}>{result}</pre></main>;
}
createRoot(document.getElementById('root')!).render(<App/>);
