//! # Utilities
//!
//! This crate includes helper utilities.

/// Converts a hex string to an [u8] array. Encoding is done by converting hex
/// value to digit value and packing 2 digits together.
///
/// # Parameters
///
/// - `hex`: Hex encoded string with no prefixes nor suffixes
/// - `output`: Mutable array that will hold encoded data
///
/// # Examples
///
/// ```ignore
/// let mut hex: [u8; 1] = [0; 1];
/// hex_to_array("FF", &mut hex);
/// assert_eq!(hex, [255]);
/// ```
///
/// # Panics
///
/// Will panic if input `hex` length is more than 2 times of `output` length.
pub fn hex_to_array(hex: &str, output: &mut [u8]) {
    // Clean output.
    for i in 0..output.len() {
        output[i] = 0;
    }

    let len = hex.len();

    hex.chars().enumerate().for_each(|(idx, char)| {
        output[idx / 2] += if idx % 2 == 0 && idx + 1 != len {
            char.to_digit(16).unwrap() as u8 * 16
        } else {
            char.to_digit(16).unwrap() as u8
        };
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn hex_to_array() {
        let mut hex: [u8; 1] = [0; 1];
        super::hex_to_array("F", &mut hex);
        assert_eq!(hex, [15]);

        let mut hex: [u8; 2] = [0; 2];
        super::hex_to_array("1234", &mut hex);
        assert_eq!(hex, [18, 52]);

        let mut hex: [u8; 2] = [0; 2];
        super::hex_to_array("ABCD", &mut hex);
        assert_eq!(hex, [171, 205]);

        let mut hex: [u8; 4] = [0; 4];
        super::hex_to_array("B00B1E5", &mut hex);
        assert_eq!(hex, [176, 11, 30, 5]);

        // Dirty input data.
        let mut hex: [u8; 4] = [0x45; 4];
        super::hex_to_array("B00B1E5", &mut hex);
        assert_eq!(hex, [176, 11, 30, 5]);
    }
}
