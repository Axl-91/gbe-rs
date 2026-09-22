use super::*;

use rand::RngExt;

#[test]
fn dma_starts_inactive() {
    let dma = Dma::new();

    assert!(!dma.is_active());
}

#[test]
fn dma_start_sets_source_and_initial_state() {
    let mut rng = rand::rng();
    let source = rng.random();

    let mut dma = Dma::new();

    dma.start(source);

    assert!(dma.is_active());
    assert_eq!(dma.source_address(), (source as u16) << 8);
    assert_eq!(dma.get_index(), 0);
}

#[test]
fn dma_is_not_ready_before_four_t_cycles() {
    let mut dma = Dma::new();

    dma.start(0);

    for _ in 0..3 {
        dma.consume_cycle();
        assert!(!dma.is_ready_to_transfer());
    }
}

#[test]
fn dma_is_ready_after_four_t_cycles() {
    let mut dma = Dma::new();

    dma.start(0);

    for _ in 0..4 {
        dma.consume_cycle();
    }

    assert!(dma.is_ready_to_transfer());
}

#[test]
fn dma_tick_advances_to_next_byte() {
    let mut dma = Dma::new();

    dma.start(0);

    for _ in 0..4 {
        dma.consume_cycle();
    }

    dma.tick();

    assert!(dma.is_active());
    assert_eq!(dma.get_index(), 1);
    assert!(!dma.is_ready_to_transfer());
}

#[test]
fn dma_reaches_next_transfer_after_four_more_t_cycles() {
    let mut dma = Dma::new();

    dma.start(0);

    for _ in 0..4 {
        dma.consume_cycle();
    }

    dma.tick();

    for _ in 0..4 {
        dma.consume_cycle();
    }

    assert!(dma.is_ready_to_transfer());
    assert_eq!(dma.get_index(), 1);
}

#[test]
fn dma_finishes_after_last_byte() {
    let mut dma = Dma::new();

    dma.start(0);

    for _ in 0..=LAST_INDEX {
        for _ in 0..4 {
            dma.consume_cycle();
        }

        assert!(dma.is_ready_to_transfer());

        dma.tick();
    }

    assert!(!dma.is_active());
}

#[test]
fn dma_advances_index_for_each_transfer() {
    let mut dma = Dma::new();

    dma.start(0);

    for expected_index in 0..LAST_INDEX {
        for _ in 0..4 {
            dma.consume_cycle();
        }

        assert_eq!(dma.get_index(), expected_index);

        dma.tick();
    }

    for _ in 0..4 {
        dma.consume_cycle();
    }

    assert_eq!(dma.get_index(), LAST_INDEX);
}
