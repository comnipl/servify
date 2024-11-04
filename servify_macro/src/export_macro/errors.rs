pub(super) const ERR_MULTIPLE_FN_IN_EXPORT: &str =
    "Only one function can be exported in one impl block with servify::export";

pub(super) const ERR_NO_FN_IN_EXPORT: &str =
    "One function must be exported in one impl block with servify::export";

pub(super) const ERR_UNEXPECTED_ITEM_IN_EXPORT: &str =
    "Supported items are only exporting function, associated function, associated constants, and Request struct";
