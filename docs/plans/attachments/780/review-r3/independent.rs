use auto_man::project_model_adapter::{old_normalized_from_code, new_normalized_from_code, compare_file};
use auto_project_model::{parse_document, SourceInfo};

fn parity_or_classify(code:&str) {
    let old=old_normalized_from_code(code).expect("oracle input must be valid");
    let (d,new)=new_normalized_from_code(code,&SourceInfo::inline()).expect("new parse or explicit unsupported");
    println!("input={code:?}\nstatic={} unsupported={:?}\nold={old:?}\nnew={new:?}",d.is_static_equivalent(),d.unsupported);
    assert!(!d.is_static_equivalent() || old==new,"must classify unsupported or preserve frozen fields");
}
#[test] fn quoted_array_control(){parity_or_classify("name: \"control\"\nmembers: [\"front\"]\nback: { project: \"back\" }\n");}
#[test] fn nested_variable_array(){parity_or_classify("name: \"dynamic\"\nmembers: [port]\n");}
#[test] fn nested_variable_object(){parity_or_classify("name: \"dynamic\"\nback: { project: port }\n");}
#[test] fn top_object_variable(){parity_or_classify("{ name: \"dynamic\", members: [port] }");}
#[test] fn duplicate_dep_path(){parity_or_classify("name: \"dup\"\ndep foo {\n path: \"first\"\n path: \"second\"\n}\n");}
#[test] fn legacy_wrapper_classified(){
    let code="root {\n name: \"wrapped\"\n render: \"vue\"\n}\n";
    let old=old_normalized_from_code(code).unwrap();
    let (d,new)=new_normalized_from_code(code,&SourceInfo::inline()).unwrap();
    println!("wrapper old={old:?} new={new:?} unsupported={:?}",d.unsupported);
    assert!(!d.is_static_equivalent(),"T01 explicitly excludes root wrapper from static claims");
}
#[test] fn dep_child_not_silently_lost(){
    let d=parse_document("dep foo {\n child Extra { marker: \"__CHILD_SENTINEL__\" }\n}\n",SourceInfo::inline()).unwrap();
    let repr=format!("{d:?}");println!("dep_child={repr}");
    assert!(repr.contains("__CHILD_SENTINEL__"),"unknown child must survive in raw or explicit unsupported evidence");
}
#[test] fn unknown_node_args_preserved(){
    let d=parse_document("custom(\"__ARG_ONE__\", \"__ARG_TWO__\") { tag: \"kept\" }",SourceInfo::inline()).unwrap();
    let repr=format!("{d:?}");println!("raw_args={repr}");
    assert!(repr.contains("__ARG_ONE__") && repr.contains("__ARG_TWO__"),"unmodeled node args must not disappear");
}
#[test] fn missing_source_is_not_equal_empty(){
    let path=std::env::temp_dir().join(format!("plan780-review-missing-{}-pac.at",std::process::id()));
    assert!(!path.exists());let report=compare_file(&path);
    println!("missing old={:?} new={:?} equal={:?}",report.old,report.new,report.equal_claim());
    assert!(report.old.is_err() && report.new.is_err(),"missing input must be observable, not empty static equality");
}
#[test] fn invalid_utf8_not_equal_empty(){
    let dir=std::env::temp_dir().join(format!("plan780-review-utf8-{}",std::process::id()));
    assert!(!dir.exists());std::fs::create_dir(&dir).unwrap();
    let path=dir.join("pac.at");std::fs::write(&path,[255u8,254u8]).unwrap();
    let report=compare_file(&path);std::fs::remove_file(path).unwrap();std::fs::remove_dir(dir).unwrap();
    println!("badutf8 old={:?} new={:?} equal={:?}",report.old,report.new,report.equal_claim());
    assert!(report.old.is_err() && report.new.is_err(),"invalid UTF8 must not become empty equality");
}
#[test] fn comment_eof_matches_legacy_policy(){
    let code="name: \"ok\"\n/* missing close";
    let old=old_normalized_from_code(code);
    let result=parse_document(code,SourceInfo::inline());
    println!("comment_eof_old={old:?} new={result:?}");
    if old.is_ok() {parity_or_classify(code);} else {assert!(result.is_err() || !result.unwrap().is_static_equivalent());}
}
#[test] fn normal_comment_control(){parity_or_classify("name: \"ok\" /* closed */\nrender: \"vm\"\n");}
