pub fn platform_description() -> String {
    let mut description = String::new();

    #[cfg(target_arch = "x86_64")]
    description.push_str("x86-64");

    #[cfg(target_pointer_width = "64")]
    description.push_str(" 64-bit");

    macro_rules! check_features {
        ($($feature:literal),*) => {
            $(
                #[cfg(target_feature = $feature)]
                {
                    description.push_str(" ");
                    description.push_str($feature);
                }
            )*
        };
    }

    check_features!(
        "avx",
        "avx2",
        "bmi1",
        "bmi2",
        "fma",
        "lzcnt",
        "movbe",
        "pclmulqdq",
        "popcnt",
        "rdrand",
        "rdseed",
        "sse",
        "sse2",
        "sse3",
        "sse4.1",
        "sse4.2",
        "ssse3"
    );
    description
}
