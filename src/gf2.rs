//! Galois Field of order 2^m
use crate::f2x::{F2x, WideF2x};
use rand::Rng;

/// An algebraic field is defined by 0, 1, addition, and multiplication. Every non-zero element
/// should have a multiplicative inverse.
pub trait FieldArithmetic: Sized + Clone {
    fn is_zero(&self) -> bool;
    fn is_one(&self) -> bool;
    fn zero() -> Self;
    fn one() -> Self;
    fn random() -> Self;
    fn modadd(&self, rhs: &Self) -> Self;
    fn modsub(&self, rhs: &Self) -> Self;
    fn modmul(&self, rhs: &Self) -> Self;
    fn modinv(&self) -> Option<Self>;
    // TODO: I don't need modular exponentiation yet, but it is common
    // fn modexp(&self, exp: usize) -> Self;
}

macro_rules! galois_field {
    ($name:ident, $limbs:literal, $irreducible:expr) => {
        /// An element of the binary extension field with the specified exponent
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        pub struct $name {
            pub poly: F2x<{ Self::LIMBS }>,
        }

        impl $name {
            pub const LIMBS: usize = $limbs;
            pub const MODULUS: WideF2x<{ Self::LIMBS }> = $irreducible;
            pub const ONE: Self = Self::from_poly(F2x::<{ Self::LIMBS }>::ONE);
            pub const ZERO: Self = Self::from_poly(F2x::<{ Self::LIMBS }>::ZERO);

            /// Sample a random element
            pub fn random() -> Self {
                let mut limbs = [0; Self::LIMBS];
                let mut rng = rand::thread_rng();
                rng.fill(&mut limbs);
                let poly = F2x::from_limbs(limbs);
                Self { poly }
            }

            pub const fn from_poly(poly: F2x<{ Self::LIMBS }>) -> Self {
                Self { poly }
            }

            pub fn add(&self, rhs: &Self) -> Self {
                Self::from_poly(self.poly.add(&rhs.poly))
            }

            pub fn sub(&self, rhs: &Self) -> Self {
                self.add(rhs)
            }

            pub fn mul(&self, rhs: &Self) -> Self {
                let prod = self.poly.widening_mul(&rhs.poly);
                let (_, rem) = prod.div_rem(&Self::MODULUS);
                Self::from_poly(rem.truncate())
            }

            /// Compute the multiplicative inverse under the modulus. Will return None if self is
            /// not invertible
            pub fn inv(&self) -> Option<Self> {
                let inverse = self.poly.modinv(&Self::MODULUS);
                inverse.map_or(None, |poly| Some(Self::from_poly(poly)))
            }
        }
    };
}

galois_field!(
    GF2p128,
    8,
    // x^128 + x^77 + x^35 + x^11 + 1
    WideF2x::from_f2x(
        F2x::<8>::ONE,
        F2x::<8>::from_limbs([0x0000, 0x0000, 0x0000, 0x2000, 0x0000, 0x0008, 0x0000, 0x0801,]),
    )
);

galois_field!(
    GF2p192,
    12,
    // x^192 + x^142 + x^103 + x^17 + 1
    WideF2x::from_f2x(
        F2x::<12>::ONE,
        F2x::<12>::from_limbs([
            0x0000, 0x0000, 0x0000, 0x4000, 0x0000, 0x0080, 0x0000, 0x0000, 0x0000, 0x0000, 0x0002,
            0x0001,
        ]),
    )
);

galois_field!(
    GF2p256,
    16,
    // x^256 + x^241 + x^178 + x^121 + 1
    WideF2x::from_f2x(
        F2x::<16>::ONE,
        F2x::<16>::from_limbs([
            0x0002, 0x0000, 0x0000, 0x0000, 0x0004, 0x0000, 0x0000, 0x0000, 0x0200, 0x0000, 0x0000,
            0x0000, 0x0000, 0x0000, 0x0000, 0x0001,
        ]),
    )
);

#[cfg(test)]
mod tests {
    use super::*;

    const NTESTS: usize = 10;

