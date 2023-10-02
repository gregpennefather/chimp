pub fn count_rooks_on_open_file(rooks: u64, open_files: u64) -> i16 {
    (rooks & open_files).count_ones() as i16
}