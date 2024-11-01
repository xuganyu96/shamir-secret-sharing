#![no_std]
#![allow(non_camel_case_types)]

pub type Word = u16;

/// Carryless multiplciation of words
/// e.g. mul(0b1111, 0b1111) = 15 * 15 = 225 = 0b11100001
///     clmul(0b1111, 0b1111) = 0b1010101
/// TODO: this is not constant time!
pub fn widening_clmul(a: Word, b: Word) -> (Word, Word) {
    let mut prod: (Word, Word) = (0, 0);

    for i in 0..(Word::BITS) {
        if ((1 << i) & b) != 0 {
            // Need to "widening left shift" a by i positions
            let (mut high_bits, overflow) = a.overflowing_shr(Word::BITS - i);
            if overflow {
                high_bits = 0;
            }
            let (mut low_bits, overflow) = a.overflowing_shl(i);
            if overflow {
                low_bits = 0;
            }
            prod = (prod.0 ^ high_bits, prod.1 ^ low_bits);
        }
    }

    return prod;
}

/// Limbs are organized in big-endian bytes order. The limb at smaller index encodes coefficients
/// at higher-power term
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ExtField2<const L: usize> {
    limbs: [Word; L],
}

impl<const L: usize> ExtField2<L> {
    pub const ZERO: Self = Self::zero();
    pub const ONE: Self = Self::one();
    pub const BITS: usize = (Word::BITS as usize) * L;

    pub const fn as_limbs(&self) -> &[Word] {
        &self.limbs
    }

    pub const fn from_limbs(limbs: [Word; L]) -> Self {
        Self { limbs }
    }

    pub const fn zero() -> Self {
        Self { limbs: [0; L] }
    }

    pub const fn one() -> Self {
        let mut limbs = [0; L];
        limbs[L - 1] = 1;

        Self::from_limbs(limbs)
    }

    /// Equivalent to applying the bitflip operator "!"
    pub fn not(&self) -> Self {
        let mut output = Self::ZERO;

        self.as_limbs()
            .iter()
            .enumerate()
            .for_each(|(i, limb)| output.limbs[i] = !limb);

        return output;
    }

    /// Addition in GF(2^m) is a simple XOR and will never overflow
    #[allow(unused_variables)]
    pub fn gf_add(&self, other: &Self) -> Self {
        let mut limbs = [0; L];

        for i in 0..L {
            // No need for bound check; guaranteed to be within bounds.
            limbs[i] = self.as_limbs()[i] ^ other.as_limbs()[i];
        }

        return Self::from_limbs(limbs);
    }

    /// Subtraction is identical to addition in GF(2^m) because -1 = 1
    pub fn gf_sub(&self, other: &Self) -> Self {
        self.gf_add(other)
    }

    /// School book multiplication with L^2 steps
    pub fn widening_gf_mul(&self, other: &Self) -> (Self, Self) {
        let (mut high, mut low) = (Self::ZERO, Self::ZERO);
        for i in 0..L {
            for j in 0..L {
                let (high_limb, low_limb) =
                    widening_clmul(self.limbs[L - i - 1], other.limbs[L - j - 1]);
                if (i + j) < L {
                    low.limbs[L - (i + j) - 1] ^= low_limb;
                } else {
                    high.limbs[L - (i + j - L) - 1] ^= low_limb;
                }
                if (i + j + 1) < L {
                    low.limbs[L - (i + j + 1) - 1] ^= high_limb;
                } else {
                    high.limbs[L - (i + j + 1 - L) - 1] ^= high_limb;
                }
            }
        }

        return (high, low);
    }

    /// modulus multiplication
    #[allow(unused_variables)]
    pub fn gf_modmul(&self, other: &Self, modulus: &Self) -> Self {
        todo!();
    }

    /// Attempt to left shift (e.g. 0xFFFF.overflowing_shl(4) = 0xFFF0)
    /// If the shift amount is greater than there are bits in the
    #[allow(unused_variables)]
    pub fn overflowing_shl(&self, rhs: usize) -> (Self, bool) {
        todo!();
    }