    #[test]
    fn test_gf2_128_mul() {
        let lhs = GF2p128::from_poly(F2x::<8>::from_limbs([
            0x1254, 0x4198, 0x8DA7, 0x29BD, 0xECF1, 0x64DE, 0xFBA7, 0xB692,
        ]));
        let rhs = GF2p128::from_poly(F2x::<8>::from_limbs([
            0x7D89, 0xD76A, 0x644E, 0x3A1C, 0x047C, 0xB60A, 0x1B98, 0x30F0,
        ]));
        let rem = GF2p128::from_poly(F2x::<8>::from_limbs([
            0x38B8, 0xB93F, 0xE30A, 0xE55E, 0xFC24, 0x2F4D, 0x5A16, 0xBD14,
        ]));
        assert_eq!(lhs.mul(&rhs), rem);

        let lhs = GF2p128::from_poly(F2x::<8>::from_limbs([
            0x5E17, 0xA183, 0x0FB7, 0xC7A9, 0xD3D3, 0xD1A4, 0x8962, 0xC3F5,
        ]));
        let rhs = GF2p128::from_poly(F2x::<8>::from_limbs([
            0x6ECC, 0x895B, 0x76CC, 0x855E, 0x9C14, 0xEF6F, 0x587A, 0x4A04,
        ]));
        let rem = GF2p128::from_poly(F2x::<8>::from_limbs([
            0xC42C, 0xBD01, 0xE0B0, 0x693D, 0x5E49, 0x6D28, 0xA69F, 0x701A,
        ]));
        assert_eq!(lhs.mul(&rhs), rem);

        let lhs = GF2p128::from_poly(F2x::<8>::from_limbs([
            0x6C70, 0x49B5, 0x97A4, 0x65D1, 0x2370, 0x8DBE, 0x127F, 0x5EFB,
        ]));
        let rhs = GF2p128::from_poly(F2x::<8>::from_limbs([
            0x3387, 0xF3D0, 0xBD53, 0xADF3, 0x2994, 0x3B7A, 0xB2A8, 0x974A,
        ]));
        let rem = GF2p128::from_poly(F2x::<8>::from_limbs([
            0x4402, 0xF342, 0x347A, 0xA630, 0x0B5D, 0x31BB, 0xBED8, 0x76A2,
        ]));
        assert_eq!(lhs.mul(&rhs), rem);

        let lhs = GF2p128::from_poly(F2x::<8>::from_limbs([
            0x98AA, 0xD35C, 0x02F5, 0x612C, 0x67A1, 0x9B5C, 0xF1FE, 0x98C7,
        ]));
        let rhs = GF2p128::from_poly(F2x::<8>::from_limbs([
            0x4F13, 0x9428, 0x6D75, 0x61C6, 0x2CE7, 0xA102, 0xB546, 0xC183,
        ]));
        let rem = GF2p128::from_poly(F2x::<8>::from_limbs([
            0x09D9, 0x9D26, 0x5665, 0x2DAE, 0x2A22, 0x9928, 0xC29C, 0xD153,
        ]));
        assert_eq!(lhs.mul(&rhs), rem);

        let lhs = GF2p128::from_poly(F2x::<8>::from_limbs([
            0x1EA8, 0xE4CA, 0x33E9, 0x7C78, 0xD1E4, 0x903F, 0x6F70, 0x20F8,
        ]));
        let rhs = GF2p128::from_poly(F2x::<8>::from_limbs([
            0xEA22, 0x6D4C, 0xC5D4, 0x0C82, 0xF584, 0x3185, 0x36F4, 0x12B2,
        ]));
        let rem = GF2p128::from_poly(F2x::<8>::from_limbs([
            0x8AF2, 0x5D11, 0xB56A, 0x68C0, 0xC3DD, 0xD9A1, 0xAC6E, 0x930B,
        ]));
        assert_eq!(lhs.mul(&rhs), rem);
    }

    #[test]
    fn random_gf2_128_inv() {
        for _ in 0..NTESTS {
            let elem = GF2p128::random();
            if elem.poly.is_zero() {
                assert!(elem.inv().is_none());
            } else {
                let inv = elem.inv().unwrap();
                assert_eq!(elem.mul(&inv), GF2p128::ONE);
            }
        }
    }

    #[test]
    fn random_gf2p192_inv() {
        for _ in 0..NTESTS {
            let elem = GF2p192::random();
            if elem.poly.is_zero() {
                assert!(elem.inv().is_none());
            } else {
                let inv = elem.inv().unwrap();
                assert_eq!(elem.mul(&inv), GF2p192::ONE);
            }
        }
    }

    #[test]
    fn random_gf2p256_inv() {
        for _ in 0..NTESTS {
            let elem = GF2p256::random();
            if elem.poly.is_zero() {
                assert!(elem.inv().is_none());
            } else {
                let inv = elem.inv().unwrap();
                assert_eq!(elem.mul(&inv), GF2p256::ONE);
            }
        }
    }
}