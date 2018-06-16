use super::traits::{ICurl, HASH_LENGTH};
use num_bigint::{BigInt, Sign};
use num_integer::Integer;
use num_traits::pow;
use num_traits::{Signed, ToPrimitive, Zero};
use tiny_keccak::Keccak;
use utils::converter;

const BIT_HASH_LENGTH: usize = 384;
const BYTE_HASH_LENGTH: usize = BIT_HASH_LENGTH / 8;
const MAX_POWERS_LONG: usize = 40;

const BYTE_LENGTH: usize = 48;
const INT_LENGTH: usize = BYTE_LENGTH / 4;

lazy_static! {
    pub static ref MAX_POWERS: Vec<BigInt> = {
        let mut results = vec![BigInt::from(0); MAX_POWERS_LONG + 1];
        let radix = BigInt::from(converter::RADIX);
        for (i, result) in results.iter_mut().enumerate().take(MAX_POWERS_LONG + 1) {
            *result = pow(radix.clone(), i);
        }
        results
    };
}

#[derive(Clone)]
pub struct Kerl {
    keccak: Keccak,
}

impl Default for Kerl {
    fn default() -> Kerl {
        Kerl {
            keccak: Keccak::new_keccak384(),
        }
    }
}

impl ICurl for Kerl {
    fn absorb(&mut self, trits: &mut [i8]) {
        assert!(trits.len() % HASH_LENGTH == 0);
        let mut pos = 0;
        while pos < trits.len() {
            let mut bytes = [0; BYTE_HASH_LENGTH];
            trits[pos + HASH_LENGTH - 1] = 0;
            bytes_from_big_int(&big_int_from_trits(trits, pos, HASH_LENGTH), &mut bytes);
            self.keccak.update(&bytes);
            pos += HASH_LENGTH;
        }
    }

    fn squeeze(&mut self, trits: &mut [i8]) {
        assert!(trits.len() % HASH_LENGTH == 0);
        let mut pos = 0;
        while pos < trits.len() {
            let mut state = [0; BYTE_HASH_LENGTH];
            self.keccak.pad();
            self.keccak.fill_block();
            self.keccak.squeeze(&mut state);
            self.keccak = Keccak::new_keccak384();
            let value = BigInt::from_signed_bytes_be(&state);
            trits_from_big_int(&value, trits, pos, HASH_LENGTH);
            trits[pos + HASH_LENGTH - 1] = 0;
            for b in state.iter_mut() {
                *b ^= 0xFF;
            }
            self.keccak.update(&state);
            pos += HASH_LENGTH;
        }
    }
}

impl Kerl {
    pub fn reset(&mut self) {
        self.keccak = Keccak::new_keccak384();
    }
}

#[derive(Debug, Fail)]
enum BigIntConversionError {
    #[fail(display = "invalid trit value [{}] encountered at index: {}", value, index)]
    InvalidTritArray { value: i8, index: usize },
}

fn big_int_from_trits(trits: &[i8], offset: usize, size: usize) -> BigInt {
    for trit in trits.iter().take(offset + size).skip(offset) {
        assert!(*trit >= -1 && *trit <= 1);
    }
    let mut value = BigInt::zero();
    let mut n = offset + size - 1;

    let mut count: usize;
    let mut num: BigInt;
    while n >= offset {
        count = 0;
        num = BigInt::zero();
        while n >= offset && count < MAX_POWERS_LONG {
            num = 3 * num + trits[n];
            count += 1;
            if n == 0 {
                break;
            }
            n -= 1;
        }
        value = value * &MAX_POWERS[count] + &num;
        if n == 0 {
            break;
        }
    }
    value
}

fn trits_from_big_int(value: &BigInt, destination: &mut [i8], offset: usize, size: usize) {
    assert!(destination.len() - offset >= size);
    if *value == BigInt::zero() {
        for entry in destination[offset..size].iter_mut() {
            *entry = 0;
        }
    }
    let sign = value.sign();
    let mut absolute = value.abs();
    let radix = BigInt::from(converter::RADIX);
    for i in 0..size {
        let div_remainder = absolute.div_rem(&radix);
        absolute = div_remainder.0;
        let mut remainder = div_remainder.1.to_i32().unwrap();
        if remainder > i32::from(converter::MAX_TRIT_VALUE) {
            remainder = i32::from(converter::MIN_TRIT_VALUE);
            absolute += 1;
        }
        destination[offset + i] = if sign == Sign::Minus {
            -remainder as i8
        } else {
            remainder as i8
        }
    }
}

