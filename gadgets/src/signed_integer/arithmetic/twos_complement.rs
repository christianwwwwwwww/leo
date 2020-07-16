use crate::{binary::RippleCarryAdder, errors::SignedIntegerError, signed_integer::*};

use snarkos_models::{
    curves::PrimeField,
    gadgets::{r1cs::ConstraintSystem, utilities::boolean::Boolean},
};

/// Inverts the given number and adds 1 to the lsb of the result
pub trait TwosComplement
where
    Self: std::marker::Sized,
{
    #[must_use]
    fn twos_comp<F: PrimeField, CS: ConstraintSystem<F>>(&self, cs: CS) -> Result<Self, SignedIntegerError>;
}

impl TwosComplement for Vec<Boolean> {
    fn twos_comp<F: PrimeField, CS: ConstraintSystem<F>>(&self, mut cs: CS) -> Result<Self, SignedIntegerError> {
        // flip all bits
        let flipped: Self = self.iter().map(|bit| bit.not()).collect();

        // add one
        let mut one = vec![Boolean::constant(true)];
        one.append(&mut vec![Boolean::Constant(false); self.len() - 1]);

        let mut bits = flipped.add_bits(cs.ns(|| format!("add one")), &one)?;
        let _carry = bits.pop(); // we already accounted for overflow above

        Ok(bits)
    }
}

macro_rules! twos_comp_int_impl {
    ($($gadget: ident)*) => ($(
        impl TwosComplement for $gadget {
            fn twos_comp<F: PrimeField, CS: ConstraintSystem<F>>(
                &self,
                cs: CS
            ) -> Result<Self, SignedIntegerError> {
                let value = match self.value {
                    Some(val) => {
                        match val.checked_neg() {
                            Some(val_neg) => Some(val_neg),
                            None => return Err(SignedIntegerError::Overflow) // -0 should fail
                        }
                    }
                    None => None,
                };

                // calculate two's complement
                let bits = self.bits.twos_comp(cs)?;

                Ok(Self {
                    bits,
                    value,
                })
            }
        }
    )*)
}

twos_comp_int_impl!(Int8 Int16 Int32 Int64 Int128);
