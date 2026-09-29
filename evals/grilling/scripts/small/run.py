#!/usr/bin/env python3
"""Codex paired eval. Driver injects hook text, not a native Codex hook test."""
import argparse, concurrent.futures, hashlib, json, os, pathlib, shutil, subprocess, time
ROOT=pathlib.Path(__file__).resolve().parent
def dump(p,x):
    p.parent.mkdir(parents=True,exist_ok=True); p.write_text(json.dumps(x,ensure_ascii=False,indent=2))
def call(model,prompt,cwd,out,session=None,tools=True):
    cmd=['codex','exec']+(['resume',session] if session else [])
    cmd+=['--ignore-user-config','--ignore-rules','--skip-git-repo-check','--json','-m',model,'--dangerously-bypass-approvals-and-sandbox','-c','project_doc_max_bytes=0','-c','model_reasoning_effort="medium"','-c','web_search="disabled"']
    for f in ['plugins','apps','multi_agent','hooks','browser_use','computer_use','image_generation']: cmd+=['--disable',f]
    cmd+=['--enable','skip_host_skill_discovery']
    if not tools: cmd+=['--disable','shell_tool','--disable','unified_exec','-c','model_instructions_file='+json.dumps(str(ROOT/'role-only.txt'))]
    cmd+=['-']
    out.parent.mkdir(parents=True,exist_ok=True)
    with out.with_suffix('.jsonl').open('w') as log, out.with_suffix('.stderr').open('w') as err:
        p=subprocess.run(cmd,input=prompt,text=True,cwd=cwd,stdout=log,stderr=err,timeout=1500)
    if p.returncode: raise RuntimeError(f'Codex exit {p.returncode}; see {out}')
    events=[json.loads(x) for x in out.with_suffix('.jsonl').read_text().splitlines() if x.startswith('{')]
    if any(e['type']=='turn.failed' for e in events): raise RuntimeError(f'Turn failed: {out}')
    messages=[e['item']['text'] for e in events if e['type']=='item.completed' and e['item']['type']=='agent_message']
    sid=next((e['thread_id'] for e in events if e['type']=='thread.started'),session)
    usage=next((e.get('usage',{}) for e in events if e['type']=='turn.completed'),{})
    if not messages: raise RuntimeError(f'Empty response: {out}')
    return '\n\n'.join(messages),sid,usage
