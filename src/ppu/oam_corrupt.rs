use super::{OAM_SIZE, Ppu};

const OAM_ROW_SIZE: usize = 8;
const OAM_WORD_SIZE: usize = 2;
const OAM_ROW_COUNT: usize = OAM_SIZE / OAM_ROW_SIZE;
const OAM_ROW_CYCLES: u16 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OamCorruption {
    Write,
    ReadWrite,
}

impl Ppu {
    pub(crate) fn is_oam_search(&self) -> bool {
        matches!(self.mode, super::PpuMode::OamSearch)
    }

    pub(crate) fn oam_corrupt(&mut self, corruption: OamCorruption) {
        if !self.is_oam_search() {
            return;
        }

        match corruption {
            OamCorruption::Write => self.corrupt_oam_write(),
            OamCorruption::ReadWrite => self.corrupt_oam_read_write(),
        }
    }

    fn get_oam_row(&self) -> usize {
        (self.mode_cycles / OAM_ROW_CYCLES) as usize
    }

    fn get_oam_word(&self, row: usize, word: usize) -> u16 {
        let index = row * OAM_ROW_SIZE + word * OAM_WORD_SIZE;
        u16::from_le_bytes([self.oam[index], self.oam[index + 1]])
    }

    fn set_oam_word(&mut self, row: usize, word: usize, value: u16) {
        let index = row * OAM_ROW_SIZE + word * OAM_WORD_SIZE;
        let [low, high] = value.to_le_bytes();

        self.oam[index] = low;
        self.oam[index + 1] = high;
    }

    fn copy_oam_row(&mut self, source: usize, destination: usize) {
        let source_start = source * OAM_ROW_SIZE;
        let destination_start = destination * OAM_ROW_SIZE;

        let mut row = [0; OAM_ROW_SIZE];
        row.copy_from_slice(&self.oam[source_start..source_start + OAM_ROW_SIZE]);

        self.oam[destination_start..destination_start + OAM_ROW_SIZE].copy_from_slice(&row);
    }

    fn corrupt_oam_write(&mut self) {
        let row = self.get_oam_row();

        if row == 0 || row >= OAM_ROW_COUNT {
            return;
        }

        let previous_row = row - 1;

        let a = self.get_oam_word(row, 0);
        let b = self.get_oam_word(previous_row, 0);
        let c = self.get_oam_word(previous_row, 2);

        let corrupted_word = ((a ^ c) & (b ^ c)) ^ c;
        self.set_oam_word(row, 0, corrupted_word);

        let source_start = previous_row * OAM_ROW_SIZE + OAM_WORD_SIZE;
        let destination_start = row * OAM_ROW_SIZE + OAM_WORD_SIZE;

        self.oam.copy_within(
            source_start..source_start + OAM_ROW_SIZE - OAM_WORD_SIZE,
            destination_start,
        );
    }

    fn corrupt_oam_read(&mut self) {
        let row = self.get_oam_row();

        if row == 0 || row >= OAM_ROW_COUNT {
            return;
        }

        let previous_row = row - 1;

        let a = self.get_oam_word(row, 0);
        let b = self.get_oam_word(previous_row, 0);
        let c = self.get_oam_word(previous_row, 2);

        self.set_oam_word(row, 0, b | (a & c));

        let source_start = previous_row * OAM_ROW_SIZE + OAM_WORD_SIZE;
        let destination_start = row * OAM_ROW_SIZE + OAM_WORD_SIZE;

        self.oam.copy_within(
            source_start..source_start + OAM_ROW_SIZE - OAM_WORD_SIZE,
            destination_start,
        );
    }

    fn corrupt_oam_read_write(&mut self) {
        let row = self.get_oam_row();

        if (4..OAM_ROW_COUNT - 1).contains(&row) {
            let previous_row = row - 1;
            let two_rows_before = row - 2;

            let a = self.get_oam_word(two_rows_before, 0);
            let b = self.get_oam_word(previous_row, 0);
            let c = self.get_oam_word(row, 0);
            let d = self.get_oam_word(previous_row, 2);

            let corrupted_word = (b & (a | c | d)) | (a & c & d);

            self.set_oam_word(previous_row, 0, corrupted_word);

            self.copy_oam_row(previous_row, two_rows_before);
            self.copy_oam_row(previous_row, row);
        }

        self.corrupt_oam_read();
    }
}
