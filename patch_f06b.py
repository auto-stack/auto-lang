# -*- coding: utf-8 -*-
# F06 part B: evaluate() admits the callback result; new admission variant
p = 'experimental/ac-semantics/core/src/reflect.rs'
s = open(p, encoding='utf8').read()

old = '''        self.chain.push(req.callee.clone());
        let result = body(req, self);
        self.chain.pop();
        result
    }'''
new = '''        self.chain.push(req.callee.clone());
        let result = body(req, self);
        self.chain.pop();
        // F-796-R4-06: the callback's RESULT is admitted like the
        // request — wrong schema, non-portable/Unknown values and a
        // vm_execute marking from a MODEL producer are all rejected
        let result = result?;
        result.admit_as_model()?;
        Ok(result)
    }'''
assert old in s, 'evaluate anchor'
s = s.replace(old, new)

old = '''    UncheckedExecutable,
    CyclicRequest { chain: Vec<String> },
}'''
new = '''    UncheckedExecutable,
    CyclicRequest { chain: Vec<String> },
    /// F-796-R4-06: a MODEL producer minted a vm_execute marking — the
    /// real AVM path owns that marking exclusively.
    VmExecuteFromModel,
}'''
assert old in s, 'enum anchor'
s = s.replace(old, new)

old = '''            CtAdmission::CyclicRequest { chain } => {
                write!(f, "cyclic ct request chain: {}", chain.join(" -> "))
            }'''
new = '''            CtAdmission::CyclicRequest { chain } => {
                write!(f, "cyclic ct request chain: {}", chain.join(" -> "))
            }
            CtAdmission::VmExecuteFromModel => {
                write!(
                    f,
                    "model producer minted a vm_execute marking (AVM path only)"
                )
            }'''
assert old in s, 'display anchor'
s = s.replace(old, new)
open(p, 'w', encoding='utf8', newline='\n').write(s)
print('F06B ok')