fn bytes_from_big_int(value: &BigInt, destination: &mut [u8]) {
    assert!(destination.len() >= BYTE_HASH_LENGTH);
    let bytes = value.to_signed_bytes_be();
    let mut start = BYTE_HASH_LENGTH - bytes.len();
    let sign: u8 = if value.sign() == Sign::Minus { 255 } else { 0 };
    destination[0..start].copy_from_slice(&vec![sign; start]);
    for byte in &bytes {
        destination[start] = *byte;
        start += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use utils::converter;

    #[test]
    fn test_big_int_from_trits() {
        let value = 1433452143;
        let size = 50;
        let mut trits = vec![0; size];
        let len = trits.len();
        converter::copy_trits(value, &mut trits, 0, len);
        let len = trits.len();
        let big = big_int_from_trits(&mut trits, 0, len);
        let mut out = vec![0; size];
        trits_from_big_int(&big, &mut out, 0, size);
        assert_eq!(trits, out);
    }

    #[test]
    fn test_bytes_from_big_int() {
        let byte_size = 48;
        let big_int: BigInt = "13190295509826637194583200125168488859623001289643321872497025844241981297292953903419783680940401133507992851240799".parse().unwrap();
        let mut out_bytes = vec![0; byte_size];
        bytes_from_big_int(&big_int, &mut out_bytes);
        let out_big_int = BigInt::from_signed_bytes_be(&out_bytes);
        assert_eq!(big_int, out_big_int);
    }

    #[test]
    fn test_kerl_one_absorb() {
        let mut initial = converter::trits_from_string(
            "EMIDYNHBWMBCXVDEFOFWINXTERALUKYYPPHKP9JJFGJEIUY9MUDVNFZHMMWZUYUSWAIOWEVTHNWMHANBH",
        );
        let mut k = Kerl::default();
        k.absorb(&mut initial);
        let mut hash_value = vec![0; HASH_LENGTH];
        k.squeeze(&mut hash_value);
        let hash = converter::trytes(&hash_value);
        assert_eq!(
            hash,
            "EJEAOOZYSAWFPZQESYDHZCGYNSTWXUMVJOVDWUNZJXDGWCLUFGIMZRMGCAZGKNPLBRLGUNYWKLJTYEAQX"
        );
    }

    #[test]
    fn test_kerl_multi_squeeze() {
        let mut initial = converter::trits_from_string(
            "9MIDYNHBWMBCXVDEFOFWINXTERALUKYYPPHKP9JJFGJEIUY9MUDVNFZHMMWZUYUSWAIOWEVTHNWMHANBH",
        );
        let mut k = Kerl::default();
        k.absorb(&mut initial);
        let mut hash_value = vec![0; HASH_LENGTH * 2];
        k.squeeze(&mut hash_value);
        let hash = converter::trytes(&hash_value);
        assert_eq!(
            hash,
            "G9JYBOMPUXHYHKSNRNMMSSZCSHOFYOYNZRSZMAAYWDYEIMVVOGKPJBVBM9TDPULSFUNMTVXRKFIDOHUXXVYDLFSZYZTWQYTE9SPYYWYTXJYQ9IFGYOLZXWZBKWZN9QOOTBQMWMUBLEWUEEASRHRTNIQWJQNDWRYLCA"
        );
    }

    #[test]
    fn test_kerl_multi_absorb_multi_squeeze() {
        let mut initial = converter::trits_from_string(
            "G9JYBOMPUXHYHKSNRNMMSSZCSHOFYOYNZRSZMAAYWDYEIMVVOGKPJBVBM9TDPULSFUNMTVXRKFIDOHUXXVYDLFSZYZTWQYTE9SPYYWYTXJYQ9IFGYOLZXWZBKWZN9QOOTBQMWMUBLEWUEEASRHRTNIQWJQNDWRYLCA",
        );
        let mut k = Kerl::default();
        k.absorb(&mut initial);
        let mut hash_value = vec![0; HASH_LENGTH * 2];
        k.squeeze(&mut hash_value);
        let hash = converter::trytes(&hash_value);
        assert_eq!(
            hash,
            "LUCKQVACOGBFYSPPVSSOXJEKNSQQRQKPZC9NXFSMQNRQCGGUL9OHVVKBDSKEQEBKXRNUJSRXYVHJTXBPDWQGNSCDCBAIRHAQCOWZEBSNHIJIGPZQITIBJQ9LNTDIBTCQ9EUWKHFLGFUVGGUWJONK9GBCDUIMAYMMQX"
        );
    }
}
