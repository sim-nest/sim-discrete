use sim_codec::{Input, decode_with_codec};
use sim_kernel::{
    CapabilitySet, Cx, DefaultFactory, EagerPolicy, Error, ObjectCompat, ReadPolicy, Symbol,
    TrustLevel, read_construct_capability,
};
use std::sync::Arc;

use crate::{
    CombinationSpaceDescriptor, FwhtSignalDescriptor, GraphDescriptor, MatrixDescriptor,
    PermutationDescriptor, forms,
};

#[test]
fn matrix_graph_fwht_and_rank_space_lisp_round_trip() {
    let mut cx = codec_cx();

    let matrix = MatrixDescriptor::from_parts(2, 2, &[1, 2, 3, 4]).unwrap();
    assert_lisp_round_trip(&mut cx, &matrix, "discrete/Matrix", matrix.as_text());
    assert_eq!(matrix.parts().unwrap(), (2, 2, vec![1, 2, 3, 4]));

    let graph = GraphDescriptor::from_text(&forms::encode_graph(false, 3, &[(0, 1, 1), (1, 2, 2)]))
        .unwrap();
    assert_lisp_round_trip(&mut cx, &graph, "discrete/Graph", graph.as_text());
    assert_eq!(graph.graph().unwrap().edge_count(), 2);

    let signal = FwhtSignalDescriptor::from_coeffs(&[1, 0, 0, 0]).unwrap();
    assert_lisp_round_trip(&mut cx, &signal, "discrete/FwhtSignal", signal.as_text());
    assert_eq!(signal.signal().unwrap().values, vec![1, 0, 0, 0]);

    let rank_space =
        CombinationSpaceDescriptor::from_text(&forms::encode_rank_space("combination", &[6, 3]))
            .unwrap();
    assert_lisp_round_trip(
        &mut cx,
        &rank_space,
        "discrete/CombinationSpace",
        rank_space.as_text(),
    );
    let space = rank_space.space().unwrap();
    let ordinal = space.rank(&[0, 2, 4]).unwrap();
    assert_eq!(space.unrank(&ordinal).unwrap(), vec![0, 2, 4]);
}

#[test]
fn invalid_discrete_citizen_forms_fail_closed() {
    assert!(MatrixDescriptor::from_text("#(discrete/matrix v1 int 2 2 [1 2 3])").is_err());
    assert!(
        CombinationSpaceDescriptor::from_text(&forms::encode_rank_space("permutation", &[4]))
            .is_err()
    );
}

#[test]
fn cross_crate_citizen_failure_is_matchable_domain_error() {
    let error = PermutationDescriptor::from_text(&forms::encode_permutation(&[0, 0])).unwrap_err();
    let Error::DomainError {
        domain,
        category,
        message,
    } = error
    else {
        panic!("expected DomainError, got {error:?}");
    };
    assert_eq!(domain, Symbol::new("discrete"));
    assert_eq!(category, Symbol::qualified("discrete", "comb"));
    assert!(message.contains("not a permutation"));
}

fn assert_lisp_round_trip<T>(cx: &mut Cx, value: &T, class: &str, form: &str)
where
    T: ObjectCompat,
{
    let expr = value.as_expr(cx).unwrap();
    let text = format!("#({class} v1 {form:?})");
    let decoded = decode_with_codec(
        cx,
        &Symbol::qualified("codec", "lisp"),
        Input::Text(text),
        read_policy_with_construct(),
    )
    .unwrap();
    assert_eq!(decoded, expr);
}

fn codec_cx() -> Cx {
    let mut cx = Cx::new(Arc::new(EagerPolicy), Arc::new(DefaultFactory));
    cx.grant(read_construct_capability());
    cx.load_lib(&sim_citizen::CitizenLib::namespace("discrete"))
        .unwrap();
    let lisp = sim_codec_lisp::LispCodecLib::new(cx.registry_mut().fresh_codec_id()).unwrap();
    cx.load_lib(&lisp).unwrap();
    cx
}

fn read_policy_with_construct() -> ReadPolicy {
    ReadPolicy {
        trust: TrustLevel::TrustedSource,
        capabilities: CapabilitySet::new().grant(read_construct_capability()),
    }
}
