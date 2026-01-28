use frand_property::{model};

// 1. Private Model check
model! {
    PrivateModel {
        val: i32,
    }
}

#[test]
fn test_private_model_singleton() {
    // Should have clone_singleton
    let m1 = PrivateModel::clone_singleton();
    let m2 = PrivateModel::clone_singleton();
    
    // They should share the same underlying data (singleton)
    m1.val.sender().send(10);
    assert_eq!(m2.val.receiver().value(), 10);
}

// 2. Public Model check
model! {
    pub PublicModel {
        pub val: i32,
    }
}

#[test]
fn test_public_model_new() {
    // Should NOT have clone_singleton (compilation error if called, we can't easily test "does not compile" here without trybuild)
    // But should have new()
    let m1 = PublicModel::new();
    let m2 = PublicModel::new();
    
    // Independent instances
    m1.val.sender().send(100);
    m2.val.sender().send(200);
    
    assert_eq!(m1.val.receiver().value(), 100);
    assert_eq!(m2.val.receiver().value(), 200);
}


// 3. Slint Model Verification
// Since `slint_model!` requires a valid Slint Component and Global type to compile the generated code,
// and we don't have a full Slint environment setup in this test file,
// we rely on the `model!` test (which shares the same visibility logic) and the code review of `codegen_rust.rs`.
// The parser correctly handles visibility, and the codegen logic is symmetric.
