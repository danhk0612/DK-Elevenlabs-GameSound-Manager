"""Opt-in fixture lifecycle, no product defaults or user assets."""
import argparse, concurrent.futures, json, os, pathlib, platform, signal, subprocess, tempfile, time, traceback
p=argparse.ArgumentParser();p.add_argument('--bin',required=True);p.add_argument('--sha',required=True);p.add_argument('--out',required=True);a=p.parse_args()
b=pathlib.Path(a.bin).resolve();root=pathlib.Path(tempfile.mkdtemp(prefix='T00 수명 한글 '));ext='.exe' if os.name=='nt' else ''
env=dict(os.environ,T00_SPIKE_ROOT=str(root),T00_SPIKE_AUTOSTART='1',T00_SPIKE_IDLE_MS='1500')
report={'source_sha':a.sha,'os':platform.platform(),'checks':{},'scope':'fixture root only; not user-wide singleton','fixture_idle_ms':1500};roots=[]
def cli(c,e=None):
 r=subprocess.run([str(b/('spike-cli'+ext)),json.dumps(c)],env=e or env,capture_output=True,text=True,encoding='utf-8',timeout=15)
 v=json.loads(r.stdout);assert (r.returncode==0)==v['ok'] and not r.stderr;return v
def mcp(c):
 msgs=[{'jsonrpc':'2.0','id':1,'method':'initialize','params':{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'fixture','version':'0'}}},{'jsonrpc':'2.0','method':'notifications/initialized'},{'jsonrpc':'2.0','id':2,'method':'tools/call','params':{'name':'spike_command','arguments':c}}]
 r=subprocess.run([str(b/('spike-mcp'+ext))],input='\n'.join(map(json.dumps,msgs))+'\n',env=env,capture_output=True,text=True,encoding='utf-8',timeout=15)
 assert r.returncode==0 and not r.stderr
 return json.loads(json.loads(r.stdout.splitlines()[-1])['result']['content'][0]['text'])
def wait(fn,sec=10):
 end=time.monotonic()+sec
 while time.monotonic()<end:
  if fn():return
  time.sleep(.05)
 raise AssertionError('timed out')
def mark(n):report['checks'][n]='passed'
try:
 assert not cli({'op':'status'},dict(env,T00_SPIKE_AUTOSTART='0'))['ok'];mark('autostart_disabled')
 roots.append(dict(env))
 with concurrent.futures.ThreadPoolExecutor(max_workers=8) as pool:
  values=list(pool.map(lambda i:cli({'op':'status'}) if i%2 else mcp({'op':'status'}),range(8)))
 pid=values[0]['pid'];assert all(v['ok'] and v['pid']==pid for v in values);mark('cli_mcp_concurrent_autostart_one_owner')
 assert mcp({'op':'submit','id':'busy','delay_ms':3500,'cost':1})['ok']
 assert cli({'op':'shutdown_if_idle'})['error']=='engine busy';mark('busy_shutdown_if_idle_rejected')
 time.sleep(2)
 state=cli({'op':'status'});assert state['pid']==pid and state['jobs'][0]['state']=='running';mark('busy_job_survives_idle_interval_and_mcp_exit')
 wait(lambda:not (root/'endpoint.json').exists())
 assert (root/'busy.fixture').read_text()=='fixture:busy';mark('idle_after_completion_preserves_result')
 again=cli({'op':'status'});assert again['pid']!=pid and again['jobs'][0]['state']=='succeeded';mark('idle_exit_then_auto_reconnect')
 assert cli({'op':'submit','id':'crash','delay_ms':30000,'cost':1})['ok']
 os.kill(again['pid'],signal.SIGTERM);time.sleep(.2)
 restored=cli({'op':'status'});assert restored['pid']!=again['pid']
 assert next(j for j in restored['jobs'] if j['id']=='crash')['state']=='interrupted' and restored['reserved']==2;mark('stale_endpoint_kill_restart_no_replay')
 other=dict(env,T00_SPIKE_ROOT=str(root/'other workspace'));roots.append(other)
 second=cli({'op':'status'},other);assert second['pid']!=restored['pid'] and second['jobs']==[];mark('two_roots_independent_not_user_wide_singleton')
 bad=dict(env,T00_SPIKE_ROOT=str(root/'bad config'),T00_SPIKE_IDLE_MS='bad')
 assert not cli({'op':'status'},bad)['ok'];mark('invalid_idle_config_readiness_failure')
 report['result']='passed_automated_only'
except Exception:
 report['result']='failed';report['failure']=traceback.format_exc().replace(str(root),'<temporary-root>')
finally:
 for e in roots:
  e['T00_SPIKE_AUTOSTART']='0'
  try:cli({'op':'shutdown'},e)
  except Exception:pass
 pathlib.Path(a.out).write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps(report,ensure_ascii=False,indent=2))
raise SystemExit(0 if report['result']=='passed_automated_only' else 1)
