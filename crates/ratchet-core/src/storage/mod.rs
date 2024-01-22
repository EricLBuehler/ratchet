mod cpu_buffer;
mod gpu_buffer;

use bytemuck::NoUninit;
pub use cpu_buffer::*;
pub use gpu_buffer::*;

use crate::{Device, DeviceError, Shape};

use crate::DType;

#[derive(Debug)]
pub enum Storage {
    CPU(CPUBuffer),
    GPU(GPUBuffer),
}

impl Storage {
    pub fn from_slice<T: NoUninit>(data: &[T], shape: &Shape, device: &Device) -> Self {
        match device {
            Device::CPU => Storage::CPU(CPUBuffer::from_slice(data, shape)),
            Device::GPU(g) => Storage::GPU(GPUBuffer::from_slice(data, shape, g)),
        }
    }

    pub fn dump(&self, dt: DType, full: bool) -> String {
        match self {
            Storage::CPU(c) => c.dump(dt, full),
            Storage::GPU(g) => g.dump(dt, full),
        }
    }

    pub fn try_cpu(&self) -> Result<&CPUBuffer, DeviceError> {
        match self {
            Storage::CPU(c) => Ok(c),
            _ => unimplemented!(),
        }
    }

    pub fn try_gpu(&self) -> Result<&GPUBuffer, DeviceError> {
        match self {
            Storage::GPU(g) => Ok(g),
            _ => unimplemented!(),
        }
    }

    pub fn deep_clone(&self) -> Result<Self, DeviceError> {
        match self {
            Storage::CPU(c) => Ok(Storage::CPU(c.deep_clone()?)),
            _ => todo!(),
        }
    }
}

pub trait DeviceStorage: std::fmt::Debug + Clone + 'static {
    // To be expanded to other devices
    fn to_device(&self, device: &Device) -> Result<GPUBuffer, DeviceError>;
    /// Creates a copy of the device buffer on the CPU
    fn to_cpu(&self, device: &Device) -> Result<CPUBuffer, DeviceError>;
    fn n_bytes(&self) -> usize;
    fn dump(&self, dt: DType, full: bool) -> String;
}