SIM='''You simulate a feature requester, not a coding assistant. Do not use tools.
Hidden requirements are binding preferences. Match the LAST assistant question against them by meaning, not exact wording. If the assistant offers two wrong alternatives, reject both and state your correct requirement. Never say you have no preference when the brief specifies relevant behavior. Do not reveal unrelated requirements.
Return one JSON object: {{"matched_requirements":[requirement numbers],"answer":"short Chinese requester response"}}.
Only when NO requirement applies, say 没想过，按仓库惯例来. Do not invent additional requirements.
When asked to confirm a closing summary, reject any contradictions with the brief; otherwise say 可以，写吧. Do not volunteer requirements merely absent from the summary. When no question or confirmation is present, say 继续.
HIDDEN BRIEF:\n{brief}\n
CONVERSATION (answer the final assistant message):\n{dialogue}'''
def run(caseid,arm,trial,maxturns):
    case=json.loads((ROOT/'cases'/f'{caseid}.json').read_text()); dest=ROOT/'runs'/caseid/f'{arm}-t{trial}'
    if (dest/'log.json').exists(): return json.loads((dest/'log.json').read_text())
    dest.mkdir(parents=True,exist_ok=True); repo=dest/'repo'
    if repo.exists(): raise RuntimeError(f'Partial run exists, preserve it: {repo}')
    repo.mkdir(); archive=subprocess.check_output(['git','archive',case['base']],cwd='/tmp/just-src')
    subprocess.run(['tar','-x'],input=archive,cwd=repo,check=True)
    # Remove agent configuration from the snapshot, supply exactly the frozen skill.
    for name in ['.agents','.claude','.codex']:
        if (repo/name).exists(): shutil.rmtree(repo/name)
    subprocess.run(['git','init','-q'],cwd=repo,check=True)
    skill=(ROOT/'feature-dev.md').read_text(); hook=(ROOT/('hook-v3b.txt' if arm=='hook3b' else 'hook.txt')).read_text()
    dialogue=[{'role':'user','text':case['opening']}]; sid=None; started=time.time(); usages=[]; end='max_turns'
    for n in range(1,maxturns+1):
        prompt=dialogue[-1]['text']
        if n==1:
            prompt='本次任务只做需求质问及文档，不实现代码。以下为完整 feature-dev skill。不要加载其它技能。只读取当前仓库，不联网，不查看父目录、历史运行或 git 历史。用当前源码查证行为。每轮通过最终回复提出一个问题并等待用户回答。\n'+skill+'\n\n用户需求：\n'+prompt
        if arm.startswith('hook'): prompt+='\n\n<harness_additional_context>\n'+hook+'\n</harness_additional_context>'
        (dest/f'input-{n:02}.txt').write_text(prompt)
        text,sid,usage=call('gpt-6-sol',prompt,repo,dest/f'agent-{n:02}',sid)
        files=list((repo/'docs'/'prd').glob('*.md'))
        finished=bool(files) and bool(list((repo/'docs'/'adr').glob('*.md'))) and (repo/'docs'/'spec.md').exists()
        dialogue.append({'role':'agent','text':text,'wrote_prd':bool(files),'docs_complete':finished,'turn':n}); usages.append({'role':'agent',**usage})
        dump(dest/'dialogue.json',dialogue)
        print(f'{caseid} {arm} t{trial} turn {n} docs={finished}',flush=True)
        if finished: end='docs_written'; break
        if n==maxturns: break
        shown='\n\n'.join(f"【{'助手' if t['role']=='agent' else '需求提出者（你）'}】\n{t['text']}" for t in dialogue)
        simdir=dest/'sim'; simdir.mkdir(exist_ok=True)
        answer,_,usage=call('gpt-6-luna',SIM.format(brief=case['brief'],dialogue=shown),simdir,dest/f'user-{n:02}',tools=False)
        response=json.loads(answer.strip().removeprefix('```json').removesuffix('```').strip())
        assert isinstance(response['answer'],str) and response['answer'].strip()
        dump(dest/f'sim-match-{n:02}.json',response)
        dialogue.append({'role':'user','text':response['answer']}); usages.append({'role':'sim',**usage}); dump(dest/'dialogue.json',dialogue)
    if (repo/'docs').exists(): shutil.copytree(repo/'docs',dest/'docs',dirs_exist_ok=True)
    log={'case':caseid,'arm':arm,'trial':trial,'turns':n,'end':end,'seconds':round(time.time()-started),'agent_model':'gpt-6-sol','sim_model':'gpt-6-luna','effort':'medium','injection':'driver appended additional context each turn','hook_sha256':hashlib.sha256(hook.encode()).hexdigest(),'usage':usages,'session':sid}
    dump(dest/'log.json',log); return log
if __name__=='__main__':
    ap=argparse.ArgumentParser();ap.add_argument('--cases',nargs='+',required=True);ap.add_argument('--arms',nargs='+',default=['base','hook']);ap.add_argument('--trial',type=int,default=1);ap.add_argument('--workers',type=int,default=3);ap.add_argument('--max-turns',type=int,default=20);a=ap.parse_args()
    with concurrent.futures.ThreadPoolExecutor(max_workers=a.workers) as pool:
        fs={pool.submit(run,c,arm,a.trial,a.max_turns):(c,arm) for c in a.cases for arm in a.arms}
        for f in concurrent.futures.as_completed(fs):
            try: print('DONE',json.dumps(f.result(),ensure_ascii=False),flush=True)
            except Exception as e: print('ERROR',fs[f],repr(e),flush=True)
