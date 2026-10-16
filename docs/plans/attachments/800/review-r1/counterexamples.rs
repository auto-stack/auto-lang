use auto_package_resolver::PackageResolverCore;
use std::{fs, path::PathBuf};
fn main() {
 let root=std::env::temp_dir().join(format!("p800_review_{}",std::process::id()));
 let a=root.join("a"); let b=root.join("b"); let stdroot=root.join("std");
 for p in [&a,&b,&stdroot] {fs::create_dir_all(p).unwrap();}
 fs::create_dir_all(a.join("hit")).unwrap();
 fs::write(a.join("hit/package.at"),"").unwrap(); fs::write(b.join("hit.at"),"").unwrap();
 let mut c=PackageResolverCore::new(root.clone(),stdroot.clone());
 c.add_search_path(a.clone()); c.add_search_path(b.clone());
 assert_eq!(c.resolve("hit").unwrap(),a.join("hit/package.at"));
 fs::write(a.join("hit.at"),"").unwrap();
 assert_eq!(c.resolve("hit").unwrap(),a.join("hit.at"));
 c.add_dependency("hit".into(),root.join("ghost"),());
 assert_eq!(c.resolve("hit").unwrap(),root.join("ghost/package.at"));
 c.add_dependency("std.missing".into(),root.join("shadow"),());
 assert_eq!(c.resolve("std.missing").unwrap(),stdroot.join("missing.at"));
 assert!(c.exists("std.missing"));
 fs::write(a.join("a.at"),"").unwrap();
 assert_eq!(c.resolve("a.b").unwrap(),a.join("a.at"));
 let err=c.find_module_in_dep(&root.join("absent"),&["a.b"]).unwrap_err();
 assert_eq!(err,format!("Module 'a.b' not found in dependency.\n\nSearched locations:\n- {} (file module)\n- {} (directory module)\n\nDependency root: {}",root.join("absent").join("a.at").display(),root.join("absent").join("a.b").join("mod.at").display(),root.join("absent").display()));
 let empty=PackageResolverCore::<()>::new(PathBuf::new(),PathBuf::new());
 assert_eq!(empty.resolve_in_dependency("ghost",&[]).unwrap_err(),"Dependency 'ghost' not declared in pac.at.\n\nAdd it to your pac.at file:\ndep ghost(path: \"path/to/ghost\")\n\nDeclared dependencies: (none)");
 println!("PASS: seven independent priority/fallback/dotted-path/exact-error controls");
 fs::remove_dir_all(root).unwrap();
}

