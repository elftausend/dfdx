use crate::prelude::*;

/// Average the values along dimension `I` of `T`.
///
/// **Pytorch equivalent**: `t.mean(I)`
///
/// Examples:
/// ```rust
/// # use dfdx::prelude::*;
/// let t: Tensor2D<2, 3> = Tensor2D::new([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]);
/// let r: Tensor1D<2> = t.mean_axis::<-1>();
/// assert_eq!(r.data(), &[2.0, 5.0]);
/// ```
pub fn mean_axis<T: Reduce1<I>, const I: isize>(t: T) -> T::Reduced
where
    T::Array: HasAxis<I>,
{
    div_scalar(sum_axis::<T, I>(t), <T::Array as HasAxis<I>>::SIZE as f32)
}

macro_rules! mean_axis_impl {
    ($typename:ident, [$($Vs:tt),*]) => {
impl<$(const $Vs: usize, )* H: Tape> $typename<$($Vs, )* H> {
    /// Calls [mean_axis()] on `self`.
    pub fn mean_axis<const I: isize>(self) -> <Self as Reduce1<I>>::Reduced
    where
        Self: Reduce1<I>,
        <Self as HasArrayType>::Array: HasAxis<I>,
    {
        mean_axis::<Self, I>(self)
    }
}
    };
}

mean_axis_impl!(Tensor0D, []);
mean_axis_impl!(Tensor1D, [M]);
mean_axis_impl!(Tensor2D, [M, N]);
mean_axis_impl!(Tensor3D, [M, N, O]);
mean_axis_impl!(Tensor4D, [M, N, O, P]);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valids_mean_axis() {
        let _: Tensor0D = Tensor1D::<5>::zeros().mean_axis::<-1>();

        let _: Tensor1D<3> = Tensor2D::<5, 3>::zeros().mean_axis::<0>();
        let _: Tensor1D<5> = Tensor2D::<5, 3>::zeros().mean_axis::<-1>();

        let _: Tensor2D<5, 3> = Tensor3D::<7, 5, 3>::zeros().mean_axis::<0>();
        let _: Tensor2D<7, 3> = Tensor3D::<7, 5, 3>::zeros().mean_axis::<1>();
        let _: Tensor2D<7, 5> = Tensor3D::<7, 5, 3>::zeros().mean_axis::<-1>();

        let _: Tensor3D<7, 5, 3> = Tensor4D::<9, 7, 5, 3>::zeros().mean_axis::<0>();
        let _: Tensor3D<9, 5, 3> = Tensor4D::<9, 7, 5, 3>::zeros().mean_axis::<1>();
        let _: Tensor3D<9, 7, 3> = Tensor4D::<9, 7, 5, 3>::zeros().mean_axis::<2>();
        let _: Tensor3D<9, 7, 5> = Tensor4D::<9, 7, 5, 3>::zeros().mean_axis::<-1>();
    }

    #[test]
    fn test_mean_axis_0_2d() {
        let t: Tensor2D<2, 3> = Tensor2D::new([[1.0, 2.0, 3.0], [-2.0, 4.0, -6.0]]);
        let r = t.trace().mean_axis::<0>();
        assert_eq!(r.data(), &[-0.5, 3.0, -1.5]);
        let gradients = r.exp().mean().backward();
        assert_eq!(
            gradients.ref_gradient(&t),
            &[[0.10108845, 3.3475895, 0.037188362]; 2]
        );
    }

    #[test]
    fn test_mean_axis_1_2d() {
        let t: Tensor2D<2, 3> = Tensor2D::new([[1.0, 2.0, 3.0], [-2.0, 4.0, -6.0]]);
        let r = t.trace().mean_axis::<-1>();
        assert_eq!(r.data(), &[2.0, -4.0 / 3.0]);
        let gradients = r.exp().mean().backward();
        assert_eq!(
            gradients.ref_gradient(&t),
            &[[1.2315093; 3], [0.043932855; 3]]
        );
    }
}
