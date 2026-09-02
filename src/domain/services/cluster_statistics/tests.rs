use super::*;

#[test]
fn aggregates_diffs_mean_and_median() {
    let mut stats = ClusterStatistics::default();
    assert_eq!(0, stats.samples());
    assert_eq!(0.0, stats.mean_diff());
    assert_eq!(None, stats.median_diff());

    stats.add_diff(0.03);
    stats.add_diff(0.05);
    stats.add_diff(0.10);

    assert_eq!(3, stats.samples());
    assert!((stats.mean_diff() - 0.06).abs() < 1e-9);
    assert_eq!(Some(0.05), stats.median_diff());
}
