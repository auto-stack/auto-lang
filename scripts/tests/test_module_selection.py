"""Counterexamples for partition and complete-module selection boundaries."""
import copy
import os
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "test-governance"))
from module_catalog import SCHEMA, discover, load, select, source_identity, validate, verify_selection, write_catalog
from build_modules import make_catalog, navigation
from run_selected import applicability, exact_filter, summary, validate_case_result, validate_python_result


def catalog():
    rows = [{"id": f"r|binary|{n}", "recipe": "r", "binary": "binary", "test": n}
            for n in ("lex", "parse", "integration")]
    return {"schema": SCHEMA, "recipes": {"r": {}}, "obligations": rows,
            "source_identity": {"code": "source"}, "modules": [
                {"id": "lexer", "obligations": [rows[0]["id"]],
                 "source_paths": ["src/lexer.rs"], "consumers": ["parser"]},
                {"id": "parser", "obligations": [rows[1]["id"]],
                 "source_paths": ["src/parser"], "consumers": ["integration"]},
                {"id": "integration", "obligations": [rows[2]["id"]],
                 "source_paths": ["tests/integration.rs"], "consumers": []}]}


class ModuleSelection(unittest.TestCase):
    def test_golden_wrapper_requires_every_original_child_and_exact_file_identity(self):
        from execution_trace import validate_bindings
        child={'family':'legacy-golden-a2r','id':'original','source':{'path':'Cargo.toml','sha256':'source'},'oracles':[]}
        rows=[{'test':'wrapper','legacy_children':[child]}]
        event={'rust_test':'wrapper','metadata':copy.deepcopy(child),'complete':True,'observation':{'status':'PASS'}}
        self.assertTrue(validate_bindings(rows,{'invocations':[event]})['valid'])
        for mutation in ('missing','duplicate','source','test','incomplete','failed'):
            values=[copy.deepcopy(event)]
            if mutation=='missing':values=[]
            if mutation=='duplicate':values*=2
            if mutation=='source':values[0]['metadata']['source']['sha256']='wrong'
            if mutation=='test':values[0]['rust_test']='other'
            if mutation=='incomplete':values[0]['complete']=False
            if mutation=='failed':values[0]['observation']['status']='FAIL'
            with self.subTest(mutation=mutation):self.assertFalse(validate_bindings(rows,{'invocations':values})['valid'])

    def test_refresh_cannot_erase_frozen_vm_or_golden_wrapper_associations(self):
        import hashlib
        from unittest import mock
        from wrapper_bindings import apply,associations
        root=Path(__file__).resolve().parents[2]
        source={'path':'Cargo.toml','sha256':hashlib.sha256((root/'Cargo.toml').read_bytes()).hexdigest()}
        legacy={'wrapper_inventory':{'sources':[source],'bindings':[{'test':'wrapper','children':[
            {'family':'legacy-golden-a2r','id':'original','source':source,'oracles':[]}]}],'unresolved':[]}}
        vm={'declared_wrapper_bindings':[{'test':'vm_wrapper','cache_id':'original_vm'}]}
        value={'input_inventory':{'legacy-inputs':legacy,'vm-cache-inputs':vm},'obligations':[
            {'recipe':'language','binary':'auto-lang','test':'wrapper','ignored':False}]}
        contract={'schema':'test-legacy-wrapper-associations/1','required':associations(legacy,vm)}
        with mock.patch('wrapper_bindings.load',return_value=contract):
            apply(value);apply(value,verify=True)
            for mutation in ('golden','vm','row','fingerprint'):
                changed=copy.deepcopy(value)
                if mutation=='golden':changed['input_inventory']['legacy-inputs']['wrapper_inventory']['bindings']=[]
                if mutation=='vm':changed['input_inventory']['vm-cache-inputs']['declared_wrapper_bindings']=[]
                if mutation=='row':changed['obligations'][0]['legacy_children']=[]
                if mutation=='fingerprint':changed['input_inventory']['legacy-inputs']['wrapper_inventory']['sources'][0]['sha256']='wrong'
                with self.subTest(mutation=mutation),self.assertRaises(ValueError):apply(changed,verify=True)

    def test_milestone_keeps_original_tower_config_and_driver_budget(self):
        from run_selected import execution_settings
        recipe={'args':['--lib','--config-file','.config/nextest-full.toml']}
        module={'conditions':[{'kind':'milestone','required_env':{'T3_MILESTONE':'1'}}]}
        args,budget=execution_settings(recipe,module)
        self.assertTrue(args[-1].endswith('nextest-t3.toml'))
        self.assertEqual(budget,14400)
        self.assertEqual(recipe['args'][-1],'.config/nextest-full.toml')
        self.assertEqual(execution_settings(recipe,{'conditions':[]})[1],1800)
        self.assertEqual(execution_settings(recipe,module,20000)[1],20000)
        with self.assertRaises(ValueError):execution_settings({'args':[]},module)

    @unittest.skipUnless(os.name=='nt','PowerShell consumer is Windows-only')
    def test_powershell_consumer_rejects_prose_missing_id_and_failed_body(self):
        import json,subprocess,tempfile
        root=Path(__file__).resolve().parents[2]
        script='''param($Helper,$Data,$OutputRoot)
$script:fixture=Get-Content -LiteralPath $Data -Raw
function global:cargo {
 param([Parameter(ValueFromRemainingArguments=$true)][string[]]$CargoArgs)
 $index=[Array]::IndexOf($CargoArgs,'--out')
 $directory=Join-Path $CargoArgs[$index+1] 'execution'
 New-Item -ItemType Directory -Path $directory -Force | Out-Null
 if($script:fixture -ne 'null'){Set-Content -LiteralPath (Join-Path $directory 'results.json') -Value $script:fixture}
 $global:LASTEXITCODE=0
 Write-Output 'required_original PASS (prose is not evidence)'
}
. $Helper
try {Invoke-ScopedTestModules -Repo (Get-Location).Path -TestNames @('required_original') -Output $OutputRoot -RequiredIds @('required_original');exit 0}
catch {Write-Output $_;exit 1}
'''
        valid={'scope':'module','complete':True,'gate_passed':True,'results':[
            {'outcome':'PASS','exit':0,'applicable':['r|unittest|test_controls.Consumer.required_original']}]}
        with tempfile.TemporaryDirectory() as directory:
            base=Path(directory);path=base/'control.ps1';path.write_text(script)
            for mutation in ('valid','missing-id','failed-body','missing-receipt','only-not-applicable'):
                value=copy.deepcopy(valid)
                if mutation=='missing-id':value['results'][0]['applicable']=['r|binary|other']
                if mutation=='failed-body':value['results'][0]['exit']=1
                if mutation=='only-not-applicable':value['results'][0]['outcome']='NOT_APPLICABLE'
                if mutation=='missing-receipt':value=None
                data=base/(mutation+'.json');data.write_text(json.dumps(value))
                result=subprocess.run(['powershell','-NoProfile','-File',str(path),
                    '-Helper',str(root/'scripts/test-governance/invoke-modules.ps1'),
                    '-Data',str(data),'-OutputRoot',str(base/mutation)],capture_output=True,text=True,timeout=20)
                with self.subTest(mutation=mutation):self.assertEqual(result.returncode==0,mutation=='valid',result.stdout+result.stderr)

    def test_legacy_navigation_selects_whole_leaves_and_schedule_cannot_drop_recipe(self):
        import tempfile
        from module_command import legacy_seed_modules,schedule_seed_modules
        from module_catalog import write
        value=catalog();value['legacy_map']={'t':['lexer','parser'],'tv':['parser']}
        for module in value['modules']:module['recipe']='r'
        self.assertEqual(legacy_seed_modules(value,['t','tv']),['lexer','parser'])
        seeds=legacy_seed_modules(value,['t'],['lex'])
        self.assertEqual(seeds,['lexer'])
        self.assertEqual(select(value,seeds)['modules'],['integration','lexer','parser'])
        for tiers,filters in ((['t'],['missing']),(['unknown'],[]),(['t','t'],[])):
            with self.assertRaises(ValueError):legacy_seed_modules(value,tiers,filters)
        with tempfile.TemporaryDirectory() as directory:
            path=Path(directory)/'schedule.json'
            write(path,{'schema':'test-periodic-module-schedule/1','recipes':['r']})
            self.assertEqual(schedule_seed_modules(value,path),['integration','lexer','parser'])
            value['recipes']['new-owner']={}
            with self.assertRaises(ValueError):schedule_seed_modules(value,path)

    def test_test_navigation_never_filters_the_selected_module_body(self):
        from module_command import test_seed_modules
        value=catalog()
        self.assertEqual(test_seed_modules(value,['lex'],[]),['lexer'])
        self.assertEqual(test_seed_modules(value,[],['lex|parse']),['lexer','parser'])
        self.assertEqual(select(value,test_seed_modules(value,['lex'],[]))['modules'],['integration','lexer','parser'])
        with self.assertRaises(ValueError):test_seed_modules(value,['absent'],[])

    def test_rust_helper_edges_ignore_doc_guest_literals_and_nested_comments(self):
        from impact_graph import rust_reference_code,rust_references
        source='''// vm_file_tests::pilot_aggregate_vm
        /* outer /* nested */ vm_file_tests::ignored */
        fn read<'a>(value: &'a str) {
            let guest = r##"vm_file_tests::fake // not a comment"##;
            let quoted = "vm_file_tests::fake";
            let ch = '/';
            crate::tests::test_cases_tests::run_pilot_for_aggregate("vm");
        }'''
        actual=rust_reference_code(source)
        self.assertNotIn('vm_file_tests',actual)
        self.assertIn("fn read<'a>(value: &'a str)",actual)
        self.assertIn('crate::tests::test_cases_tests::run_pilot_for_aggregate',actual)
        references=rust_references(source)
        self.assertIn('test_cases_tests',references)
        self.assertNotIn('vm_file_tests',references)
        self.assertIn('vm_file_tests',rust_references('use super::{vm_file_tests as cache};'))

    def test_wrapper_exit_zero_cannot_replace_bound_child_execution(self):
        from execution_trace import validate_bindings
        binding={'kind':'vm-cache','id':'case','source_sha256':'source','oracle_sha256':'oracle'}
        rows=[{'test':'registered_wrapper','legacy_input':binding}]
        traces={'invocations':[]}
        self.assertFalse(validate_bindings(rows,traces)['valid'])
        traces['invocations']=[{'rust_test':'registered_wrapper','complete':True,
            'metadata':{'family':'legacy-vm-cache',**binding},'observation':{'status':'PASS'}}]
        self.assertTrue(validate_bindings(rows,traces)['valid'])
        traces['invocations'][0]['metadata']['oracle_sha256']='wrong'
        self.assertFalse(validate_bindings(rows,traces)['valid'])
        self.assertFalse(validate_bindings([],traces)['valid'])

    def test_registered_nested_rust_namespace_maps_to_deepest_actual_source(self):
        import tempfile
        from impact_graph import rust_source
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory)
            source=root/'crate/src/parser/tests.rs';source.parent.mkdir(parents=True);source.write_text('')
            (root/'crate/src/parser.rs').write_text('')
            row={'package_path':'crate','binary':'crate','test':'parser::tests::check_type'}
            self.assertEqual(rust_source(root,row),(source,'parser::tests::'))

    def test_input_oracle_keeps_producer_order_and_detects_real_sidecar_change(self):
        import hashlib,json,tempfile
        from unittest.mock import patch
        import input_inventory
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory)
            source=root/'case.at';source.write_bytes(b'1+2')
            expected=root/'case.expected.result';expected.write_bytes(b'3')
            wrapper=root/'crates/auto-lang/src/tests/vm_file_tests.rs';wrapper.parent.mkdir(parents=True)
            wrapper.write_bytes(b'wrapper')
            def sha(data):return hashlib.sha256(data).hexdigest()
            oracle={'stdout':None,'result':'3','error_presence':False}
            case={'id':'case','source_path':'case.at','source_sha256':sha(b'1+2'),'oracle':oracle,
                  'oracle_sha256':sha(json.dumps(oracle,separators=(',',':')).encode())}
            input_id={'kind':'vm-cache','id':'case','source_sha256':case['source_sha256'],'oracle_sha256':case['oracle_sha256']}
            c={'input_inventory':{'legacy-inputs':{'cases':[{'family':'vm','id':'case'}],
                    'wrapper_inventory':{'sources':[],'bindings':[],'unresolved':[]}},
                'vm-cache-inputs':{'cases':[case],'wrapper_source_sha256':sha(b'wrapper'),
                    'declared_wrapper_bindings':[{'test':'test_case','cache_id':'case','registered':True,'runtime_ignored':False}]}},
               'obligations':[{'recipe':'language','binary':'auto-lang','test':'test_case','ignored':False,
                               'legacy_input':input_id,'input_paths':['case.at','case.expected.result']}]}
            contract=root/'scripts/test-governance/input-contract.json';contract.parent.mkdir(parents=True)
            contract.write_text(json.dumps({'schema':'test-input-universe/1','vm_cache_ids':['case'],'legacy_ids':[['vm','case']]}))
            (contract.parent/'legacy-wrapper-contract.json').write_text(json.dumps({
                'schema':'test-legacy-wrapper-associations/1',
                'required':[{'kind':'vm-cache','test':'test_case','id':'case'}]}))
            with patch.object(input_inventory,'ROOT',root),patch('wrapper_bindings.ROOT',root):
                input_inventory.verify_inputs(c)
                expected.write_bytes(b'4')
                with self.assertRaisesRegex(ValueError,'oracle hash'):input_inventory.verify_inputs(c)
                expected.write_bytes(b'3')
                c['obligations'][0]['input_paths'].pop()
                with self.assertRaisesRegex(ValueError,'consumer edge'):input_inventory.verify_inputs(c)

    def test_windows_long_command_refused_before_workload_spawn(self):
        import os
        if os.name!='nt':self.skipTest('Windows argv boundary')
        from run_selected import command
        from unittest.mock import patch
        with patch('native_meter.measure') as meter:
            with self.assertRaisesRegex(ValueError,'argv budget'):
                command(Path.cwd(),['python','x'*24001],Path('unused.log'),1,{})
            meter.assert_not_called()

    def test_child_trace_keeps_negative_not_run_and_incomplete_without_inference(self):
        from execution_trace import collect
        import json,tempfile
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory)
            event={"schema":"test-execution-trace/1","module":"m","invocation":"1-0",
                   "phase":"start","rust_test":"wrapper","data":{"family":"suite"}}
            (root/'start.json').write_text(json.dumps(event))
            result=collect(root,'m')
            self.assertEqual(result['incomplete'],1)
            self.assertEqual(result['invocations'][0]['observation']['status'],'ERROR')
            event.update(phase='finish',data={'results':[{'status':'FAIL'},{'status':'NOT_RUN'}]})
            (root/'finish.json').write_text(json.dumps(event))
            result=collect(root,'m')
            self.assertEqual(result['incomplete'],0)
            self.assertEqual([r['status'] for r in result['invocations'][0]['observation']['results']],['FAIL','NOT_RUN'])
            with self.assertRaises(ValueError):collect(root,'other-module')
            (root/'duplicate.json').write_text(json.dumps(event))
            with self.assertRaises(ValueError):collect(root,'m')

    def test_public_module_command_has_no_implicit_all(self):
        import subprocess
        process=subprocess.run([sys.executable,str(Path(__file__).resolve().parents[1]/'test-governance/module_command.py')],
                               capture_output=True,text=True)
        self.assertEqual(process.returncode,2)
        self.assertIn('name at least one module',process.stderr)

    def test_worker_tools_keep_regular_and_batch_features_separate(self):
        from prepare_tools import recipe_tools
        recipe={"tools":["av","ac"],"tool_overrides":[{"binary":"b","test_prefix":"batch::","tools":["batch-av"]}]}
        batch={"binary":"b","test":"batch::real"}
        normal={"binary":"b","test":"ordinary"}
        self.assertEqual(recipe_tools(recipe,[normal]),["ac","av"])
        self.assertEqual(recipe_tools(recipe,[batch]),["batch-av"])
        with self.assertRaises(ValueError):recipe_tools(recipe,[batch,normal])

    def test_original_conditions_exclude_bodies_without_claiming_green(self):
        row={"id":"r|b|tower","conditions":[{"kind":"milestone","required_env":{"T3_MILESTONE":"1"}}]}
        self.assertEqual(len(applicability([row],{})),1)
        self.assertEqual(applicability([row],{"T3_MILESTONE":"1"}),[])
        self.assertEqual(len(applicability([{"id":"r|b|ignored","ignored":True}],{})),1)
        with self.assertRaises(ValueError):applicability([row,{"id":"r|b|ordinary"}],{})
        excluded={"exit":None,"outcome":"NOT_APPLICABLE"}
        result=summary([{"exit":0,"outcome":"PASS"},excluded],2)
        self.assertTrue(result["gate_passed"])
        self.assertFalse(result["passed"])
        self.assertEqual(result["excluded_modules"],1)
        self.assertFalse(summary([excluded],1)["gate_passed"])
        self.assertFalse(summary([{"exit":0,"outcome":"PASS"}],2)["gate_passed"])

    def test_shards_preserve_partition_and_reject_edited_source(self):
        import tempfile
        with tempfile.TemporaryDirectory() as directory:
            c=catalog()
            for m in c["modules"]:m["recipe"]="r"
            path=Path(directory)/"modules.json"
            write_catalog(path,c)
            rebuilt=load(path)
            validate(rebuilt)
            self.assertEqual({r["id"] for r in rebuilt["obligations"]},{r["id"] for r in c["obligations"]})
            shard=next((Path(directory)/"catalog").iterdir())
            shard.write_text("{}")
            with self.assertRaises(ValueError):load(path)

    def test_case_meta_oracle_preserves_failed_status_and_rejects_infra_or_missing(self):
        import json
        import tempfile
        row={"case_id":"red","check_id":"bad-golden","expected_status":"FAIL",
             "source_sha256":"source","oracle_sha256":"oracle"}
        result={"case":"red","check":"bad-golden","status":"FAIL","mismatches":[{"observation":"artifact"}],
                "identity":{"source":"source","oracle":"oracle"}}
        report={"schema":"test-case-module-result/1","scope":"case","selected_gate_exit":1,
                "selected_report":{"results":[result],"load_errors":[],"coverage_gaps":[]},
                "unselected":[{"status":"NOT_RUN"}]}
        with tempfile.TemporaryDirectory() as directory:
            path=Path(directory)/"report.json"
            path.write_text(json.dumps(report))
            checked=validate_case_result(path,[row],1)
            self.assertTrue(checked["assertions_satisfied"])
            self.assertEqual(checked["checks"][0]["raw_status"],"FAIL")
            for axis in ("infra","identity","scope","missing"):
                bad=copy.deepcopy(report)
                if axis=="infra":bad["selected_report"]["results"][0]["status"]="ERROR"
                if axis=="identity":bad["selected_report"]["results"][0]["identity"]["oracle"]="other"
                if axis=="scope":bad["unselected"][0]["status"]="PASS"
                if axis=="missing":bad["selected_report"]["results"]=[]
                path.write_text(json.dumps(bad))
                try:checked=validate_case_result(path,[row],2 if axis=="infra" else 1)
                except ValueError:continue
                self.assertFalse(checked["assertions_satisfied"],axis)

    def test_legacy_membership_and_conditions_are_separate_complete_leaves(self):
        import sys
        names=("topic::normal","topic::slow","topic::tower","topic::ignored")
        cases={n:{"ignored":n.endswith("ignored"),"filter-match":{"status":"matches"}} for n in names}
        doc={"rust-suites":{"binary":{"status":"listed","testcases":cases}}}
        legacy={"schema":"test-legacy-membership/1","platform":sys.platform,
                "tiers":["t","tf","ta","t3"],"extra_conditions":[],"rows":[
                    {"binary":"binary","test":n,"ignored":n.endswith("ignored"),
                     "selected_in":(["t","tf","ta","t3"] if n.endswith("normal")
                        else ["tf","ta","t3"] if n.endswith("slow")
                        else ["t3"] if n.endswith("tower") else [])}
                    for n in names]}
        c=make_catalog({"language":{}},{"language":doc},legacy=legacy)
        validate(c)
        self.assertEqual(len(c["modules"]),4)
        self.assertEqual(len(c["legacy_map"]["t"]),1)
        self.assertEqual(len(c["legacy_map"]["t3"]),3)
        ignored=next(m for m in c["modules"] if any(x["kind"]=="ignored" for x in m["conditions"]))
        self.assertFalse(ignored["legacy_tiers"])
        tower=next(m for m in c["modules"] if m["legacy_tiers"]==["t3"])
        self.assertEqual(tower["conditions"][0]["required_env"],{"T3_MILESTONE":"1"})
        for mutation in ("missing","new-ignore"):
            changed=copy.deepcopy(doc)
            if mutation=="missing": del changed["rust-suites"]["binary"]["testcases"][names[0]]
            else: changed["rust-suites"]["binary"]["testcases"][names[0]]["ignored"]=True
            with self.assertRaises(ValueError):
                make_catalog({"language":{}},{"language":changed},legacy=legacy)

    def test_python_exit_zero_cannot_hide_lost_or_skipped_case(self):
        import json
        import tempfile
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)/"run.log"
            good = {"schema":"python-unittest-result/1","selected":["one","two"],
                    "executed":2,"skipped":[],"errors":[],"failures":[]}
            path.write_text(json.dumps(good)+"\n")
            validate_python_result(path,["one","two"])
            for field,value in [("selected",["one"]),("executed",1),("skipped",[["two","condition"]])]:
                bad = {**good,field:value}
                path.write_text(json.dumps(bad)+"\n")
                with self.assertRaises(ValueError):
                    validate_python_result(path,["one","two"])

    def test_cargo_runtime_edges_and_module_edges_form_one_checked_closure(self):
        c = catalog()
        for m in c["modules"]:
            m["owner_package"] = m["id"]
        c["impact_graph"] = {"schema":"test-impact-graph/1", "nodes":{
            n:{"source_paths":["pkg/"+n]} for n in ("lexer","parser","integration")},
            "edges":[["parser","integration"]]}
        r = select(c, changes=["pkg/parser/source.rs"])
        self.assertEqual(r["modules"], ["integration", "parser"])
        verify_selection(c,r,{"code":"source"})
        r["modules"].remove("integration")
        r["obligations"] = [c["modules"][1]["obligations"][0]]
        with self.assertRaises(ValueError):
            verify_selection(c,r,{"code":"source"})

    def test_python_enumeration_remains_distinct_and_rejects_zero_or_duplicates(self):
        doc = {"schema":"python-unittest-discovery/1", "tests":[{"id":"module.Case.test_a","ignored":False}]}
        rows = discover(doc,"python")
        self.assertEqual(rows[0]["id"],"python|unittest|module.Case.test_a")
        doc["tests"].append(doc["tests"][0])
        with self.assertRaises(ValueError):
            discover(doc,"python")
        doc["tests"] = []
        with self.assertRaises(ValueError):
            discover(doc,"python")

    def test_reference_document_is_input_but_plan_receipt_is_bookkeeping(self):
        import tempfile
        from unittest.mock import patch
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "auto-lang"
            (root / "docs/plans").mkdir(parents=True)
            reference = root / "docs/reference.txt"
            receipt = root / "docs/plans/progress.md"
            reference.write_text("original")
            receipt.write_text("pending")
            def git(*args, **kwargs):
                return "head" if kwargs.get("text") else b"docs/reference.txt\0docs/plans/progress.md\0"
            with patch("module_catalog.subprocess.check_output", side_effect=git):
                before = source_identity(root)
                receipt.write_text("complete")
                self.assertEqual(source_identity(root)["code"], before["code"])
                reference.write_text("changed fixture")
                self.assertNotEqual(source_identity(root)["code"], before["code"])

    def test_plan_number_is_provenance_not_module_ownership(self):
        self.assertEqual(navigation({"test": "tests::plan781_budget_http_tests::a",
                                     "binary": "auto-lang"}), "budget-http")

    def test_runtime_discovery_partitions_and_preserves_profiles_and_ignored(self):
        doc = {"rust-suites": {"binary": {"status": "listed", "testcases": {
            f"parser::tests::case_{i}": {"ignored": i == 2,
                "filter-match": {"status": "matches"}} for i in range(5)}}}}
        c = make_catalog({"a": {}, "b": {}}, {"a": doc, "b": doc}, max_cases=2)
        validate(c)
        self.assertEqual(len(c["modules"]), 6)
        self.assertEqual(len(c["obligations"]), 10)
        self.assertEqual(sum(r["ignored"] for r in c["obligations"]), 2)
        self.assertEqual({m["timing"]["state"] for m in c["modules"]}, {"unmeasured"})

    def test_runtime_discovery_cannot_accept_zero_or_missing_binary(self):
        for document in ({}, {"rust-suites": {}},
                         {"rust-suites": {"binary": {"status": "skipped"}}}):
            with self.assertRaises(ValueError):
                make_catalog({"r": {}}, {"r": document})

    def test_runtime_subset_may_filter_unselected_binary_but_not_empty_everything(self):
        doc = {"rust-suites": {
            "excluded": {"status": "skipped"},
            "selected": {"status": "listed", "testcases": {"f": {
                "ignored": False, "filter-match": {"status": "matches"}}}}}}
        with self.assertRaises(ValueError):
            discover(doc, "r")
        self.assertEqual([r["id"] for r in discover(doc, "r", allow_filtered_skipped=True)],
                         ["r|selected|f"])
        with self.assertRaises(ValueError):
            discover({"rust-suites": {"excluded": {"status": "skipped"}}}, "r",
                     allow_filtered_skipped=True)

    def test_filter_uses_exact_operator_and_rejects_injection(self):
        self.assertEqual(exact_filter([{"binary": "a::b", "test": "tests::f"}]),
                         "(binary_id(=a::b) and test(=tests::f))")
        with self.assertRaises(ValueError):
            exact_filter([{"binary": "a", "test": "f) or all()"}])
        self.assertEqual(exact_filter([{'binary':'probe::bin/worker','test':'query::test熔断'}]),
                         '(binary_id(=probe::bin/worker) and test(=query::test熔断))')

    def test_valid_partition_and_multiple_module_union(self):
        c = catalog()
        validate(c)
        r = select(c, ["parser", "integration"])
        self.assertEqual(r["modules"], ["integration", "parser"])
        self.assertEqual(len(verify_selection(c, r, {"code": "source"})), 2)

    def test_missing_and_duplicate_partition_refused(self):
        for mutation in ("missing", "duplicate", "unknown", "empty"):
            with self.subTest(mutation=mutation):
                c = catalog()
                if mutation == "missing":
                    c["modules"].pop()
                    c["modules"][1]["consumers"] = []
                elif mutation == "duplicate":
                    c["modules"][1]["obligations"] += c["modules"][0]["obligations"]
                elif mutation == "unknown":
                    c["modules"][0]["obligations"] = ["ghost"]
                else:
                    c["modules"][0]["obligations"] = []
                with self.assertRaises(ValueError):
                    validate(c)

    def test_transitive_consumer_and_boundary_path(self):
        c = catalog()
        r = select(c, changes=["src/parser/expr.rs"])
        self.assertEqual(r["modules"], ["integration", "parser"])
        self.assertFalse(r["fallback"])
        r = select(c, changes=["src/parser_extra.rs"])
        self.assertTrue(r["fallback"])
        self.assertEqual(len(r["obligations"]), 3)

    def test_cycles_terminate_and_union_once(self):
        c = catalog()
        c["modules"][2]["consumers"] = ["lexer"]
        r = select(c, ["parser"])
        self.assertEqual(len(r["obligations"]), 3)

    def test_unknown_deleted_and_config_inputs_expand(self):
        for path in ("deleted.rs", "Cargo.lock", "new-test.rs", ".config/nextest.toml"):
            r = select(catalog(), changes=[path])
            self.assertTrue(r["fallback"])
            self.assertEqual(len(r["obligations"]), 3)

    def test_unknown_named_module_is_error_not_empty_success(self):
        with self.assertRaises(ValueError):
            select(catalog(), ["typo"])

    def test_mutated_receipt_cannot_omit_consumer_or_case(self):
        c = catalog()
        good = select(c, ["lexer"])
        for axis in ("case", "module", "duplicate", "catalog", "source"):
            with self.subTest(axis=axis):
                r = copy.deepcopy(good)
                if axis == "case": r["obligations"].pop()
                elif axis == "module": r["modules"].remove("integration")
                elif axis == "duplicate": r["modules"].append("parser")
                elif axis == "catalog": r["catalog_sha256"] = "other"
                else: r["source_identity"]["code"] = "other"
                with self.assertRaises(ValueError):
                    verify_selection(c, r, {"code": "source"})

    def test_stale_catalog_expands_but_forbids_execution(self):
        c = catalog()
        r = select(c, ["parser"], current_source={"code": "new"})
        self.assertEqual(len(r["obligations"]), 3)
        self.assertFalse(r["execution_allowed"])
        with self.assertRaises(ValueError):
            verify_selection(c, r, {"code": "new"})

    def test_empty_changes_explains_only_and_cannot_execute(self):
        c = catalog()
        r = select(c)
        self.assertEqual(r["obligations"], [])
        with self.assertRaises(ValueError):
            verify_selection(c, r, {"code": "source"})


if __name__ == "__main__":
    unittest.main()
