use num_rational::Rational64;
use num_traits::{FromPrimitive, ToPrimitive};

/// Make a trimmed copy of the provided `String`.
pub fn take_if_not_blank(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Clone the contents of the given slice into a new vector while performing type conversion.
pub fn clone_into_vec<A: Clone + Into<B>, B>(data: &[A]) -> Vec<B> {
    data.iter().cloned().map(std::convert::Into::into).collect()
}

/// Convert `Rational64` to `f64`, or `0.0` if the conversion fails.
pub fn f64_or_default(rational: Rational64) -> f64 {
    rational.to_f64().unwrap_or_default()
}

/// Convert `f64` to `Rational64`, or `0.0` if the conversion fails.
pub fn rational_or_default(rational: f64) -> Rational64 {
    Rational64::from_f64(rational).unwrap_or_default()
}

/// A helper method to check that a given `container` has the expected value, and it is the
/// only value with such ID in the container.
///
/// The method returns `Err(())` when the `value` is not in the `container`. If the `value` is in
/// the container, the method either returns `Ok(true)` (the value is unique), or `Ok(false)`
/// (the value is not unique).
pub fn is_unique_id<T: Eq, ID: Eq, F>(container: &[T], value: &T, id: F) -> Result<bool, ()>
where
    F: Fn(&T) -> ID,
{
    let check_id = id(value);
    let mut found = false;
    let mut count = 0u32;
    for item in container {
        if value == item {
            found = true;
        }
        if id(item) == check_id {
            count += 1;
        }
    }
    if found { Ok(count == 1) } else { Err(()) }
}
