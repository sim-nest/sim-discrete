use super::*;

// conformance: certified minimum-cost assignment with deterministic ties,
// insertion, deletion, doubling, and voice-crossing policy

fn matrix(rows: &[&[i64]]) -> CostMatrix<i64> {
    CostMatrix::try_from(rows.iter().map(|row| row.to_vec()).collect::<Vec<Vec<_>>>())
        .expect("matrix")
}

#[test]
fn unrestricted_assignment_is_optimal_and_certified() {
    let costs = matrix(&[&[9, 2, 7], &[6, 4, 3], &[5, 8, 1]]);
    let policy = AssignmentPolicy::new(vec![20; 3], vec![20; 3]);
    let assignment = min_cost_assignment(&costs, policy.clone()).expect("assignment");

    assert_eq!(assignment.total_cost, 9);
    assert_eq!(
        assignment.operations,
        vec![
            AssignmentOperation::Match {
                source: 0,
                target: 1,
                cost: 2,
            },
            AssignmentOperation::Match {
                source: 1,
                target: 0,
                cost: 6,
            },
            AssignmentOperation::Match {
                source: 2,
                target: 2,
                cost: 1,
            },
        ]
    );
    verify_assignment(&costs, &policy, &assignment).expect("certificate");
}

#[test]
fn insertion_deletion_and_doubling_are_jointly_optimized() {
    let costs = matrix(&[&[1, 9, 2], &[8, 8, 8]]);
    let policy = AssignmentPolicy::new(vec![7, 3, 7], vec![6, 4]).with_doubling(vec![2, 10]);
    let assignment = min_cost_assignment(&costs, policy.clone()).expect("assignment");

    assert_eq!(assignment.total_cost, 12);
    assert_eq!(
        assignment.operations,
        vec![
            AssignmentOperation::Match {
                source: 0,
                target: 0,
                cost: 1,
            },
            AssignmentOperation::Double {
                source: 0,
                target: 2,
                cost: 4,
            },
            AssignmentOperation::Delete { source: 1, cost: 4 },
            AssignmentOperation::Insert { target: 1, cost: 3 },
        ]
    );
    verify_assignment(&costs, &policy, &assignment).expect("certificate");
}

#[test]
fn rectangular_assignment_skips_forbidden_edges_with_stable_ties() {
    let mut costs = matrix(&[&[1, 1, 9], &[1, 1, 2]]);
    costs.forbid(0, 0).expect("in range");
    let policy = AssignmentPolicy::new(vec![20; 3], vec![20; 2]).with_doubling(vec![0, 0]);

    let first = min_cost_assignment(&costs, policy.clone()).expect("assignment");
    let replay = min_cost_assignment(&costs, policy.clone()).expect("replay");

    assert_eq!(first, replay);
    assert_eq!(first.total_cost, 4);
    assert_eq!(
        first.operations,
        vec![
            AssignmentOperation::Match {
                source: 0,
                target: 1,
                cost: 1,
            },
            AssignmentOperation::Match {
                source: 1,
                target: 0,
                cost: 1,
            },
            AssignmentOperation::Double {
                source: 1,
                target: 2,
                cost: 2,
            },
        ]
    );
    verify_assignment(&costs, &policy, &first).expect("certificate");
}

#[test]
fn assignment_control_charges_edges_and_enforces_bounds() {
    let costs = matrix(&[&[1, 2], &[2, 1]]);
    let policy = AssignmentPolicy::new(vec![5; 2], vec![5; 2]);
    let bounded = AlgorithmControl::default().with_max_work(1);

    assert!(matches!(
        min_cost_assignment_with_control(&costs, policy.clone(), &bounded, &NeverInterrupt),
        Err(GraphError::ControlStopped(_))
    ));

    let assignment = min_cost_assignment_with_control(
        &costs,
        policy,
        &AlgorithmControl::default(),
        &NeverInterrupt,
    )
    .expect("controlled assignment");
    assert!(assignment.receipt.edges > 0);
    assert_eq!(assignment.receipt.cells, 0);
    assignment.receipt.validate().expect("receipt");
}

