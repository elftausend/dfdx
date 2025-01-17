use super::utils::move_tape_and_add_backward_op;
use crate::prelude::*;

/// Select values along a single axis `I` resulting in `T`. Equivalent
/// to `torch.select` and `torch.gather` from pytorch.
pub trait Select1<T, const I: isize> {
    type Indices: Clone;

    /// Select sub elements using [Self::Indices].
    /// The same element can be selected multiple times depending
    /// on [Self::Indices].
    fn select(self, indices: &Self::Indices) -> T;
}

macro_rules! impl_select {
    ($Axis:expr, $SrcTy:ty, $IndTy:tt, $DstTy:ty, {$($Dims:tt),*}) => {
impl<$(const $Dims: usize, )* H: Tape> Select1<$DstTy, $Axis> for $SrcTy {
    type Indices = $IndTy;
    fn select(self, indices: &Self::Indices) -> $DstTy {
        let mut result: <$DstTy as Tensor>::NoTape = TensorCreator::zeros();
        Cpu::select_axis(self.data(), indices, result.mut_data());

        #[allow(clippy::clone_on_copy)]
        let i = indices.clone();

        move_tape_and_add_backward_op(self, result, move |mut t, result, grads| {
            let (t_grad, result_grad) = grads.mut_and_ref(&t, &result);
            Cpu::fill(t.mut_data(), &mut |v| *v = 0.0);
            Cpu::select_add(t.mut_data(), &i, result_grad);
            Cpu::add(t_grad, t.data());
        })
    }
}
    };
}

// 1d
impl_select!(-1, Tensor1D<M, H>, usize, Tensor0D<H>, {M});
impl_select!(-1, Tensor1D<M, H>, [usize; Z], Tensor1D<Z, H>, {M, Z});

// 2d
impl_select!(0, Tensor2D<M, N, H>, usize, Tensor1D<N, H>, {M, N});
impl_select!(0, Tensor2D<M, N, H>, [usize; Z], Tensor2D<Z, N, H>, {M, N, Z});
impl_select!(-1, Tensor2D<M, N, H>, [usize; M], Tensor1D<M, H>, {M, N});
impl_select!(-1, Tensor2D<M, N, H>, [[usize; Z]; M], Tensor2D<M, Z, H>, {M, N, Z});

// 3d
impl_select!(0, Tensor3D<M, N, O, H>, usize, Tensor2D<N, O, H>, {M, N, O});
impl_select!(0, Tensor3D<M, N, O, H>, [usize; Z], Tensor3D<Z, N, O, H>, {M, N, O, Z});
impl_select!(1, Tensor3D<M, N, O, H>, [usize; M], Tensor2D<M, O, H>, {M, N, O});
impl_select!(1, Tensor3D<M, N, O, H>, [[usize; Z]; M], Tensor3D<M, Z, O, H>, {M, N, O, Z});
impl_select!(-1, Tensor3D<M, N, O, H>, [[usize; N]; M], Tensor2D<M, N, H>, {M, N, O});
impl_select!(-1, Tensor3D<M, N, O, H>, [[[usize; Z]; N]; M], Tensor3D<M, N, Z, H>, {M, N, O, Z});

// 4d
impl_select!(0, Tensor4D<M, N, O, P, H>, usize, Tensor3D<N, O, P, H>, {M, N, O, P});
impl_select!(0, Tensor4D<M, N, O, P, H>, [usize; Z], Tensor4D<Z, N, O, P, H>, {M, N, O, P, Z});
impl_select!(1, Tensor4D<M, N, O, P, H>, [usize; M], Tensor3D<M, O, P, H>, {M, N, O, P});
impl_select!(1, Tensor4D<M, N, O, P, H>, [[usize; Z]; M], Tensor4D<M, Z, O, P, H>, {M, N, O, P, Z});
impl_select!(2, Tensor4D<M, N, O, P, H>, [[usize; N]; M], Tensor3D<M, N, P, H>, {M, N, O, P});
impl_select!(2, Tensor4D<M, N, O, P, H>, [[[usize; Z]; N]; M], Tensor4D<M, N, Z, P, H>, {M, N, O, P, Z});
impl_select!(-1, Tensor4D<M, N, O, P, H>, [[[usize; O]; N]; M], Tensor3D<M, N, O, H>, {M, N, O, P});
impl_select!(-1, Tensor4D<M, N, O, P, H>, [[[[usize; Z]; O]; N]; M], Tensor4D<M, N, O, Z, H>, {M, N, O, P, Z});

