from pathlib import Path
import json,subprocess,concurrent.futures,time,shutil,argparse
ROOT=Path(__file__).resolve().parents[2]
SIM=(ROOT/'prompts/simulator.txt').read_text()
def dump(p,d):p.write_text(json.dumps(d,ensure_ascii=False,indent=2))
def call(model,prompt,cwd,dest,session=None,sim=False):
 dest.with_suffix('.prompt.txt').write_text(prompt)
 cmd=['grok','--model',model,'--prompt-file',str(dest.with_suffix('.prompt.txt')),'--output-format','json','--no-subagents','--disable-web-search','--verbatim']
 if sim:cmd+=['--tools','','--max-turns','1','--system-prompt-override','You simulate a feature requester. Follow the supplied simulation protocol. Do not use tools. Return only the requested JSON object.']
 else:
  cmd+=['--always-approve','--system-prompt-override','You are a coding assistant conducting requirements clarification. Follow the supplied feature-dev skill. Work only in the current repository. Do not read parent directories, user configuration, other skills or historical runs. Do not use the network. You may inspect source and write requirements documents. Do not implement code.']
  if session:cmd+=['--resume',session]
 with dest.with_suffix('.json').open('w') as f,dest.with_suffix('.stderr').open('w') as e:r=subprocess.run(cmd,cwd=cwd,stdout=f,stderr=e,timeout=900)
 if r.returncode:raise RuntimeError(f'{dest.name} exit {r.returncode}')
 raw=json.loads(dest.with_suffix('.json').read_text())
 if not raw.get('text'):raise RuntimeError(f'No text: {dest}')
 return raw

def run(arm):
 dest=OUTPUT/arm;dest.mkdir(parents=True,exist_ok=True);repo=dest/'repo';repo.mkdir();case=json.loads((ROOT/'cases'/f'{CASE}.json').read_text())
 subprocess.run(['tar','-x'],input=subprocess.check_output(['git','archive',case['base']],cwd=JUST_SRC),cwd=repo,check=True)
 for n in ['.agents','.claude','.codex','.grok']:
  if (repo/n).exists():shutil.rmtree(repo/n)
 subprocess.run(['git','init','-q'],cwd=repo,check=True)
 dialogue=[{'role':'user','text':case['opening']}];session=None;start=time.time();end='max_turns'
 try:
  for turn in range(1,31):
   prompt=dialogue[-1]['text']
   if turn==1:prompt='本次只做需求质问及文档，不实现代码。只读取当前仓库，禁止联网、读取父目录或历史。完整 feature-dev skill 如下：\n'+(ROOT/'prompts/feature-dev.md').read_text()+'\n用户需求：\n'+prompt
   if arm!='base':prompt+='\n<harness_additional_context>\n'+(ROOT/'prompts'/f'{arm}.txt').read_text()+'\n</harness_additional_context>'
   raw=call('grok-4.7',prompt,repo,dest/f'agent-{turn:02}',session);session=raw['sessionId']
   complete=bool(list((repo/'docs/prd').glob('*.md'))) and bool(list((repo/'docs/adr').glob('*.md'))) and (repo/'docs/spec.md').exists()
   dialogue.append({'role':'agent','text':raw['text'],'turn':turn,'docs_complete':complete});dump(dest/'dialogue.json',dialogue);print(arm,turn,'docs',complete,flush=True)
   if complete:end='docs_written';break
   if turn==30:break
   shown='\n\n'.join(f"【{'助手' if m['role']=='agent' else '需求提出者（你）'}】\n{m['text']}" for m in dialogue)
   simdir=dest/'sim';simdir.mkdir(exist_ok=True)
   raw=call('grok-4.6',SIM.format(brief=case['brief'],dialogue=shown),simdir,dest/f'user-{turn:02}',sim=True)
   answer=json.loads(raw['text'].strip().removeprefix('```json').removesuffix('```').strip());dialogue.append({'role':'user','text':answer['answer']});dump(dest/'dialogue.json',dialogue)
 except Exception as e:end='error';dump(dest/'error.json',{'error':str(e)});print(arm,'ERROR',str(e),flush=True)
 if (repo/'docs').exists():shutil.copytree(repo/'docs',dest/'docs',dirs_exist_ok=True)
 dump(dest/'log.json',{'arm':arm,'case':case['id'],'turns':sum(m['role']=='agent' for m in dialogue),'end':end,'seconds':round(time.time()-start),'agent':'grok-4.7','sim':'grok-4.6','session':session});print('DONE',arm,end,flush=True)
if __name__=='__main__':
 parser=argparse.ArgumentParser(description='Rerun the archived Grok grilling protocol in fresh disposable repositories.')
 parser.add_argument('--just-source',type=Path,required=True,help='Full casey/just clone containing the case base commit')
 parser.add_argument('--output',type=Path,required=True,help='New directory for repositories and raw logs')
 parser.add_argument('--case',choices=[p.stem for p in (ROOT/'cases').glob('*.json')],default='1748')
 parser.add_argument('--arms',nargs='+',choices=['base','v1','v2','v3','v4','v3b'],default=['base','v1','v2','v3','v4','v3b'])
 parser.add_argument('--workers',type=int,default=3)
 args=parser.parse_args()
 if args.workers<1:parser.error('--workers must be positive')
 JUST_SRC=args.just_source.resolve(); OUTPUT=args.output.resolve(); CASE=args.case
 if not (JUST_SRC/'.git').exists():parser.error('--just-source must be a git clone')
 if OUTPUT.exists():parser.error('--output must not already exist')
 OUTPUT.mkdir(parents=True)
 with concurrent.futures.ThreadPoolExecutor(max_workers=args.workers) as pool:list(pool.map(run,list(dict.fromkeys(args.arms))))
