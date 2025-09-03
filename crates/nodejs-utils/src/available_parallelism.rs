pub const HEROKU_AVAILABLE_PARALLELISM: &str = "HEROKU_AVAILABLE_PARALLELISM";

#[must_use]
pub fn available_parallelism_env() -> (String, String) {
    (
        HEROKU_AVAILABLE_PARALLELISM.to_string(),
        std::thread::available_parallelism()
            // XXX: The Rust implementation always rounds down the value reported here if the
            //      (quota / period) calculated from cgroups cpu.max produces a fractional value.
            //      For Heroku Fir Dynos this will always end up reducing the cpu allocation
            //      value by 1 since a small amount of quota is reserved for the system so we need
            //      to add that back unless Rust changes how they deal with rounding.
            .map(|value| (value.get() + 1).to_string())
            .unwrap_or_default(),
    )
}
