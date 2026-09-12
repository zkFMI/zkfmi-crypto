use zkfmi_cosnark_trial::{field::{decode, domain, encode, encode_row, F}, fold_binding_probe};
use ark_poly::EvaluationDomain;

#[test]
fn field_encoding_is_canonical() {
    for n in [0u64, 1, 2, 17, u64::MAX] {
        assert_eq!(decode(encode(F::from(n))), Some(F::from(n)));
    }
    assert!(decode([255; 32]).is_none());
}

#[test]
fn actual_fft_recovers_coefficients_and_rejects_bad_size() {
    let values = vec![F::from(2u64), F::from(3u64), F::from(5u64)];
    let encoded = encode_row(&values, 16).unwrap();
    let decoded = domain(16).unwrap().ifft(&encoded);
    assert_eq!(&decoded[..3], &values);
    assert!(decoded[3..].iter().all(|v| *v == F::from(0u64)));
    assert!(domain(7).is_err());
    assert!(encode_row(&values, 2).is_err());
}

#[test]
fn literal_equations_fail_necessary_binding_not_the_fft_check() {
    let result = fold_binding_probe::run();
    assert!(result.sumcheck_initial_identity);
    assert_eq!(result.literal_fold_rows_in_degree_bound, 7);
    assert_eq!(result.literal_fold_matches_even_plus_r_odd, 7);
    assert!(!result.literal_final_evaluation_matches_sumcheck);
    assert!(result.normalized_control_matches_sumcheck);
    assert_eq!(result.normalized_control_rows_in_degree_bound, 7);
    assert!(result.normalized_endpoint_controls_pass);
    assert!(!result.full_candidate_executed);
    assert!(!result.malicious_mpc_executed);
    assert!(!result.zero_knowledge_demonstrated);
}
