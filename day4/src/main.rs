use std::env;

fn main() {
    let range = env::args().nth(1).expect("Expected range argument.");
    let mut range = range.split('-');
    let lower_bound = range
        .next()
        .expect("Malformed range.")
        .parse::<i32>()
        .expect("Lower bound not a number.");
    let upper_bound = range
        .next()
        .expect("Malformed range.")
        .parse::<i32>()
        .expect("Lower bound not a number.");

    let possible = (lower_bound..upper_bound + 1)
        .filter(is_password_compatible)
        .count();
    let possible_additional = (lower_bound..upper_bound + 1)
        .filter(is_password_additionally_compatible)
        .count();
    println!(
        "Possible passwords: {}\nPossible additional passwords: {}",
        possible, possible_additional
    );
}

fn is_password_compatible(n: &i32) -> bool {
    let digits = to_digits(*n);

    digits[0] > 0
        && has_adjacent_same_digits(digits)
        && has_adjacent_same_digits(digits)
        && has_increasing_digits(digits)
}

fn is_password_additionally_compatible(n: &i32) -> bool {
    if !is_password_compatible(n) {
        return false;
    }
    let digits = to_digits(*n);

    (0..5).any(|i| {
        digits[i] == digits[i + 1]
            && (0..6)
                .filter(|n| *n != i && *n != i + 1)
                .all(|j| digits[i] != digits[j])
    })
}

fn has_adjacent_same_digits(digits: [u8; 6]) -> bool {
    (0..5).any(|i| digits[i] == digits[i + 1])
}

fn has_increasing_digits(digits: [u8; 6]) -> bool {
    (0..5).all(|i| digits[i] <= digits[i + 1])
}

fn to_digits(mut n: i32) -> [u8; 6] {
    let mut digits: [u8; 6] = [0; 6];

    for i in 0..6 {
        digits[5 - i] = (n % 10) as u8;
        n /= 10;
    }

    digits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_digits() {
        assert_eq!(to_digits(123456), [1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_is_password_compatible() {
        assert!(is_password_compatible(&111111));
        assert!(!is_password_compatible(&223450));
        assert!(!is_password_compatible(&123789));
    }

    #[test]
    fn test_is_password_additionally_compatible() {
        assert!(is_password_additionally_compatible(&112233));
        assert!(!is_password_additionally_compatible(&123444));
        assert!(is_password_additionally_compatible(&111122));
    }
}
