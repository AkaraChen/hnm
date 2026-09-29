#!/usr/bin/env python3
import concurrent.futures,json,pathlib,sys
from run import ROOT,call,dump
RUBRIC='''你是需求对话评审。不要使用工具。仅输出 JSON，无 markdown。
评估实际最终文档是否覆盖隐藏决策；不要要求逐字复现，不要因采用等价且满足要求的实现而扣分。决策并非真实用户原话，属于从合并 PR 重建的模拟偏好。
输出结构：
{"coverage":[{"decision":1,"status":"correct|wrong|missing","evidence":"短文档引文或解释"}],
"turns":[{"turn":1,"kind":"question|confirmation|delivery|other","value":"decisive|discoverable|redundant|low_value|na","summary":"中文简述","reason":"中文依据"}],
"non_goal_violations":[],"unconfirmed_assumptions":[],"simulator_leaks":[],"insight":"简短中文总结"}
coverage 每个决策一项，编号从1开始。仅依据文档评分，不能用对话代替文档。
每个助手回合一项：收尾合同确认是 confirmation，不当成低价值或多问题；真正要求多个独立新决定的提问可以在 reason 标注。
decisive=回答解决真实未定的行为或约束；discoverable=开场/已知源码已给出答案；redundant=前文已答；low_value=用户无偏好、实现细节、不会改变验收的追问。
unconfirmed_assumptions 仅列会影响用户可见合同、没有被用户确认的文档决定，常规实现细节不算。simulator_leaks 仅列未问就主动泄露隐藏要求的明确回合。
不要因为文档长就评分高。不要凭空推测未展示的代码。所有引文必须在输入中找得到。
'''
def grade(d):
    if (d/'grade.json').exists(): return
    log=json.loads((d/'log.json').read_text());case=json.loads((ROOT/'cases'/f"{log['case']}.json").read_text())
    dialog=json.loads((d/'dialogue.json').read_text())
    docs='\n\n'.join(str(p.relative_to(d/'docs'))+'\n'+p.read_text() for p in sorted((d/'docs').rglob('*.md')))
    # No arm name or hook prompt in the grader input.
    prompt=RUBRIC+'\n需求：'+case['opening']+'\n隐藏决策：'+json.dumps(case['decisions'],ensure_ascii=False)+'\n非目标：'+json.dumps(case['non_goals'],ensure_ascii=False)+'\n对话：'+json.dumps(dialog,ensure_ascii=False)+'\n最终文档：'+docs
    dest=d/'judge';dest.mkdir(exist_ok=True)
    text,_,usage=call('gpt-6-sol',prompt,dest,d/'judge-output',tools=False)
    text=text.strip();text=text.removeprefix('```json').removeprefix('```').removesuffix('```').strip();g=json.loads(text)
    assert len(g['coverage'])==len(case['decisions'])
    assert {x['decision'] for x in g['coverage']}==set(range(1,len(case['decisions'])+1))
    assert all(x['status'] in ['correct','wrong','missing'] for x in g['coverage'])
    expected={t['turn'] for t in dialog if t['role']=='agent'}
    assert len(g['turns'])==len(expected) and {t['turn'] for t in g['turns']}==expected
    assert all(t['kind'] in ['question','confirmation','delivery','other'] for t in g['turns'])
    assert all(t['value'] in ['decisive','discoverable','redundant','low_value','na'] for t in g['turns'])
    g['judge_model']='gpt-6-sol';g['usage']=usage;dump(d/'grade.json',g); print('GRADED',d,flush=True)
if __name__=='__main__':
    dirs=[pathlib.Path(p) for p in sys.argv[1:]] or sorted(p.parent for p in (ROOT/'runs').glob('*/*/log.json'))
    with concurrent.futures.ThreadPoolExecutor(max_workers=3) as ex:
        fs={ex.submit(grade,d):d for d in dirs}
        for f in concurrent.futures.as_completed(fs):
            try: f.result()
            except Exception as e: print('ERROR',fs[f],repr(e),flush=True)
