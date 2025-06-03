/// A helper method to check that `Option<String>` value is blank.
pub fn is_blank(value: &Option<String>) -> bool {
    if let Some(value) = value {
        if value.trim().is_empty() {
            return true;
        }
    }
    false
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
    if !found { Err(()) } else { Ok(count == 1) }
}