#[test]
fn crossing_policy_changes_the_certified_optimum() {
    let costs = matrix(&[&[50, 1], &[1, 50]]);
    let base = AssignmentPolicy::new(vec![100; 2], vec![100; 2]);
    let crossing = min_cost_assignment(&costs, base.clone()).expect("crossing");
    let ordered = min_cost_assignment(
        &costs,
        base.with_voice_crossing(VoiceCrossingPolicy::Forbid),
    )
    .expect("ordered");

    assert_eq!(crossing.total_cost, 2);
    assert_eq!(ordered.total_cost, 100);
    assert!(matches!(
        crossing.certificate,
        AssignmentCertificate::MinCostFlow { .. }
    ));
    assert!(matches!(
        ordered.certificate,
        AssignmentCertificate::OrderPreserving { .. }
    ));
}

#[test]
fn equal_cost_ties_have_a_stable_assignment() {
    let costs = matrix(&[&[0, 0], &[0, 0]]);
    let policy = AssignmentPolicy::new(vec![5; 2], vec![5; 2]);
    let first = min_cost_assignment(&costs, policy.clone()).expect("first");
    let replay = min_cost_assignment(&costs, policy).expect("replay");

    assert_eq!(first, replay);
    assert_eq!(
        first.operations,
        vec![
            AssignmentOperation::Match {
                source: 0,
                target: 0,
                cost: 0,
            },
            AssignmentOperation::Match {
                source: 1,
                target: 1,
                cost: 0,
            },
        ]
    );
}

#[test]
fn empty_sides_produce_only_explicit_edits() {
    let no_sources = CostMatrix::new(0, 2, Vec::<i64>::new()).expect("matrix");
    let inserted = min_cost_assignment(&no_sources, AssignmentPolicy::new(vec![3, 4], Vec::new()))
        .expect("insertions");
    assert_eq!(inserted.total_cost, 7);
    assert_eq!(
        inserted.operations,
        vec![
            AssignmentOperation::Insert { target: 0, cost: 3 },
            AssignmentOperation::Insert { target: 1, cost: 4 },
        ]
    );

    let no_targets = CostMatrix::new(2, 0, Vec::<i64>::new()).expect("matrix");
    let deleted = min_cost_assignment(&no_targets, AssignmentPolicy::new(Vec::new(), vec![5, 6]))
        .expect("deletions");
    assert_eq!(deleted.total_cost, 11);
}

#[test]
fn tampered_flow_and_order_certificates_fail_closed() {
    let costs = matrix(&[&[1, 3], &[2, 1]]);
    let flow_policy = AssignmentPolicy::new(vec![5; 2], vec![5; 2]);
    let mut flow = min_cost_assignment(&costs, flow_policy.clone()).expect("flow");
    let AssignmentCertificate::MinCostFlow { potentials } = &mut flow.certificate else {
        panic!("flow certificate");
    };
    potentials.fill(100);
    potentials[0] = -100;
    assert!(matches!(
        verify_assignment(&costs, &flow_policy, &flow),
        Err(GraphError::CertificateInvalid(_))
    ));

    let ordered_policy = flow_policy.with_voice_crossing(VoiceCrossingPolicy::Forbid);
    let mut ordered = min_cost_assignment(&costs, ordered_policy.clone()).expect("ordered");
    let AssignmentCertificate::OrderPreserving { suffix_costs } = &mut ordered.certificate else {
        panic!("ordered certificate");
    };
    suffix_costs[0][0] += 1;
    assert!(matches!(
        verify_assignment(&costs, &ordered_policy, &ordered),
        Err(GraphError::CertificateInvalid(_))
    ));
}

#[test]
fn malformed_costs_and_overflow_fail_closed() {
    let ragged = CostMatrix::try_from(vec![vec![1_i64], vec![1, 2]]);
    assert!(matches!(ragged, Err(GraphError::InvalidAssignment(_))));

    let costs = CostMatrix::new(2, 0, Vec::<i64>::new()).expect("matrix");
    let policy = AssignmentPolicy::new(Vec::new(), vec![i64::MAX, i64::MAX]);
    assert!(matches!(
        min_cost_assignment(&costs, policy),
        Err(GraphError::WeightOverflow(_))
    ));

    let non_finite = CostMatrix::new(1, 1, vec![f64::NAN]).expect("matrix shape");
    assert!(matches!(
        min_cost_assignment(&non_finite, AssignmentPolicy::new(vec![1.0], vec![1.0])),
        Err(GraphError::NonFiniteCost(_))
    ));
}
