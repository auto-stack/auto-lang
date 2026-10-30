//! Bounded source inventory for PLAN-890. Parses Rust items, never Rust held
//! inside strings/templates. This is not runtime test discovery or acceptance.
use quote::ToTokens;
use serde_json::{json, Value};
use syn::visit::{self, Visit};

#[derive(Default)]
struct Items {
    scope: Vec<String>,
    rows: Vec<Value>,
}

impl Items {
    fn record(&mut self, kind: &str, name: &str, attrs: &[syn::Attribute]) {
        let mut path = self.scope.clone();
        path.push(name.to_owned());
        self.rows.push(json!({
            "kind": kind,
            "symbol": path.join("::"),
            "attributes": attrs.iter().map(|a| a.to_token_stream().to_string()).collect::<Vec<_>>(),
        }));
    }
}

impl<'ast> Visit<'ast> for Items {
    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        self.record("module", &item.ident.to_string(), &item.attrs);
        self.scope.push(item.ident.to_string());
        visit::visit_item_mod(self, item);
        self.scope.pop();
    }

    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        let owner = item.self_ty.to_token_stream().to_string();
        let label = match &item.trait_ {
            Some((_, path, _)) => format!("<{owner} as {}>", path.to_token_stream()),
            None => format!("<{owner}>"),
        };
        self.scope.push(label);
        visit::visit_item_impl(self, item);
        self.scope.pop();
    }

    fn visit_item_trait(&mut self, item: &'ast syn::ItemTrait) {
        self.record("trait", &item.ident.to_string(), &item.attrs);
        self.scope.push(item.ident.to_string());
        visit::visit_item_trait(self, item);
        self.scope.pop();
    }

    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        self.record("function", &item.sig.ident.to_string(), &item.attrs);
        self.scope.push(item.sig.ident.to_string());
        visit::visit_item_fn(self, item);
        self.scope.pop();
    }

    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        self.record("method", &item.sig.ident.to_string(), &item.attrs);
        self.scope.push(item.sig.ident.to_string());
        visit::visit_impl_item_fn(self, item);
        self.scope.pop();
    }

    fn visit_trait_item_fn(&mut self, item: &'ast syn::TraitItemFn) {
        self.record("trait_method", &item.sig.ident.to_string(), &item.attrs);
        visit::visit_trait_item_fn(self, item);
    }

    fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
        self.record("struct", &item.ident.to_string(), &item.attrs);
        visit::visit_item_struct(self, item);
    }

    fn visit_item_enum(&mut self, item: &'ast syn::ItemEnum) {
        self.record("enum", &item.ident.to_string(), &item.attrs);
        visit::visit_item_enum(self, item);
    }

    fn visit_item_type(&mut self, item: &'ast syn::ItemType) {
        self.record("type", &item.ident.to_string(), &item.attrs);
        visit::visit_item_type(self, item);
    }

    fn visit_item_static(&mut self, item: &'ast syn::ItemStatic) {
        self.record("static", &item.ident.to_string(), &item.attrs);
        visit::visit_item_static(self, item);
    }

    fn visit_item_const(&mut self, item: &'ast syn::ItemConst) {
        self.record("const", &item.ident.to_string(), &item.attrs);
        visit::visit_item_const(self, item);
    }

    fn visit_item_macro(&mut self, item: &'ast syn::ItemMacro) {
        // Deliberately retain a named unresolved expansion, rather than parse
        // macro tokens as if every token represented an executed Rust item.
        self.record(
            "macro_unexpanded",
            &item.mac.path.to_token_stream().to_string(),
            &item.attrs,
        );
        visit::visit_item_macro(self, item);
    }
}

fn inventory(source: &str) -> syn::Result<Vec<Value>> {
    let file = syn::parse_file(source)?;
    let mut visitor = Items::default();
    visitor.visit_file(&file);
    Ok(visitor.rows)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let list = std::fs::read_to_string(std::env::args().nth(1).ok_or("input list required")?)?;
    let mut files = Vec::new();
    for path in list.lines().filter(|p| !p.trim().is_empty()) {
        let source = std::fs::read_to_string(path)?;
        files.push(json!({"path": path, "items": inventory(&source)?}));
    }
    if files.is_empty() {
        return Err("zero source files".into());
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "scope": "rust-source-items-not-expanded-or-runtime-test-discovery",
            "files": files,
        }))?
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn templates_are_inert_and_real_scoped_items_are_preserved() {
        let rows = inventory(
            r####"
            // fn comment_is_not_an_item() {}
            const TEMPLATE: &str = r#"fn template_is_not_an_item() {}"#;
            mod owner {
                #[test]
                #[cfg(windows)]
                fn actual() { fn local() {} }
                struct State;
                impl State { fn method() {} }
                trait Hook { fn invoke(&self); }
                lazy_static::lazy_static! { static ref SLOT: u64 = 0; }
            }
        "####,
        )
        .unwrap();
        let symbols: Vec<_> = rows.iter().map(|r| r["symbol"].as_str().unwrap()).collect();
        assert!(symbols.contains(&"owner::actual"));
        assert!(symbols.contains(&"owner::actual::local"));
        assert!(symbols.contains(&"owner::<State>::method"));
        assert!(symbols.contains(&"owner::Hook::invoke"));
        assert!(!symbols.iter().any(|s| s.contains("not_an_item")));
        let actual = rows
            .iter()
            .find(|r| r["symbol"] == "owner::actual")
            .unwrap();
        assert_eq!(actual["attributes"].as_array().unwrap().len(), 2);
        assert!(rows.iter().any(|r| r["kind"] == "macro_unexpanded"));
    }
}
