use hyper_transpiler::{GenerateOptions, Pipeline};

#[test]
fn test_inspect_attrs() {
    let source = "<button {is_active}>Click</button>\n<div class={my_class}></div>\n<div {**spread}></div>\n";
    let mut pipeline = Pipeline::standard();
    let mut options = GenerateOptions::default();
    options.include_ranges = true;
    let result = pipeline.compile(source, &options).unwrap();

    println!("Injections count: {}", result.injections.len());
    for inj in &result.injections {
        if inj.injection_type == "python" {
            let extracted = source
                .encode_utf16()
                .skip(inj.start)
                .take(inj.end - inj.start)
                .collect::<Vec<u16>>();
            let text = String::from_utf16(&extracted).unwrap();
            println!(
                "INJ: [{}..{}] = {:?} | prefix: {:?} | suffix: {:?}",
                inj.start, inj.end, text, inj.prefix, inj.suffix
            );
        }
    }
}
