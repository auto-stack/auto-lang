use ace_av::{closures_fixture, dynamic_fixture};
use ace_av_core::abc::{decode_abc, encode_abc};
use ace_av_core::abc_verify::verify_abc_bytes;
use ace_av_core::contract::{
    ABC_CLOSURES_VERSION, ABC_DYNAMIC_VERSION, ABC_PROFILE_CLOSURES, ABC_PROFILE_DYNAMIC,
};

#[test]
fn av_closures_dynamic_wire_identity_and_cross_owner_rejection() {
    let closure = closures_fixture::first_case_module();
    let dynamic = dynamic_fixture::first_case_module();
    let c = encode_abc(&closure).unwrap().to_flat();
    let d = encode_abc(&dynamic).unwrap().to_flat();
    let c_decoded = decode_abc(&c).unwrap();
    let d_decoded = decode_abc(&d).unwrap();
    assert_eq!(
        (c_decoded.version, c_decoded.profile),
        (ABC_CLOSURES_VERSION, ABC_PROFILE_CLOSURES)
    );
    assert_eq!(
        (d_decoded.version, d_decoded.profile),
        (ABC_DYNAMIC_VERSION, ABC_PROFILE_DYNAMIC)
    );
    assert_ne!(
        (c_decoded.version, c_decoded.profile),
        (d_decoded.version, d_decoded.profile)
    );
    assert!(verify_abc_bytes(&c, &closure.owner).is_ok());
    assert!(verify_abc_bytes(&d, &dynamic.owner).is_ok());
    assert!(verify_abc_bytes(&c, &dynamic.owner).is_err());
    assert!(verify_abc_bytes(&d, &closure.owner).is_err());
}
