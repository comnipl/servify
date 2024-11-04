pub(super) const ERR_MULTIPLE_FN_IN_EXPORT: &str =
    "Within the servify::export macro, only one function can be exported in one impl block";

pub(super) const ERR_NO_FN_IN_EXPORT: &str =
    "Within the servify::export macro, one function must be exported in one impl block";

pub(super) const ERR_UNEXPECTED_ITEM_IN_EXPORT: &str =
    "Supported items within the servify::export macro are only exporting function, associated function, associated constants, and Request struct";
