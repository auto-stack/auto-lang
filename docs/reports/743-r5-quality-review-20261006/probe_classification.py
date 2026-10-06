from pathlib import Path
import sys,importlib.util,json
repo=Path(sys.argv[1]);out=Path(sys.argv[2])
s=importlib.util.spec_from_file_location('classification_probe',repo/'scripts/acc_inventory.py');ai=importlib.util.module_from_spec(s);s.loader.exec_module(ai)
samples={
 'type_qualified_parameter':'type Meter {\n fn len() int { return 1 }\n}\ntype Other {}\nfn demo(Meter Other) int {\n return Meter.len()\n}\n',
 'type_bare_parameter':'type Meter {}\ntype Other {}\nfn demo(Meter Other) {\n Meter()\n}\n',
 'type_no_binding_control':'type Meter {\n fn len() int { return 1 }\n}\nfn demo() {\n Meter.len()\n Meter()\n}\n',
 'enum_case_parameter':'enum Mode { select }\ntype Other {}\nfn demo(select Other) {\n select()\n}\n',
 'type_qualified_import':'type Meter {}\nuse "some_module" {Meter}\nfn demo() {\n Meter.len()\n}\n',
}
result={}
for name,source in samples.items():
    result[name]={'source':source,'scan':ai.scan_module(name+'.at',source).to_json()}
    print(name,[(c['name'],c['kind'],c.get('receiver')) for c in result[name]['scan']['call_candidates']])
out.write_text(json.dumps(result,ensure_ascii=False,indent=2),encoding='utf8')
