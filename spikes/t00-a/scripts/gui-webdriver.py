"""Native WebView2 automation: no invoke mocks, no test-only Rust commands."""
import argparse, base64, ctypes, json, os, pathlib, platform, socket, subprocess, tempfile, time, traceback
import urllib.request, urllib.error
p=argparse.ArgumentParser();p.add_argument('--bin',required=True);p.add_argument('--sha',required=True);p.add_argument('--out',required=True);a=p.parse_args()
b=pathlib.Path(a.bin).resolve();out=pathlib.Path(a.out);root=pathlib.Path(tempfile.mkdtemp(prefix='T00 GUI 한글 '))
env=dict(os.environ,T00_SPIKE_ROOT=str(root),T00_SPIKE_AUTOSTART='1',T00_SPIKE_IDLE_MS='20000',TAURI_WEBVIEW_AUTOMATION='true')
report={'source_sha':a.sha,'os':platform.platform(),'scope':'actual Tauri/WebView2 via Microsoft Edge WebDriver, no IPC mocks','checks':{},'human_observation':'unverified','audible_output':'unverified'}
session=None;driver=None
with socket.socket() as s:s.bind(('127.0.0.1',0));port=s.getsockname()[1]
base=f'http://127.0.0.1:{port}'
def http(method,path,body=None):
 data=None if body is None else json.dumps(body).encode()
 req=urllib.request.Request(base+path,data=data,method=method,headers={'Content-Type':'application/json'})
 try:
  with urllib.request.urlopen(req,timeout=45) as r: v=json.load(r)
 except urllib.error.HTTPError as e:raise RuntimeError(e.read().decode())
 v=v.get('value',v)
 if isinstance(v,dict) and v.get('error'):raise RuntimeError(str(v))
 return v
def wait(fn,sec=20):
 end=time.monotonic()+sec
 while time.monotonic()<end:
  try:
   v=fn()
   if v:return v
  except (OSError,RuntimeError,ValueError):pass
  time.sleep(.1)
 raise AssertionError('timed out')
def cmd(path,body=None,method='POST'):return http(method,'/session/'+session+path,body)
def js(script):return cmd('/execute/sync',{'script':script,'args':[]})
def click(selector):
 el=cmd('/element',{'using':'css selector','value':selector})
 cmd('/element/'+el['element-6066-11e4-a52e-4f735466cecf']+'/click',{})
def result():
 try:return json.loads(js("return document.getElementById('result').textContent"))
 except (ValueError,TypeError):return {}
def cli(c):
 r=subprocess.run([str(b/'spike-cli.exe'),json.dumps(c)],env=env,capture_output=True,text=True,encoding='utf-8',timeout=15)
 v=json.loads(r.stdout);assert r.returncode==0 and v['ok'];return v
def new_session():
 global session
 v=http('POST','/session',{'capabilities':{'alwaysMatch':{'browserName':'webview2','ms:edgeChromium':True,'ms:edgeOptions':{'binary':str(b/'spike-gui.exe'),'args':[]}}}})
 session=v['sessionId'];report['capabilities']={k:v['capabilities'].get(k) for k in ('browserName','browserVersion','platformName')}
 wait(lambda:js("return !!document.getElementById('status')"))
def screenshot(name):
 out.with_name(name).write_bytes(base64.b64decode(cmd('/screenshot',method='GET')))
def close_native_window():
 # WM_CLOSE exercises native Tauri close handling; avoid killing the engine process tree.
 handles=[]
 callback=ctypes.WINFUNCTYPE(ctypes.c_bool,ctypes.c_void_p,ctypes.c_void_p)
 user=ctypes.windll.user32
 @callback
 def each(hwnd,_):
  text=ctypes.create_unicode_buffer(256);user.GetWindowTextW(ctypes.c_void_p(hwnd),text,256)
  if text.value=='T00-A isolated spike':handles.append(hwnd)
  return True
 user.EnumWindows(each,0)
 assert len(handles)==1,handles
 assert user.PostMessageW(ctypes.c_void_p(handles[0]),0x0010,0,0)
 wait(lambda:not user.IsWindow(ctypes.c_void_p(handles[0])))
def mark(n):report['checks'][n]='passed'
try:
 subprocess.run(['ffmpeg','-v','error','-f','lavfi','-i','sine=frequency=440:duration=1','-c:a','pcm_s16le',str(root/'합성 음원.wav')],check=True,capture_output=True,timeout=20)
 driver=subprocess.Popen(['msedgedriver.exe',f'--port={port}','--allowed-ips=127.0.0.1'],env=env,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
 wait(lambda:http('GET','/status'))
 new_session();click('#status');first=wait(lambda:result().get('ok') and result())
 assert first['jobs']==[];pid=first['pid'];assert cli({'op':'status'})==first;mark('gui_button_real_invoke_equals_cli_status')
 screenshot('gui-status.png')
 click('#submit');submitted=wait(lambda:result().get('id') and result());job=submitted['id']
 assert next(j for j in cli({'op':'status'})['jobs'] if j['id']==job)['state']=='running';mark('gui_submit_same_engine_job')
 close_native_window();mark('native_wm_close')
 try:http('DELETE','/session/'+session)
 except RuntimeError:pass
 session=None
 wait(lambda:next(j for j in cli({'op':'status'})['jobs'] if j['id']==job)['state']=='succeeded',15)
 assert cli({'op':'status'})['pid']==pid;mark('gui_closed_cli_job_continues')
 new_session();click('#status');reconnected=wait(lambda:result().get('ok') and result())
 assert reconnected['pid']==pid and next(j for j in reconnected['jobs'] if j['id']==job)['state']=='succeeded';mark('new_gui_reconnect_same_engine_result')
 click('#load-audio');wait(lambda:js("return document.querySelector('audio').readyState >= 2"))
 click('#play-audio');wait(lambda:js("const a=document.querySelector('audio');return a.ended && a.currentTime > 0"))
 media=js("const a=document.querySelector('audio');return {duration:a.duration,currentTime:a.currentTime,ended:a.ended,error:a.error?.code ?? null}")
 assert .99<=media['duration']<=1.01 and media['error'] is None
 report['media']=media;mark('real_webview_decode_play_ended_not_human_listening');screenshot('gui-playback.png')
 report['result']='passed_automated_only'
except Exception:
 report['result']='failed';report['failure']=traceback.format_exc().replace(str(root),'<temporary-root>')
 if session:
  try:screenshot('gui-failure.png')
  except Exception:pass
finally:
 if session:
  try:http('DELETE','/session/'+session)
  except Exception:pass
 if driver and driver.poll() is None:driver.terminate();driver.wait(10)
 env['T00_SPIKE_AUTOSTART']='0'
 try:cli({'op':'shutdown'})
 except Exception:pass
 out.write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps(report,ensure_ascii=False,indent=2))
raise SystemExit(0 if report['result']=='passed_automated_only' else 1)
