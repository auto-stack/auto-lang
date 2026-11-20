use ace_aac_bridge::{Domains, InputIdentity, Mode, Options, Request};
use ace_av_core::contract::{MIR_OWNED_SCHEMA, MIR_SCHEMA, TEXT_RUNTIME_CAPABILITY};
use ace_av_core::mir::*;

pub fn request(mode: Mode, mir: &MirModule, id: u8) -> Request {
    Request {
        id: [id; 16],
        mode,
        domains: Domains::windows(mode),
        inputs: InputIdentity {
            source: [1; 32],
            grammar: [2; 32],
            passes: [3; 32],
            stdlib: [4; 32],
            dependencies: [5; 32],
        },
        owner: mir.owner.module.clone(),
        mir_schema: mir.owner.schema.clone(),
        options: Options { remaining_ms: 2000 },
        capabilities: mir.capabilities.clone(),
        external_provider: None,
        globals: vec![],
        initializers: vec![],
    }
}

pub fn fixture(owned: bool) -> MirModule {
    let owner = OwnerScope {
        schema: if owned { MIR_OWNED_SCHEMA } else { MIR_SCHEMA }.into(),
        module: "aac1051".into(),
    };
    let mut mir = if owned {
        MirModule::new_owned(owner.clone())
    } else {
        MirModule::new(owner.clone())
    };
    mir.types = vec![
        MirType::I32,
        MirType::Fn(FnSig {
            params: vec![],
            result: 0,
        }),
    ];
    let (types, instructions, result) = if owned {
        mir.capabilities.push(TEXT_RUNTIME_CAPABILITY.into());
        mir.types.extend([MirType::Str, MirType::StrView]);
        mir.consts.push(MirConst::Str("a\0界🇨".as_bytes().into()));
        (
            vec![2, 3, 0],
            vec![
                MirInstr::Const { dst: 0, value: 0 },
                MirInstr::View { dst: 1, src: 0 },
                MirInstr::StrLenBytes { dst: 2, view: 1 },
                MirInstr::BorrowEnd { token: 1 },
                MirInstr::Drop { slot: 0 },
            ],
            2,
        )
    } else {
        mir.consts.push(MirConst::I32(7));
        (vec![0], vec![MirInstr::Const { dst: 0, value: 0 }], 0)
    };
    mir.funcs.push(MirFunction {
        owner,
        name: "entry".into(),
        sig: 1,
        slots: types
            .into_iter()
            .map(|ty| MirSlot {
                kind: SlotKind::Temp,
                ty,
                name: None,
            })
            .collect(),
        blocks: vec![MirBlock {
            instrs: instructions,
            term: Terminator::Return { value: result },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
        env: None,
    });
    mir
}

pub fn scalar_globals(mode: Mode) -> (Request, MirModule) {
    let mut mir = fixture(false);
    mir.owner.schema = ace_av_core::contract::MIR_CLOSURES_SCHEMA.into();
    mir.profile = ace_av_core::contract::AV_CLOSURES_PROFILE.into();
    mir.funcs[0].owner = mir.owner.clone();
    mir.funcs[0].blocks[0].instrs[0] = MirInstr::GlobalGet { dst: 0, global: 0 };
    let mut initialize = mir.funcs[0].clone();
    initialize.name = "initialize".into();
    initialize.slots.push(MirSlot {
        kind: SlotKind::Temp,
        ty: 0,
        name: None,
    });
    initialize.blocks[0] = MirBlock {
        instrs: vec![
            MirInstr::Const { dst: 0, value: 0 },
            MirInstr::GlobalSet {
                dst: 1,
                global: 0,
                src: 0,
            },
        ],
        term: Terminator::Return { value: 1 },
    };
    mir.funcs.push(initialize);
    let mut metadata = request(mode, &mir, 40);
    metadata.globals.push(ace_aac_bridge::WireGlobal {
        name: "answer".into(),
        ty: 0,
        mutable: false,
    });
    metadata.initializers.push(1);
    (metadata, mir)
}
