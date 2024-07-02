use encase::ShaderType;
use ratchet_macros::WgslMetadata;

use crate::{
    gpu::dtype::WgslDType, rvec, Array, BindingMode, BuiltIn, DType, InvariantError, Kernel,
    KernelElement, KernelRenderable, KernelSource, Matmul, MatmulSpec, OperationError, Scalar,
    Strides, Tensor, Vec4, WgslKernelBuilder, WgslPrimitive, WorkgroupSize,
};
use glam::IVec3;
use inline_wgsl::wgsl;
use num_traits::Zero;

#[allow(clippy::too_many_arguments)]
#[derive(Debug, Clone, ShaderType, WgslMetadata)]
pub struct WorkgroupGEMVMeta {
    aShape: IVec3,
    aStrides: IVec3,
    bShape: IVec3,
    bStrides: IVec3,
    outShape: IVec3,
    outStrides: IVec3,
    dimAOuter: i32,
    dimBOuter: i32,
    dimInner: i32,
}

#[derive(Debug, Clone)]
pub struct WorkgroupGEMV {
    lhs: Tensor,
    rhs: Tensor,
    bias: Option<Tensor>,
    trans_lhs: bool,
    trans_rhs: bool,
    trans_out: bool,
    spec: MatmulSpec,
}

impl WorkgroupGEMV {
    pub fn from_matmul(matmul: &Matmul, spec: MatmulSpec) -> Self {
        let Matmul {
            lhs,
            rhs,
            bias,
            trans_lhs,
            trans_rhs,
            trans_out,
        } = matmul.clone();
        Self {
            lhs,
            rhs,
            bias,
            trans_lhs,
            trans_rhs,
            trans_out,
            spec,
        }
    }
}

impl Kernel for WorkgroupGEMV {
    type Metadata = WorkgroupGEMVMeta;

    fn kernel_name(&self) -> String {
        "workgroup_gemv".to_string()
    }

    fn metadata(&self, _: &Tensor, _: &KernelElement) -> Result<Self::Metadata, OperationError> {
        let spec = &self.spec;
        let mut lhs_shape = spec.lhs_shape.clone();
        lhs_shape.insert(0, spec.lhs_stack());
        let aStrides = Strides::from(&lhs_shape);

        let mut rhs_shape = spec.rhs_shape.clone();
        rhs_shape.insert(0, spec.rhs_stack());
        let bStrides = Strides::from(&rhs_shape);

        let mut out_shape = spec.out_shape.clone();
        out_shape.insert(0, spec.stacks());
        let outStrides = Strides::from(&out_shape);

        let dimAOuter = spec.dim_lhs_outer() as i32;
        let dimBOuter = spec.dim_rhs_outer() as i32;
        let dimInner = spec.dim_inner() as i32;

        Ok(WorkgroupGEMVMeta {
            aShape: lhs_shape.into(),
            aStrides: aStrides.into(),
            bShape: rhs_shape.into(),
            bStrides: bStrides.into(),
            outShape: out_shape.into(),
            outStrides: outStrides.into(),
            dimAOuter,
            dimBOuter,
            dimInner,
        })
    }

    fn calculate_dispatch(&self, dst: &Tensor) -> Result<crate::Workload, OperationError> {
        todo!()
    }

    fn kernel_element(&self, dst: &Tensor) -> KernelElement {
        todo!()
    }

    fn build_kernel(
        &self,
        inplace: bool,
        dst: &Tensor,
        workgroup_size: &WorkgroupSize,
    ) -> Result<KernelSource, OperationError> {
        todo!()
    }
}

impl KernelRenderable for WorkgroupGEMV {
    fn register_bindings<P: WgslPrimitive>(
        &self,
        builder: &mut WgslKernelBuilder,
        _: bool,
    ) -> Result<(), OperationError> {
        let (A, _, bias) = (&self.lhs, &self.rhs, &self.bias);

        if A.dt().is_float() {
            let float_arr = Array::<P>::default();
            builder.register_storage("A", BindingMode::ReadOnly, float_arr);
            builder.register_storage("X", BindingMode::ReadOnly, float_arr);
            if bias.is_some() {
                builder.register_storage("bias", BindingMode::ReadOnly, float_arr);
            }
            builder.register_storage("result", BindingMode::ReadWrite, float_arr);
        } else if A.dt().is_quantized() {
            let scalar = Array::<Scalar<P::T>>::default();
            builder.register_storage("A", BindingMode::ReadOnly, Array::<Scalar<u32>>::default());
            builder.register_storage("scale", BindingMode::ReadOnly, scalar);
            builder.register_storage("X", BindingMode::ReadOnly, Array::<Vec4<P::T>>::default());
            if bias.is_some() {
                builder.register_storage("bias", BindingMode::ReadOnly, scalar);
            }
            builder.register_storage("result", BindingMode::ReadWrite, scalar);
        } else {
            return Err(InvariantError::UnsupportedDType(A.dt()).into());
        }

        builder.register_uniform();
        Ok(())
    }