#[cfg(test)]
mod tests {
    use rand::thread_rng;

    use super::*;

    #[test]
    fn test_valid_selects_1d() {
        let _: Tensor0D = Tensor1D::<5>::zeros().select(&0);
        let _: Tensor1D<3> = Tensor1D::<5>::zeros().select(&[1, 2, 3]);
        let _: Tensor1D<10> = Tensor1D::<5>::zeros().select(&[0, 1, 2, 3, 4, 0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_select_1d_backward() {
        let mut rng = thread_rng();
        let t: Tensor1D<5> = TensorCreator::randn(&mut rng);
        let r: Tensor0D<OwnedTape> = t.trace().select(&0);
        assert_eq!(r.data(), &t.data()[0]);
        let g = r.exp().mean().backward();
        assert_eq!(g.ref_gradient(&t), &[t.data()[0].exp(), 0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_select_1d_less_backward() {
        let mut rng = thread_rng();
        let t: Tensor1D<5> = TensorCreator::randn(&mut rng);
        let r: Tensor1D<2, OwnedTape> = t.trace().select(&[0, 3]);
        assert_eq!(r.data(), &[t.data()[0], t.data()[3]]);
        let g = r.mean().backward();
        assert_eq!(g.ref_gradient(&t), &[0.5, 0.0, 0.0, 0.5, 0.0]);
    }

    #[test]
    fn test_select_1d_more_backward() {
        let mut rng = thread_rng();
        let t: Tensor1D<5> = TensorCreator::randn(&mut rng);
        let _t = *t.data();
        let r: Tensor1D<8, OwnedTape> = t.trace().select(&[0, 1, 2, 3, 4, 2, 4, 4]);
        assert_eq!(
            r.data(),
            &[_t[0], _t[1], _t[2], _t[3], _t[4], _t[2], _t[4], _t[4]]
        );
        let g = r.mean().backward();
        assert_eq!(
            g.ref_gradient(&t),
            &[1.0 / 8.0, 1.0 / 8.0, 2.0 / 8.0, 1.0 / 8.0, 3.0 / 8.0]
        );
    }

    #[test]
    fn test_select_last_1d() {
        let t: Tensor1D<3> = Tensor1D::new([1.0, 2.0, 3.0]);
        let r: Tensor0D<OwnedTape> = t.trace().select(&2);
        assert_eq!(r.data(), &3.0);
        // NOTE: .exp() so we make sure its using result grad properly
        let gradients = r.exp().backward();
        assert_eq!(gradients.ref_gradient(&t), &[0.0, 0.0, 20.085537]);
    }

    #[test]
    fn test_select_last_2d() {
        let t: Tensor2D<2, 3> = Tensor2D::new([[1.0, 2.0, 3.0], [-1.0, -2.0, -3.0]]);
        let r: Tensor1D<2, OwnedTape> = t.trace().select(&[1, 2]);
        assert_eq!(r.data(), &[2.0, -3.0]);
        let gradients = r.mean().backward();
        assert_eq!(
            gradients.ref_gradient(&t),
            &[[0.0, 0.5, 0.0], [0.0, 0.0, 0.5]]
        );
    }

    #[test]
    fn test_select_last_3d() {
        let t: Tensor3D<4, 2, 3> = Tensor3D::new([
            [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]],
            [[-1.0, -2.0, -3.0], [-4.0, -5.0, -6.0]],
            [[-3.0, 2.0, -1.0], [-6.0, 5.0, -4.0]],
            [[1.0, -2.0, 3.0], [4.0, -5.0, 6.0]],
        ]);
        let r: Tensor2D<4, 2, OwnedTape> = t.trace().select(&[[0, 1], [2, 2], [1, 1], [0, 0]]);
        assert_eq!(
            r.data(),
            &[[1.0, 5.0], [-3.0, -6.0], [2.0, 5.0], [1.0, 4.0]]
        );
        let gradients = r.mean().backward();
        assert_eq!(
            gradients.ref_gradient(&t),
            &[
                [[0.125, 0.0, 0.0], [0.0, 0.125, 0.0]],
                [[0.0, 0.0, 0.125], [0.0, 0.0, 0.125]],
                [[0.0, 0.125, 0.0], [0.0, 0.125, 0.0]],
                [[0.125, 0.0, 0.0], [0.125, 0.0, 0.0]]
            ]
        );
    }
}
