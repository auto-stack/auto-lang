//! 内容身份（PLAN-787 §5.2）。
//!
//! 与 ac-artifact-probe 的 `identity::domain_hash` 同构：blake3，域分隔，
//! 每段长度前缀，杜绝拼接歧义。相同文本跨文档/跨工程得到相同
//! `SnapshotId`（内容身份）；工程归属隔离由产物 adapter 的 project 字段承担，
//! 不进入内容身份（“来源定位不能串工程”= 查询层带 project 上下文，而非改内容 hash）。

use std::fmt;

/// blake3 域 hash：`blake3(algo || 0 || family || 0 || (len || part)*)`。
pub fn domain_hash(family: &str, parts: &[&[u8]]) -> String {
    let mut h = blake3::Hasher::new();
    h.update(b"blake3-1.5");
    h.update(&[0]);
    h.update(family.as_bytes());
    h.update(&[0]);
    for p in parts {
        h.update(&(p.len() as u64).to_le_bytes());
        h.update(p);
    }
    h.finalize().to_hex().to_string()
}

macro_rules! id_type {
    ($name:ident, $prefix:literal) => {
        #[derive(Clone, PartialEq, Eq, Hash)]
        pub struct $name(pub String);

        impl $name {
            pub fn from_parts(parts: &[&[u8]]) -> Self {
                Self(domain_hash($prefix, parts))
            }
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({})", $prefix, &self.0[..16.min(self.0.len())])
            }
        }
    };
}

id_type!(SnapshotId, "snap");
id_type!(GrammarId, "gram");

/// 语法包身份 = 规范化程序内容 hash + schema/compiler/runtime/scanner 版本
/// （PLAN-787 §5.2：同名不同内容包隔离）。
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PackageIdentity {
    pub name: String,
    pub grammar_id: GrammarId,
}

pub fn package_identity(name: &str, canonical_program: &[u8], versions: &[&str]) -> PackageIdentity {
    let version_blob: Vec<u8> = versions
        .iter()
        .flat_map(|v| v.as_bytes().iter().copied().chain([b'\0']))
        .collect();
    PackageIdentity {
        name: name.to_string(),
        grammar_id: GrammarId::from_parts(&[name.as_bytes(), canonical_program, &version_blob]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_domain_separation_and_length_prefix() {
        let a = domain_hash("snap", &[b"ab", b"c"]);
        let b = domain_hash("snap", &[b"a", b"bc"]);
        let c = domain_hash("gram", &[b"ab", b"c"]);
        assert_ne!(a, b, "length prefix must disambiguate concatenations");
        assert_ne!(a, c, "family must separate domains");
    }

    #[test]
    fn identity_stable_across_calls() {
        let x = SnapshotId::from_parts(&[b"x"]);
        let y = SnapshotId::from_parts(&[b"x"]);
        assert_eq!(x, y);
    }
}