    #[allow(unused_variables)]
    pub fn shr(&self, rhs: usize) -> Self {
        todo!();
    }
}

// TODO: what if I need to implement GF(2^12), such as in classic McEliece
pub type GF_2_16 = ExtField2<1>;
pub type GF_2_128 = ExtField2<8>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widening_clmul() {
        assert_eq!(widening_clmul(15, 15), (0, 0b1010101));
        assert_eq!(widening_clmul(0xFFFF, 0xFFFF), (0x5555, 0x5555));
        assert_eq!(widening_clmul(0xE223, 0x672F), (0x267B, 0xB291));
        assert_eq!(widening_clmul(0, 0), (0, 0));
        assert_eq!(widening_clmul(0, 1), (0, 0));
        assert_eq!(widening_clmul(1, 0), (0, 0));
    }

    #[test]
    fn test_extfield_widening_mul() {
        assert_eq!(
            GF_2_128::ZERO.not().widening_gf_mul(&GF_2_128::ZERO),
            (GF_2_128::ZERO, GF_2_128::ZERO)
        );
        // 0xFFFF * 0xFFFF = 0x5555,0x5555
        let fives = GF_2_128::from_limbs([0x5555; 8]);
        assert_eq!(
            GF_2_128::ZERO.not().widening_gf_mul(&GF_2_128::ZERO.not()),
            (fives, fives)
        );

        // Random cases generated by SymPy
        let lhs = GF_2_128::from_limbs([
            0x3DCC, 0x5CE2, 0x8A9D, 0x3FE3, 0x5309, 0x07F3, 0xC9FD, 0x43B6,
        ]);
        let rhs = GF_2_128::from_limbs([
            0x8370, 0x7DA9, 0x108D, 0xF5B7, 0x30C9, 0xAEB8, 0x719A, 0xEDB5,
        ]);
        let prod = (
            GF_2_128::from_limbs([
                0x1EAB, 0x66E7, 0x4160, 0x869E, 0xA3A7, 0x038E, 0x03AB, 0x25BF,
            ]),
            GF_2_128::from_limbs([
                0x77C9, 0xE332, 0x2107, 0x2707, 0x8AFD, 0x8E14, 0xE779, 0x45CE,
            ]),
        );
        assert_eq!(lhs.widening_gf_mul(&rhs), prod);

        let lhs = GF_2_128::from_limbs([
            0x102D, 0x2BD4, 0x66AC, 0xBCB1, 0xF7C7, 0x5FE9, 0xBBC2, 0x335D,
        ]);
        let rhs = GF_2_128::from_limbs([
            0xEB90, 0xC40B, 0xFD14, 0xE019, 0xDFC5, 0xE087, 0x23EF, 0xA19F,
        ]);
        let prod = (
            GF_2_128::from_limbs([
                0x0EA0, 0x7C5D, 0xFBA7, 0x0792, 0x1B33, 0x323D, 0xE533, 0x6BF7,
            ]),
            GF_2_128::from_limbs([
                0x19B6, 0xA88E, 0x62E5, 0xBB2E, 0x06AF, 0xAB14, 0x6A88, 0xE42B,
            ]),
        );
        assert_eq!(lhs.widening_gf_mul(&rhs), prod);

        let lhs = GF_2_128::from_limbs([
            0xA95D, 0x01B0, 0xD0A6, 0x81A9, 0x92A5, 0xA216, 0xC971, 0x961A,
        ]);
        let rhs = GF_2_128::from_limbs([
            0x0817, 0x2EE5, 0xB309, 0x150F, 0x2BF1, 0x5A62, 0x2197, 0xB1C8,
        ]);
        let prod = (
            GF_2_128::from_limbs([
                0x0543, 0x30B1, 0x8B03, 0x4A8E, 0x43F7, 0x29DB, 0x10A6, 0xBCB3,
            ]),
            GF_2_128::from_limbs([
                0x7D9B, 0x512D, 0x94F5, 0x12B3, 0x8D64, 0xE68F, 0xEAFD, 0xC150,
            ]),
        );
        assert_eq!(lhs.widening_gf_mul(&rhs), prod);
    }
}