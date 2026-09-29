#!/usr/bin/env python3
"""Independent simulator fidelity audit; does not alter quality scores."""
import concurrent.futures,json,pathlib,sys
from run import ROOT,call,dump
PROMPT='''审计模拟用户是否忠实于隐藏 brief。只评估 role=user 且非开场的回答，禁止把助手的话当成模拟用户泄露。不要用你自己的产品偏好补全 brief。
输出 JSON：{"issues":[{"after_turn":1,"type":"leak|fabricated_preference|contradiction|false_correction","severity":"minor|major","quote":"用户原话短引文","reason":"对应 brief 和提问的具体解释"}],"summary":"中文"}。
leak：主动透露问题没涉及的隐藏要求。合理紧密关联的说明不强行标记。
fabricated_preference：brief 没有规定，但用户凭空强制指定新行为；对助手建议说“可以/按惯例来”不是凭空捏造偏好。
contradiction：用户回答违背 brief 明确偏好，或有相关偏好却错误回答“没想过”。
false_correction：用户声称纠正助手，但所谓纠正与助手原话等价。
major：实质改变合同或会诱使助手遗漏/写错隐藏要求；纯措辞或轻微额外泄露为 minor。
没有明确证据就不报。只输出 JSON。
'''
def audit(d):
    if (d/'sim-audit.json').exists():return
    l=json.loads((d/'log.json').read_text());c=json.loads((ROOT/'cases'/f"{l['case']}.json").read_text());dialog=json.loads((d/'dialogue.json').read_text())
    prompt=PROMPT+'\n隐藏 brief：\n'+c['brief']+'\n对话：\n'+json.dumps(dialog,ensure_ascii=False)
    cwd=d/'auditor';cwd.mkdir(exist_ok=True)
    txt,_,_=call('gpt-6-sol',prompt,cwd,d/'sim-audit-output',tools=False)
    a=json.loads(txt.strip().removeprefix('```json').removesuffix('```').strip())
    # Reject fabricated quotations and references to non-existent user turns.
    for i in a['issues']:
        turn=i['after_turn'];idx=next(j for j,t in enumerate(dialog) if t.get('turn')==turn)
        assert idx+1<len(dialog) and dialog[idx+1]['role']=='user'
        assert i['quote'] in dialog[idx+1]['text'],(d,i)
    dump(d/'sim-audit.json',a);print('AUDITED',d,flush=True)
if __name__=='__main__':
    dirs=[pathlib.Path(p) for p in sys.argv[1:]] or sorted(p.parent for p in (ROOT/'runs').glob('*/*/log.json'))
    with concurrent.futures.ThreadPoolExecutor(max_workers=3) as pool:
        fs={pool.submit(audit,d):d for d in dirs}
        for f in concurrent.futures.as_completed(fs):
            try:f.result()
            except Exception as e:print('ERROR',fs[f],repr(e),flush=True)
