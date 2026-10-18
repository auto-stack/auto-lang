# -*- coding: utf-8 -*-
# F06: full recursive CT admission (formatting-matched anchors)
p = 'experimental/ac-semantics/core/src/reflect.rs'
s = open(p, encoding='utf8').read()

old = '''    pub fn admit(&self) -> Result<(), CtAdmission> {
        if self.schema != CT_SEAM_SCHEMA {
            return Err(CtAdmission::WrongSchema {
                expected: CT_SEAM_SCHEMA,
                found: self.schema,
            });
        }
        for a in &self.args {
            if let PortableValue::TypeRef(TypeDesc::Opaque { name }) = a {
                if name.starts_with('?') {
                    return Err(CtAdmission::UnknownValue);
                }
            }
        }
        Ok(())
    }'''
new = '''    /// Admission (F-796-R4-06 complete): reject schema drift and
    /// Unknown/unsolved markers ANYWHERE in the arguments — including
    /// inside Mode wrappers and Fn parameter/result positions — before
    /// any producer runs.
    pub fn admit(&self) -> Result<(), CtAdmission> {
        if self.schema != CT_SEAM_SCHEMA {
            return Err(CtAdmission::WrongSchema {
                expected: CT_SEAM_SCHEMA,
                found: self.schema,
            });
        }
        for a in &self.args {
            admit_portable_value(a)?;
        }
        Ok(())
    }'''
assert old in s, 'admit anchor'
s = s.replace(old, new)

# helpers + CtResult::admit_as_model before the model producer section
anchor = '// model CT producer'
i = s.find(anchor)
assert i > 0, 'model producer section'
helpers = '''/// Recursive portable-value admission (F-796-R4-06): Unknown markers
/// are rejected ANYWHERE in the value — including inside Mode wrappers
/// and Fn parameter/result positions — never only at the top level.
pub fn admit_portable_value(v: &PortableValue) -> Result<(), CtAdmission> {
    match v {
        PortableValue::Unit
        | PortableValue::I32(_)
        | PortableValue::Bool(_)
        | PortableValue::Text(_) => Ok(()),
        PortableValue::TypeRef(t) => admit_typedesc(t),
    }
}

/// Recursive TypeDesc admission, ITERATIVE (nesting is bounded by the
/// codec cap, but the walk never recurses natively).
pub fn admit_typedesc(t: &TypeDesc) -> Result<(), CtAdmission> {
    let mut stack: Vec<&TypeDesc> = vec![t];
    while let Some(node) = stack.pop() {
        match node {
            TypeDesc::Opaque { name } => {
                if name.starts_with('?') {
                    return Err(CtAdmission::UnknownValue);
                }
            }
            TypeDesc::Fn { params, ret } => {
                stack.push(ret);
                for p in params {
                    stack.push(p);
                }
            }
            TypeDesc::Mode { inner, .. } => stack.push(inner),
            // nominal refs are indirect — the target's own admission
            // happens at page level; the reference itself is portable
            TypeDesc::Record { .. }
            | TypeDesc::Unit
            | TypeDesc::I32
            | TypeDesc::U32
            | TypeDesc::I64
            | TypeDesc::U64
            | TypeDesc::F64
            | TypeDesc::Bool
            | TypeDesc::Char
            | TypeDesc::ByteBuffer => {}
        }
    }
    Ok(())
}

impl CtResult {
    /// Model-producer RESULT admission (F-796-R4-06): the schema must
    /// match, every carried value must be portable (recursively — no
    /// Unknown anywhere), and the marking must be `model_compute` — a
    /// MODEL producer can never mint `vm_execute`.
    pub fn admit_as_model(&self) -> Result<(), CtAdmission> {
        if self.schema != CT_SEAM_SCHEMA {
            return Err(CtAdmission::WrongSchema {
                expected: CT_SEAM_SCHEMA,
                found: self.schema,
            });
        }
        admit_portable_value(&self.value)?;
        if self.marking != ComputeMarking::ModelCompute {
            return Err(CtAdmission::VmExecuteFromModel);
        }
        Ok(())
    }
}

'''
# insert before the section header comment line
j = s.rfind('// ---------------------------------------------------------------------------', 0, i)
assert j > 0
s = s[:j] + helpers + s[j:]

open(p, 'w', encoding='utf8', newline='\n').write(s)
print('helpers ok')
