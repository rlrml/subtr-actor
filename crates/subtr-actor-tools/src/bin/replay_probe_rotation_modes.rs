use super::rotation_types::{EulerMode, EulerRotationOrder, EulerScale, QuaternionMode};

pub(crate) fn build_modes() -> Vec<QuaternionMode> {
    let orders = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];
    let signs = [
        [1, 1, 1],
        [1, 1, -1],
        [1, -1, 1],
        [1, -1, -1],
        [-1, 1, 1],
        [-1, 1, -1],
        [-1, -1, 1],
        [-1, -1, -1],
    ];

    let mut modes = Vec::new();
    for missing_slot in 0..4 {
        for order in orders {
            for sign in signs {
                for reconstruct_missing in [false, true] {
                    modes.push(QuaternionMode {
                        missing_slot,
                        order,
                        signs: sign,
                        reconstruct_missing,
                    });
                }
            }
        }
    }
    modes
}

pub(crate) fn build_euler_modes() -> Vec<EulerMode> {
    let orders = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];
    let signs = [
        [1, 1, 1],
        [1, 1, -1],
        [1, -1, 1],
        [1, -1, -1],
        [-1, 1, 1],
        [-1, 1, -1],
        [-1, -1, 1],
        [-1, -1, -1],
    ];
    let scales = [EulerScale::Pi, EulerScale::TwoPi, EulerScale::HalfPi];
    let rotation_orders = [
        EulerRotationOrder::Xyz,
        EulerRotationOrder::Xzy,
        EulerRotationOrder::Yxz,
        EulerRotationOrder::Yzx,
        EulerRotationOrder::Zxy,
        EulerRotationOrder::Zyx,
    ];

    let mut modes = Vec::new();
    for order in orders {
        for sign in signs {
            for scale in scales {
                for rotation_order in rotation_orders {
                    modes.push(EulerMode {
                        order,
                        signs: sign,
                        scale,
                        rotation_order,
                    });
                }
            }
        }
    }
    modes
}
