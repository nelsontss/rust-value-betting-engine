use super::*;

#[test]
fn line_key_converts_decimal_odds_lines_to_hundredths() {
    assert_eq!(250, Line(2.5).key());
    assert_eq!(200, Line(2.0).key());
    assert_eq!(-50, Line(-0.5).key());
    assert_eq!(-25, Line(-0.25).key());
    assert_eq!(0, Line(0.0).key());
}

#[test]
fn line_key_rounds_float_noise_to_canonical_value() {
    // 2.5 + tiny float drift must still map to the canonical 250 key
    assert_eq!(250, Line(2.5000002).key());
    assert_eq!(-25, Line(-0.2500001).key());
}

#[test]
fn floor_int_rounds_towards_negative_infinity() {
    assert_eq!(2, floor_int(250));
    assert_eq!(2, floor_int(275));
    assert_eq!(-3, floor_int(-250));
    assert_eq!(-3, floor_int(-275));
}

#[test]
fn ceil_int_rounds_towards_positive_infinity() {
    assert_eq!(2, ceil_int(200));
    assert_eq!(3, ceil_int(250));
    assert_eq!(-2, ceil_int(-200));
    assert_eq!(-2, ceil_int(-250));
    assert_eq!(-2, ceil_int(-275));
}

#[test]
fn line_components_keeps_full_and_half_lines_as_single_component() {
    assert_eq!(vec![250], line_components(Line(2.5)));
    assert_eq!(vec![200], line_components(Line(2.0)));
    assert_eq!(vec![-50], line_components(Line(-0.5)));
}

#[test]
fn line_components_splits_quarter_lines_into_adjacent_half_lines() {
    // -0.25 behaves like a split stake on 0.0 and -0.5
    assert_eq!(vec![0, -50], line_components(Line(-0.25)));
    // +0.25 behaves like a split stake on 0.0 and +0.5
    assert_eq!(vec![0, 50], line_components(Line(0.25)));
    // three-quarter lines split towards the surrounding half lines
    assert_eq!(vec![-50, -100], line_components(Line(-0.75)));
    assert_eq!(vec![50, 100], line_components(Line(0.75)));
}

#[test]
fn guaranteed_profit_finds_profit_when_scenarios_can_be_hedged() {
    // home wins pays 2.2x on a home-only stake, away wins pays 2.2x on away-only stake
    let scenarios = vec![(2.2, 0.0), (0.0, 2.2)];

    let profit = guaranteed_profit(&scenarios);

    assert!((profit - 0.1).abs() < 1e-9);
}

#[test]
fn guaranteed_profit_is_negative_when_no_hedge_exists() {
    // both outcomes pay less than the total stake
    let scenarios = vec![(0.5, 0.0), (0.0, 0.5)];

    let profit = guaranteed_profit(&scenarios);

    assert!(profit < 0.0);
}

#[test]
fn guaranteed_profit_is_zero_when_break_even_hedge_exists() {
    let scenarios = vec![(2.0, 0.0), (0.0, 2.0)];

    let profit = guaranteed_profit(&scenarios);

    assert!((profit - 0.0).abs() < 1e-9);
}