    fn render<P: WgslPrimitive>(
        &self,
        inplace: bool,
        dst: &Tensor,
        workgroup_size: &WorkgroupSize,
    ) -> Result<KernelSource, OperationError> {
        let device = self.lhs.device().try_gpu().unwrap();
        let mut kernel_builder = WgslKernelBuilder::new(
            workgroup_size.clone(),
            rvec![
                BuiltIn::GlobalInvocationId,
                BuiltIn::LocalInvocationId,
                BuiltIn::WorkgroupId,
            ],
            device.compute_features().clone(),
        );

        self.register_bindings::<P>(&mut kernel_builder, inplace)?;
        let n = P::W;
        let fp32_accessor = match n {
            1 => "f32",
            2 => "vec2<f32>",
            4 => "vec4<f32>",
            _ => unimplemented!(),
        };
        let scalar = P::T::DT;
        let zero = P::T::zero().render();

        //kernel_builder.render_metadata::<WorkgroupGEMVMeta>();
        kernel_builder.write_unpack(self.lhs.dt());

        let work_size = (workgroup_size.x * workgroup_size.y / (n as u32)).render();
        kernel_builder.write_global(wgsl! {
            var<workgroup> work: array<'fp32_accessor, 'work_size>;
        });

        let (TILE_X, _) = self.spec.heuristic.as_workgroup_size();
        let A_FIT = self.spec.lhs_shape()[1] % TILE_X == 0;

        let readA = match (A_FIT, self.lhs.dt()) {
            (true, DType::F32) | (true, DType::F16) => {
                wgsl! {
                    fn readA(batch: i32, row: i32, col: i32) -> 'scalar {
                        return A[dot(metadata.aStrides, vec3<i32>(batch, row, col))];
                    }
                }
            }
            (false, DType::F32) | (false, DType::F16) => {
                wgsl! {
                    fn readA(batch: i32, row: i32, col: i32) -> 'scalar {
                        var val = 'zero;
                        if (row <= metadata.aShape.y) {
                            val = A[dot(metadata.aStrides, vec3<i32>(batch, row, col))];
                        }
                        return val;
                    }
                }
            }
            (true, DType::Q8_0F(_)) | (true, DType::Q8_0H(_)) => {
                wgsl! {
                    fn readA(batch: i32, row: i32, col: i32) -> vec4<'scalar> {
                        return unpack(A[dot(metadata.aStrides, vec3<i32>(batch, row, col))]);
                    }
                }
            }
            _ => unimplemented!(),
        };
        kernel_builder.write_global(readA);

        kernel_builder.write_main(wgsl! { let row = i32(global_invocation_id.x); });

        kernel_builder.write_main(wgsl! {
            let batch = i32(global_invocation_id.z);
            let batchA = batch % metadata.aShape.x;
            let batchB = batch % metadata.bShape.x;
        });

        kernel_builder.write_main(wgsl! {
            let aOffset = metadata.aStrides.x * batchA / 'n;
            let bOffset = metadata.bStrides.x * batchB / 'n;
            let outOffset = metadata.outStrides.x * batch / 'n;
        });

        kernel_builder.write_main(wgsl! { var sum = 'fp32_accessor(0.0); });
        kernel_builder.write_main(wgsl! { let aIndex = aOffset + row * metadata.aStrides.y / 'n; });

        let workgroup_size_y = workgroup_size.y;
        let main_loop = match self.lhs.dt() {
            DType::Q8_0F(_) | DType::Q8_0H(_) => {
                wgsl! {
                    let sIndex = (aOffset / 4) + row * metadata.aStrides.y / 32;
                    for (var k = i32(global_invocation_id.y); k < metadata.dimInner / 4; k+='workgroup_size_y / 4) {
                        sum += 'fp32_accessor(unpack(A[aIndex + k]) * scale[sIndex + (k/8)] * X[k]);
                    }
                }
            }
            _ => {
                wgsl! {
                    for (var k = i32(global_invocation_id.y); k < metadata.dimInner; k+='workgroup_size_y) {
                        sum += 'fp32_accessor(readA(batchA, row, k) * X[bOffset + k]);
                    }
                }
            }
        };

        kernel_builder.write_main(main_loop);

        let workgroup_size_x = workgroup_size.x.render();
        let workgroup_size_y = workgroup_size.y.render();
        kernel_builder.write_main(wgsl! {
            let rows = 'workgroup_size_x;
            let cols = 'workgroup_size_y / 'n;

            let ii = u32(local_invocation_id.x);
            let jj = u32(local_invocation_id.y);
            work[ii + rows * jj] = sum;
            workgroupBarrier();

            // Reduce sums in log2(cols) steps
            for (var s = u32(cols) / 2u; s > 0u; s >>= 1u) {
                if (jj < s) {
                    work[ii + rows * jj] += work[ii + rows * (jj + s)];
                }
                workgroupBarrier();
            }
        });

        let bias = if self.bias.is_some() {
            wgsl! { bias[row] }
        } else {
            wgsl! { 0. }
        };

        let finalizer = match P::W {
            4 | 2 => {
                wgsl! { result[outOffset + row] = 'scalar(dot(work[ii], 'fp32_accessor(1.0)) + f32('bias));}
            }
            1 => wgsl! { result[outOffset + row] = 'scalar(work[ii] + f32('bias)); },
            _ => unimplemented!(),
        };

        kernel_builder.write_main(wgsl! {
            if (jj == 0) {
                'finalizer
            }
        });

        Ok(kernel_builder.build()?)
    }
}
