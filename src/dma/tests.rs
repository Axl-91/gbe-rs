use super::*;

use rand::RngExt;

fn start_dma(dma: &mut Dma, source: u8) {
    dma.start(source);

    for _ in 0..START_DELAY {
        dma.consume_cycle();
        dma.tick();
    }
}

fn tick_dma(dma: &mut Dma) {
    dma.consume_cycle();
    dma.tick();
}

#[test]
fn dma_starts_inactive() {
    let dma = Dma::new();

    assert!(!dma.is_transferring());
}

#[test]
fn dma_start_sets_source_and_initial_state() {
    let mut rng = rand::rng();
    let source = rng.random();

    let mut dma = Dma::new();

    dma.start(source);

    assert!(!dma.is_transferring());
    assert_eq!(dma.get_index(), 0);
}

#[test]
fn dma_starts_transferring_after_start_delay() {
    let mut rng = rand::rng();
    let source = rng.random();

    let mut dma = Dma::new();

    dma.start(source);

    for _ in 0..START_DELAY - 1 {
        dma.consume_cycle();
        dma.tick();

        assert!(!dma.is_transferring());
    }

    dma.consume_cycle();
    dma.tick();

    assert!(dma.is_transferring());
    assert_eq!(dma.source_address(), (source as u16) << 8);
    assert_eq!(dma.get_index(), 0);
}

#[test]
fn dma_is_not_ready_before_four_t_cycles() {
    let mut dma = Dma::new();

    start_dma(&mut dma, 0);

    for _ in 0..TRANSFER_DELAY - 1 {
        tick_dma(&mut dma);

        assert!(!dma.can_transfer_byte());
    }
}

#[test]
fn dma_is_ready_after_four_t_cycles() {
    let mut dma = Dma::new();

    start_dma(&mut dma, 0);

    for _ in 0..TRANSFER_DELAY {
        dma.consume_cycle();
    }

    assert!(dma.can_transfer_byte());
}

#[test]
fn dma_tick_advances_to_next_byte() {
    let mut dma = Dma::new();

    start_dma(&mut dma, 0);

    for _ in 0..TRANSFER_DELAY {
        dma.consume_cycle();
    }

    assert!(dma.can_transfer_byte());

    dma.tick();

    assert!(dma.is_transferring());
    assert_eq!(dma.get_index(), 1);
    assert!(!dma.can_transfer_byte());
}

#[test]
fn dma_reaches_next_transfer_after_four_more_t_cycles() {
    let mut dma = Dma::new();

    start_dma(&mut dma, 0);

    for _ in 0..TRANSFER_DELAY {
        dma.consume_cycle();
    }

    dma.tick();

    for _ in 0..TRANSFER_DELAY {
        dma.consume_cycle();
    }

    assert!(dma.can_transfer_byte());
    assert_eq!(dma.get_index(), 1);
}

#[test]
fn dma_finishes_after_last_byte() {
    let mut dma = Dma::new();

    start_dma(&mut dma, 0);

    for _ in 0..=LAST_INDEX {
        for _ in 0..TRANSFER_DELAY {
            dma.consume_cycle();
        }

        assert!(dma.can_transfer_byte());

        dma.tick();
    }

    assert!(!dma.is_transferring());
}

#[test]
fn dma_advances_index_for_each_transfer() {
    let mut dma = Dma::new();

    start_dma(&mut dma, 0);

    for expected_index in 0..LAST_INDEX {
        for _ in 0..TRANSFER_DELAY {
            dma.consume_cycle();
        }

        assert_eq!(dma.get_index(), expected_index);
        assert!(dma.can_transfer_byte());

        dma.tick();
    }

    for _ in 0..TRANSFER_DELAY {
        dma.consume_cycle();
    }

    assert_eq!(dma.get_index(), LAST_INDEX);
    assert!(dma.can_transfer_byte());
}

#[test]
fn dma_restart_during_starting_waits_for_restart_delay() {
    let mut rng = rand::rng();

    let first_source: u8 = rng.random();
    let second_source: u8 = rng.random();

    let mut dma = Dma::new();

    dma.start(first_source);

    for _ in 0..START_DELAY / 2 {
        tick_dma(&mut dma);
    }

    dma.start(second_source);

    for _ in 0..RESTART_DELAY - 1 {
        tick_dma(&mut dma);
    }

    assert_eq!(dma.pending_source, Some((second_source as u16) << 8));

    tick_dma(&mut dma);

    assert_eq!(dma.source_address(), (second_source as u16) << 8);
    assert_eq!(dma.get_index(), 0);
    assert!(!dma.can_transfer_byte());
}

#[test]
fn dma_restart_during_transfer_waits_for_restart_delay() {
    let mut rng = rand::rng();

    let first_source: u8 = rng.random();
    let second_source: u8 = rng.random();

    let mut dma = Dma::new();

    dma.start(first_source);

    for _ in 0..START_DELAY {
        tick_dma(&mut dma);
    }

    assert!(dma.is_transferring());

    dma.start(second_source);

    for _ in 0..RESTART_DELAY - 1 {
        tick_dma(&mut dma);

        assert!(dma.is_transferring());
        assert_eq!(dma.source_address() >> 8, first_source as u16);
    }

    tick_dma(&mut dma);

    assert_eq!(dma.source_address(), (second_source as u16) << 8);
    assert_eq!(dma.get_index(), 0);
    assert!(!dma.can_transfer_byte());
}
