use super::*;

// conformance: DTW/edit alignment verifies full and rolling certificates,
// subsequence/window policies, deterministic ties, bounds, and finite costs.

fn distance(left: &i32, right: &i32) -> i64 {
    i64::from((left - right).abs())
}

#[test]
fn full_alignment_returns_stable_edit_path_and_certificate() {
    let left = [1, 2, 3];
    let right = [1, 4, 3];
    let policy = DtwPolicy::new(GapPolicy::new(3_i64, 3));
    let alignment = dynamic_time_warp(&left, &right, distance, policy.clone()).expect("alignment");

    assert_eq!(alignment.score, 2);
    assert_eq!(
        alignment.steps,
        Some(vec![
            AlignmentStep::Match {
                left: 0,
                right: 0,
                cost: 0,
            },
            AlignmentStep::Match {
                left: 1,
                right: 1,
                cost: 2,
            },
            AlignmentStep::Match {
                left: 2,
                right: 2,
                cost: 0,
            },
        ])
    );
    verify_alignment(&left, &right, distance, &policy, &alignment).expect("certificate");
}

#[test]
fn subsequence_window_and_rolling_memory_are_enforced() {
    let left = [2, 3];
    let right = [9, 2, 3, 9];
    let policy = DtwPolicy::new(GapPolicy::new(5_i64, 5))
        .with_boundary(AlignmentBoundary::Subsequence)
        .with_memory(AlignmentMemory::RollingScoreOnly);
    let alignment =
        dynamic_time_warp(&left, &right, distance, policy.clone()).expect("subsequence");

    assert_eq!(alignment.score, 0);
    assert_eq!(alignment.steps, None);
    assert!(matches!(
        alignment.certificate,
        AlignmentCertificate::Rolling { endpoint: 3, .. }
    ));
    assert_eq!(alignment.receipt.peak_memory_cells, 10);
    verify_alignment(&left, &right, distance, &policy, &alignment).expect("rolling proof");

    let disconnected = policy
        .clone()
        .with_window(AlignmentWindow::Radius(0))
        .with_boundary(AlignmentBoundary::Global);
    assert!(matches!(
        dynamic_time_warp(&left, &right, distance, disconnected),
        Err(GraphError::Disconnected)
    ));
}

#[test]
fn alignment_control_and_non_finite_costs_fail_closed() {
    let policy = DtwPolicy::new(GapPolicy::new(1_i64, 1));
    assert!(matches!(
        dynamic_time_warp_with_control(
            &[1, 2],
            &[1, 2],
            distance,
            policy,
            &AlgorithmControl::default().with_max_work(1),
            &NeverInterrupt,
        ),
        Err(GraphError::ControlStopped(_))
    ));

    let non_finite = DtwPolicy::new(GapPolicy::new(1.0_f64, 1.0));
    assert!(matches!(
        dynamic_time_warp(&[1], &[1], |_left, _right| f64::NAN, non_finite),
        Err(GraphError::NonFiniteCost(_))
    ));
}

#[test]
fn tampered_alignment_certificate_fails_closed() {
    let policy = DtwPolicy::new(GapPolicy::new(2_i64, 2));
    let mut alignment =
        dynamic_time_warp(&[1, 2], &[1, 2], distance, policy.clone()).expect("alignment");
    let AlignmentCertificate::Full { cells } = &mut alignment.certificate else {
        panic!("full certificate");
    };
    cells[2][2].as_mut().expect("reachable").total_cost += 1;
    assert!(matches!(
        verify_alignment(&[1, 2], &[1, 2], distance, &policy, &alignment),
        Err(GraphError::CertificateInvalid(_))
    ));
}
