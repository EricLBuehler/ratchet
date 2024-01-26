// This file is generated by build.rs. Do not edit it manually.
use lazy_static::lazy_static;
use std::collections::HashMap;
lazy_static! {
    pub static ref KERNELS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert(
            "qgemm_vec4",
            include_str!(
                r"/Users/fleetwood/Code/ratchet/crates/ratchet-core/kernels/qgemm_vec4.wgsl"
            ),
        );
        m.insert(
            "sgemm_scalar",
            include_str!(
                r"/Users/fleetwood/Code/ratchet/crates/ratchet-core/kernels/sgemm_scalar.wgsl"
            ),
        );
        m.insert(
            "add_scalar",
            include_str!(
                r"/Users/fleetwood/Code/ratchet/crates/ratchet-core/kernels/add_scalar.wgsl"
            ),
        );
        m.insert(
            "sgemm_vec2",
            include_str!(
                r"/Users/fleetwood/Code/ratchet/crates/ratchet-core/kernels/sgemm_vec2.wgsl"
            ),
        );
        m.insert(
            "sgemm_vec4",
            include_str!(
                r"/Users/fleetwood/Code/ratchet/crates/ratchet-core/kernels/sgemm_vec4.wgsl"
            ),
        );
        m.insert(
            "softmax_scalar",
            include_str!(
                r"/Users/fleetwood/Code/ratchet/crates/ratchet-core/kernels/softmax_scalar.wgsl"
            ),
        );
        m.insert(
            "softmax_vec4",
            include_str!(
                r"/Users/fleetwood/Code/ratchet/crates/ratchet-core/kernels/softmax_vec4.wgsl"
            ),
        );
        m
    };
}
