pub(crate) struct Day {
    pub(crate) number: u8,
    pub(crate) part1: fn(&str) -> color_eyre::Result<String>,
    pub(crate) part2: fn(&str) -> color_eyre::Result<String>,
}
