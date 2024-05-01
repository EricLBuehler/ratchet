use derive_new::new;

use crate::{DType, OpGuards, Operation, OperationError, StorageView, Tensor};

#[derive(new, Debug, Clone)]
pub struct Norm {
    pub(crate) input: Tensor,
    pub(crate) scale: Tensor,
    pub(crate) bias: Option<Tensor>,
    pub(crate) eps: f32,
}

#[derive(new, Debug, Clone)]
pub struct GroupNorm {
    pub norm: Norm,
    pub num_groups: usize,
}

impl OpGuards for GroupNorm {
    fn check_shapes(&self) {
        assert!(self.input.rank() >= 3);
        let channels = self.input.shape()[1];
        assert!(channels % self.num_groups == 0);
        todo!("check that the input channels is divisible by self.num_groups")
    }

    fn check_dtypes(&self) {
        assert!(self.input.dt() == DType::F32);
        assert!(self.scale.dt() == DType::F32);
        if self.bias.is_some() {
            assert!(self.bias.as_ref().unwrap().dt() == DType::F32);
        }
    }
}

impl Operation for GroupNorm {
    fn compute_view(&self) -> Result<StorageView, OperationError> {
        Ok(self.input.storage_view().clone())
    }
}
