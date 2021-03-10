use std::marker::PhantomData;

use serde::{
    de::{self, Error, SeqAccess, Visitor},
    ser::SerializeSeq,
    Deserialize, Deserializer, Serialize, Serializer,
};

use crate::matrix::Matrix;

impl<T, const R: usize, const C: usize> Serialize for Matrix<T, R, C>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(R * C))?;
        for element in self.data.iter().flatten() {
            seq.serialize_element(element)?;
        }
        seq.end()
    }
}

impl<'de, T, const R: usize, const C: usize> Deserialize<'de> for Matrix<T, R, C>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(MatrixVisitor::new())
    }
}

struct MatrixVisitor<T, const R: usize, const C: usize> {
    _marker: PhantomData<T>,
}

impl<T, const R: usize, const C: usize> MatrixVisitor<T, R, C> {
    fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<'de, T, const R: usize, const C: usize> Visitor<'de> for MatrixVisitor<T, R, C>
where
    T: Deserialize<'de>,
{
    type Value = Matrix<T, R, C>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(format!("an {}x{} matrix", C, R).as_str())
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let size_expected: usize = R * C;

        if let Some(size_hint) = seq.size_hint() {
            if size_hint != R * C {
                return Err(A::Error::custom(format!(
                    "invalid size for matrix (expected: {}, got: {})",
                    size_expected, size_hint
                )));
            }
        }

        let mut matrix: Matrix<T, R, C> = unsafe { Matrix::uninitialized() };

        for col in 0..C {
            for row in 0..R {
                unsafe {
                    *matrix.get_unchecked_mut((row, col)) =
                        seq.next_element()?.ok_or(A::Error::custom(format!(
                            "invalid size for matrix (expected: {}, got: {})",
                            size_expected,
                            row * col
                        )))?;
                };
            }
        }

        Ok(matrix)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_util::MatrixCmp;

    use super::*;
    use crate::mat4::Mat4;
    use crate::vector::Vec3;

    #[test]
    fn mat4_serde_ident() -> Result<(), Box<dyn std::error::Error>> {
        let mat4: Mat4<f32> = Mat4::identity();

        let encoded = bincode::serialize(&mat4)?;

        println!("{:?}", encoded);

        let decoded = bincode::deserialize(&encoded[..])?;

        MatrixCmp::<f32>::DEFAULT.eq(&mat4, &decoded);

        Ok(())
    }

    #[test]
    fn vec3_serde_ident() -> Result<(), Box<dyn std::error::Error>> {
        let vec3: Vec3<f32> = Vec3::unit_x();

        let encoded = bincode::serialize(&vec3)?;

        println!("{:?}", encoded);

        let decoded = bincode::deserialize(&encoded[..])?;

        MatrixCmp::<f32>::DEFAULT.eq(&vec3, &decoded);

        Ok(())
    }
}
